#![deny(clippy::pedantic)]
#[allow(clippy::missing_errors_doc)]
pub mod git;

use crate::git::{
    codec::{Encoder, GitCodec},
    packfile::{Commit, CommitUserInfo, PackFileEntry, TreeItem, TreeItemKind},
    PktLine,
};

use bytes::BytesMut;
use chrono::TimeZone;
use futures::future::Future;
use std::collections::BTreeMap;
use std::{fmt::Write, pin::Pin, sync::Arc};
use thrussh::{
    server::{self, Auth, Session},
    ChannelId, CryptoVec,
};
use thrussh_keys::{key, PublicKeyBase64};
use tokio_util::codec::{Decoder, Encoder as TokioEncoder};

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

    fn new(&mut self, _: Option<std::net::SocketAddr>) -> Self::Handler {
        Handler {
            codec: GitCodec::default(),
            input_bytes: BytesMut::default(),
            output_bytes: BytesMut::default(),
            db: self.db.clone(),
            user: None,
        }
    }
}

struct Handler {
    codec: GitCodec,
    input_bytes: BytesMut,
    output_bytes: BytesMut,
    db: chartered_db::ConnectionPool,
    user: Option<chartered_db::users::User>,
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

    fn user(&self) -> Result<&chartered_db::users::User, anyhow::Error> {
        match self.user {
            Some(ref user) => Ok(user),
            None => anyhow::bail!("user not set after auth"),
        }
    }
}

type AsyncHandlerFn = Pin<
    Box<
        dyn Future<Output = Result<(Handler, Session), <Handler as server::Handler>::Error>> + Send,
    >,
>;

impl server::Handler for Handler {
    type Error = anyhow::Error;
    type FutureAuth = Pin<
        Box<
            dyn Future<
                    Output = Result<(Handler, server::Auth), <Handler as server::Handler>::Error>,
                > + Send,
        >,
    >;
    type FutureUnit = AsyncHandlerFn;
    type FutureBool = futures::future::Ready<Result<(Self, Session, bool), anyhow::Error>>;

    fn finished_auth(self, auth: Auth) -> Self::FutureAuth {
        eprintln!("finished auth");
        Box::pin(futures::future::ready(Ok((self, auth))))
    }

    fn finished_bool(self, b: bool, s: Session) -> Self::FutureBool {
        eprintln!("finished bool");
        futures::future::ready(Ok((self, s, b)))
    }

    fn finished(self, s: Session) -> Self::FutureUnit {
        eprintln!("finished");
        Box::pin(futures::future::ready(Ok((self, s))))
    }

