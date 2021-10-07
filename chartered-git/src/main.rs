#![deny(clippy::pedantic)]
mod generators;
#[allow(clippy::missing_errors_doc)]
pub mod git;

use crate::{
    generators::CargoConfig,
    git::{
        codec::{Encoder, GitCodec},
        packfile::{
            high_level::GitRepository,
            low_level::{Commit, CommitUserInfo, PackFile, PackFileEntry, TreeItem, TreeItemKind},
        },
        PktLine,
    },
};

use bytes::BytesMut;
use chrono::TimeZone;
use futures::future::Future;
use log::warn;
use std::collections::BTreeMap;
use std::{fmt::Write, pin::Pin, sync::Arc};
use thrussh::{
    server::{self, Auth, Session},
    ChannelId, CryptoVec,
};
use thrussh_keys::{key, PublicKeyBase64};
use tokio_util::codec::{Decoder, Encoder as TokioEncoder};
use url::Url;

#[tokio::main]
#[allow(clippy::semicolon_if_nothing_returned)] // broken clippy lint
async fn main() {
    env_logger::init();

    let config = Arc::new(thrussh::server::Config {
        methods: thrussh::MethodSet::PUBLICKEY,
        keys: vec![key::KeyPair::generate_ed25519().unwrap()],
        ..thrussh::server::Config::default()
    });

    let server = Server {
        db: chartered_db::init().unwrap(),
    };

    thrussh::server::run(config, "127.0.0.1:2233", server)
        .await
        .unwrap();
}

#[derive(Clone)]
struct Server {
    db: chartered_db::ConnectionPool,
}

impl server::Server for Server {
    type Handler = Handler;

    fn new(&mut self, ip: Option<std::net::SocketAddr>) -> Self::Handler {
        Handler {
            ip,
            codec: GitCodec::default(),
            input_bytes: BytesMut::default(),
            output_bytes: BytesMut::default(),
            db: self.db.clone(),
            authed: None,
            organisation: None,
        }
    }
}

struct Handler {
    ip: Option<std::net::SocketAddr>,
    codec: GitCodec,
    input_bytes: BytesMut,
    output_bytes: BytesMut,
    db: chartered_db::ConnectionPool,
    organisation: Option<String>,
    authed: Option<Authed>,
}

struct Authed {
    user: chartered_db::users::User,
    auth_key: String,
}

impl Handler {
    fn write(&mut self, packet: PktLine<'_>) -> Result<(), anyhow::Error> {
        Encoder {}.encode(packet, &mut self.output_bytes)
    }

    fn flush(&mut self, session: &mut Session, channel: ChannelId) {
        session.data(
            channel,
            CryptoVec::from_slice(self.output_bytes.split().as_ref()),
        );
    }

    fn authed(&self) -> Result<&Authed, anyhow::Error> {
        match self.authed {
            Some(ref authed) => Ok(authed),
            None => anyhow::bail!("user not set after auth"),
        }
    }

    fn org_name(&self) -> Result<&str, anyhow::Error> {
        match self.organisation {
            Some(ref org) => Ok(org.as_str()),
            None => anyhow::bail!("org not set after auth"),
        }
    }
}

type AsyncHandlerFut<T> =
    dyn Future<Output = Result<T, <Handler as server::Handler>::Error>> + Send;

impl server::Handler for Handler {
    type Error = anyhow::Error;
    type FutureAuth = Pin<Box<AsyncHandlerFut<(Handler, server::Auth)>>>;
    type FutureUnit = Pin<Box<AsyncHandlerFut<(Handler, Session)>>>;
    type FutureBool = futures::future::Ready<Result<(Self, Session, bool), anyhow::Error>>;

    fn finished_auth(self, auth: Auth) -> Self::FutureAuth {
        Box::pin(futures::future::ready(Ok((self, auth))))
    }

    fn finished_bool(self, b: bool, s: Session) -> Self::FutureBool {
        futures::future::ready(Ok((self, s, b)))
    }

    fn finished(self, s: Session) -> Self::FutureUnit {
        Box::pin(futures::future::ready(Ok((self, s))))
    }

    fn shell_request(mut self, channel: ChannelId, mut session: Session) -> Self::FutureUnit {
        Box::pin(async move {
            let username = self.authed()?.user.username.clone(); // todo
            write!(&mut self.output_bytes, "Hi there, {}! You've successfully authenticated, but chartered does not provide shell access.\r\n", username)?;
            self.flush(&mut session, channel);
            session.close(channel);
            Ok((self, session))
        })
    }

    fn exec_request(
        mut self,
        channel: ChannelId,
        data: &[u8],
        mut session: Session,
    ) -> Self::FutureUnit {
        let data = match std::str::from_utf8(data) {
            Ok(data) => data,
            Err(e) => return Box::pin(futures::future::err(e.into())),
        };
        let args = shlex::split(data);

        Box::pin(async move {
            let mut args = args.into_iter().flat_map(Vec::into_iter);

            if args.next().as_deref() != Some("git-upload-pack") {
                anyhow::bail!("not git-upload-pack");
            }

            if let Some(org) = args.next().filter(|v| v.as_str() != "/") {
                let org = org
                    .trim_start_matches('/')
                    .trim_end_matches('/')
                    .to_string();
                self.organisation = Some(org);
            } else {
                session.extended_data(channel, 1, CryptoVec::from_slice(indoc::indoc! {b"
                    \r\nNo organisation was given in the path part of the SSH URI. A chartered registry should be defined in your .cargo/config.toml as follows:
                        [registries]
                        chartered = {{ index = \"ssh://domain.to.registry.com/my-organisation\" }}\r\n
                "}));
                session.close(channel);
            }

            // TODO: check GIT_PROTOCOL=version=2 set
            self.write(PktLine::Data(b"version 2\n"))?;
            self.write(PktLine::Data(b"agent=chartered/0.1.0\n"))?;
            self.write(PktLine::Data(b"ls-refs=unborn\n"))?;
            self.write(PktLine::Data(b"fetch=shallow wait-for-done\n"))?;
            self.write(PktLine::Data(b"server-option\n"))?;
            self.write(PktLine::Data(b"object-info\n"))?;
            self.write(PktLine::Flush)?;
            self.flush(&mut session, channel);

            Ok((self, session))
        })
    }

    fn subsystem_request(
        self,
        _channel: ChannelId,
        data: &str,
        session: Session,
    ) -> Self::FutureUnit {
        eprintln!("subsystem req: {}", data);
        Box::pin(futures::future::ready(Ok((self, session))))
    }

    fn auth_publickey(mut self, _username: &str, key: &key::PublicKey) -> Self::FutureAuth {
        let public_key = key.public_key_bytes();

        Box::pin(async move {
            let (ssh_key, user) =
                match chartered_db::users::User::find_by_ssh_key(self.db.clone(), public_key)
                    .await?
                {
                    Some(user) => user,
                    None => return self.finished_auth(server::Auth::Reject).await,
                };
            let ssh_key = Arc::new(ssh_key);

            if let Err(e) = ssh_key.clone().update_last_used(self.db.clone()).await {
                warn!("Failed to update last used key: {:?}", e);
            }

            let auth_key = ssh_key
                .clone()
                .get_or_insert_session(self.db.clone(), self.ip.map(|v| v.to_string()))
                .await?
                .session_key;

            self.authed = Some(Authed { user, auth_key });

            self.finished_auth(server::Auth::Accept).await
        })
    }

    fn auth_keyboard_interactive(
        self,
        _user: &str,
        _submethods: &str,
        _response: Option<server::Response>,
    ) -> Self::FutureAuth {
        self.finished_auth(server::Auth::UnsupportedMethod)
    }

    fn auth_none(self, _user: &str) -> Self::FutureAuth {
        self.finished_auth(server::Auth::UnsupportedMethod)
    }

    fn auth_password(self, _user: &str, _password: &str) -> Self::FutureAuth {
        self.finished_auth(server::Auth::UnsupportedMethod)
    }

    fn data(mut self, channel: ChannelId, data: &[u8], mut session: Session) -> Self::FutureUnit {
        self.input_bytes.extend_from_slice(data);

        Box::pin(async move {
            let mut ls_refs = false;
            let mut fetch = false;
            let mut done = false;

            while let Some(frame) = self.codec.decode(&mut self.input_bytes)? {
                eprintln!("{:#?}", frame);

                // if the client flushed without giving us a command, we're expected to close
                // the connection or else the client will just hang
                if frame.command.is_empty() {
                    session.exit_status_request(channel, 0);
                    session.eof(channel);
                    session.close(channel);
                    return Ok((self, session));
                }

                if frame.command.as_ref() == "command=ls-refs".as_bytes() {
                    ls_refs = true;
                } else if frame.command.as_ref() == "command=fetch".as_bytes() {
                    if frame.metadata.iter().any(|v| v.as_ref() == b"done") {
                        done = true;
                    } else {
                        fetch = true;
                    }
                }
            }

            let authed = self.authed()?;
            let org_name = self.org_name()?;

            if !ls_refs && !fetch && !done {
                return Ok((self, session));
            }

            let mut packfile = GitRepository::default();

            let config = CargoConfig::new(
                Url::parse("http://127.0.0.1:8888/")?,
                &authed.auth_key,
                org_name,
            );
            let config = serde_json::to_vec(&config)?;
            packfile.insert(vec![], "config.json".to_string(), &config);

            // todo: the whole tree needs caching and then we can filter in code rather than at
            //  the database
            let tree = fetch_tree(self.db.clone(), authed.user.id, org_name.to_string()).await;
            build_tree(&mut packfile, &tree)?;

            let (commit_hash, packfile_entries) =
                packfile.commit("computer", "john@computer.no", "Update crates");

            eprintln!("commit hash: {}", hex::encode(&commit_hash));

            // echo -ne "0014command=ls-refs\n0014agent=git/2.321\n00010009peel\n000csymrefs\n000bunborn\n0014ref-prefix HEAD\n0019ref-prefix refs/HEAD\n001eref-prefix refs/tags/HEAD\n001fref-prefix refs/heads/HEAD\n0021ref-prefix refs/remotes/HEAD\n0026ref-prefix refs/remotes/HEAD/HEAD\n001aref-prefix refs/tags/\n0000"
            // GIT_PROTOCOL=version=2 ssh -o SendEnv=GIT_PROTOCOL git@github.com git-upload-pack '/w4/chartered.git'
            // ''.join([('{:04x}'.format(len(v) + 5)), v, "\n"])
            // echo -ne "0012command=fetch\n0001000ethin-pack\n0010no-progress\n0010include-tag\n000eofs-delta\n0032want f6046cf6372e0d8ab845f6dec1602c303a66ee91\n"
            // sends a 000dpackfile back
            // https://shafiul.github.io/gitbook/7_the_packfile.html
            if ls_refs {
                let commit_hash = hex::encode(&commit_hash);
                self.write(PktLine::Data(
                    format!("{} HEAD symref-target:refs/heads/master\n", commit_hash).as_bytes(),
                ))?;
                self.write(PktLine::Flush)?;
                self.flush(&mut session, channel);
            }

            if fetch {
                self.write(PktLine::Data(b"acknowledgments\n"))?;
                self.write(PktLine::Data(b"ready\n"))?;
                self.write(PktLine::Delimiter)?;
                // self.write(PktLine::Data(b"shallow-info\n"))?;
                // self.write(PktLine::Data(b"unshallow\n"))?;
                done = true;
            }

            if done {
                self.write(PktLine::Data(b"packfile\n"))?;

                self.write(PktLine::SidebandMsg(b"Hello from chartered!\n"))?;
                self.flush(&mut session, channel);

                let packfile = PackFile::new(packfile_entries);
                self.write(PktLine::SidebandData(packfile))?;
                self.write(PktLine::Flush)?;
                self.flush(&mut session, channel);

                session.exit_status_request(channel, 0);
                session.eof(channel);
                session.close(channel);
            }

            Ok((self, session))
        })
    }
}

#[derive(serde::Serialize)]
pub struct CrateFileEntry<'a> {
    #[serde(flatten)]
    inner: &'a chartered_types::cargo::CrateVersion<'a>,
    cksum: &'a str,
    yanked: bool,
}

pub type TwoCharTree<T> = BTreeMap<[u8; 2], T>;

async fn fetch_tree(
    db: chartered_db::ConnectionPool,
    user_id: i32,
    org_name: String,
) -> TwoCharTree<TwoCharTree<BTreeMap<String, String>>> {
    use chartered_db::crates::Crate;

    let mut tree: TwoCharTree<TwoCharTree<BTreeMap<String, String>>> = BTreeMap::new();

    // todo: handle files with 1/2/3 characters
    for (crate_def, versions) in Crate::list_with_versions(db, user_id, org_name)
        .await
        .unwrap()
    {
        let mut name_chars = crate_def.name.as_bytes().iter();
        let first_dir = [*name_chars.next().unwrap(), *name_chars.next().unwrap()];
        let second_dir = [*name_chars.next().unwrap(), *name_chars.next().unwrap()];

        let first_dir = tree.entry(first_dir).or_default();
        let second_dir = first_dir.entry(second_dir).or_default();

        let mut file = String::new();
        for version in versions {
            let cksum = version.checksum.clone();
            let yanked = version.yanked;
            let version = version.into_cargo_format(&crate_def);

            let entry = CrateFileEntry {
                inner: &version,
                cksum: &cksum,
                yanked,
            };

            file.push_str(&serde_json::to_string(&entry).unwrap());
            file.push('\n');
        }

        second_dir.insert(crate_def.name, file);
    }

    tree
}

fn build_tree<'a>(
    packfile: &mut GitRepository<'a>,
    tree: &'a TwoCharTree<TwoCharTree<BTreeMap<String, String>>>,
) -> Result<(), anyhow::Error> {
    for (first_level_dir, second_level_dirs) in tree.iter() {
        let first_level_dir = std::str::from_utf8(first_level_dir)?;

        for (second_level_dir, crates) in second_level_dirs.iter() {
            let second_level_dir = std::str::from_utf8(second_level_dir)?;

            for (crate_name, versions_def) in crates.iter() {
                packfile.insert(
                    vec![first_level_dir.to_string(), second_level_dir.to_string()],
                    crate_name.to_string(),
                    versions_def.as_ref(),
                );
            }
        }
    }

    Ok(())
}