    fn shell_request(mut self, channel: ChannelId, mut session: Session) -> Self::FutureUnit {
        Box::pin(async move {
            let username = self.user()?.username.clone(); // todo
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
        eprintln!("exec {:x?}", data);

        let git_upload_pack = data.starts_with(b"git-upload-pack ");

        Box::pin(async move {
            if git_upload_pack {
                // TODO: check GIT_PROTOCOL=version=2 set
                self.write(PktLine::Data(b"version 2\n"))?;
                self.write(PktLine::Data(b"agent=chartered/0.1.0\n"))?;
                self.write(PktLine::Data(b"ls-refs=unborn\n"))?;
                self.write(PktLine::Data(b"fetch=shallow wait-for-done\n"))?;
                self.write(PktLine::Data(b"server-option\n"))?;
                self.write(PktLine::Data(b"object-info\n"))?;
                self.write(PktLine::Flush)?;
                self.flush(&mut session, channel);
            } else {
                session.data(
                    channel,
                    CryptoVec::from_slice(b"Sorry, I have no clue who you are\r\n"),
                );
                session.close(channel);
            }

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
            let login_user =
                match chartered_db::users::User::find_by_ssh_key(self.db.clone(), public_key)
                    .await?
                {
                    Some(user) => user,
                    None => return self.finished_auth(server::Auth::Reject).await,
                };

            self.user = Some(login_user);
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
                eprintln!("data: {:x?}", frame);

                if frame.as_ref() == "command=ls-refs".as_bytes() {
                    ls_refs = true;
                } else if frame.as_ref() == "command=fetch".as_bytes() {
                    fetch = true;
                } else if frame.as_ref() == "done".as_bytes() {
                    fetch = false;
                    done = true;
                }
            }

            // echo -ne "0012command=fetch\n0001000ethin-pack\n0010include-tag\n000eofs-delta\n0032want d24d8020163b5fee57c9babfd0c595b8c90ba253\n0009done\n"

            let mut pack_file_entries = Vec::new();
            let mut root_tree = Vec::new();

            let config_file = PackFileEntry::Blob(
                br#"{"dl":"http://127.0.0.1:8888/a/abc/api/v1/crates","api":"http://127.0.0.1:8888/a/abc"}"#,
            );

            root_tree.push(TreeItem {
                kind: TreeItemKind::File,
                name: "config.json",
                hash: config_file.hash()?,
            });
            pack_file_entries.push(config_file);

            // todo: this needs caching and filtering
            let tree = fetch_tree(self.db.clone()).await;
            build_tree(&mut root_tree, &mut pack_file_entries, &tree)?;

            let root_tree = PackFileEntry::Tree(root_tree);
            let root_tree_hash = root_tree.hash()?;
            pack_file_entries.push(root_tree);

            let commit_user = CommitUserInfo {
                name: "Jordan Doyle",
                email: "jordan@doyle.la",
                time: chrono::Utc.ymd(2021, 9, 8).and_hms(17, 46, 1),
            };
            let commit = PackFileEntry::Commit(Commit {
                tree: root_tree_hash,
                author: commit_user,
                committer: commit_user,
                message: "Most recent crates",
            });
            let commit_hash = commit.hash()?;
            pack_file_entries.push(commit);

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

                let packfile = git::packfile::PackFile::new(pack_file_entries);
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
    name: &'a str,
    vers: &'a str,
    deps: &'a [&'a str],
    cksum: &'a str,
    features: BTreeMap<String, Vec<String>>,
    yanked: bool,
    links: Option<()>,
}

async fn fetch_tree(
    db: chartered_db::ConnectionPool,
) -> BTreeMap<[u8; 2], BTreeMap<[u8; 2], BTreeMap<String, String>>> {
    use chartered_db::crates::Crate;

    let mut tree: BTreeMap<[u8; 2], BTreeMap<[u8; 2], BTreeMap<String, String>>> = BTreeMap::new();

    // todo: handle files with 1/2/3 characters
    for (crate_def, versions) in Crate::all_with_versions(db).await.unwrap() {
        let mut name_chars = crate_def.name.as_bytes().iter();
        let first_dir = [*name_chars.next().unwrap(), *name_chars.next().unwrap()];
        let second_dir = [*name_chars.next().unwrap(), *name_chars.next().unwrap()];

        let first_dir = tree.entry(first_dir).or_default();
        let second_dir = first_dir.entry(second_dir).or_default();

        let mut file = String::new();
        for version in versions {
            let entry = CrateFileEntry {
                name: &crate_def.name,
                vers: &version.version,
                deps: &[],
                cksum: &version.checksum,
                features: BTreeMap::new(),
                yanked: version.yanked,
                links: None,
            };

            file.push_str(&serde_json::to_string(&entry).unwrap());
            file.push('\n');
        }

        second_dir.insert(crate_def.name, file);
    }

    tree
}

fn build_tree<'a>(
    root_tree: &mut Vec<TreeItem<'a>>,
    pack_file_entries: &mut Vec<PackFileEntry<'a>>,
    tree: &'a BTreeMap<[u8; 2], BTreeMap<[u8; 2], BTreeMap<String, String>>>,
) -> Result<(), anyhow::Error> {
    root_tree.reserve(tree.len());
    pack_file_entries.reserve(tree.iter().map(|(_, v)| 1 + v.len()).sum::<usize>() + tree.len());

    for (first_level_dir, second_level_dirs) in tree.iter() {
        let mut first_level_tree = Vec::with_capacity(second_level_dirs.len());

        for (second_level_dir, crates) in second_level_dirs.iter() {
            let mut second_level_tree = Vec::with_capacity(crates.len());

            for (crate_name, versions_def) in crates.iter() {
                let file = PackFileEntry::Blob(versions_def.as_ref());
                let file_hash = file.hash()?;
                pack_file_entries.push(file);

                second_level_tree.push(TreeItem {
                    kind: TreeItemKind::File,
                    name: crate_name,
                    hash: file_hash,
                });
            }

            let second_level_tree = PackFileEntry::Tree(second_level_tree);
            let second_level_tree_hash = second_level_tree.hash()?;
            pack_file_entries.push(second_level_tree);

            first_level_tree.push(TreeItem {
                kind: TreeItemKind::Directory,
                name: std::str::from_utf8(second_level_dir)?,
                hash: second_level_tree_hash,
            });
        }

        let first_level_tree = PackFileEntry::Tree(first_level_tree);
        let first_level_tree_hash = first_level_tree.hash()?;
        pack_file_entries.push(first_level_tree);

        root_tree.push(TreeItem {
            kind: TreeItemKind::Directory,
            name: std::str::from_utf8(first_level_dir)?,
            hash: first_level_tree_hash,
        });
    }

    Ok(())
}
