#![deny(clippy::pedantic)]
#![deny(rust_2018_idioms)]
mod command_handlers;
mod config;
mod generators;
mod tree;

#[allow(clippy::missing_errors_doc)]
pub mod git;

use crate::{
    generators::CargoConfig,
    git::{
        codec::{Encoder, GitCodec},
        packfile::high_level::GitRepository,
        PktLine,
    },
    tree::Tree,
};

use arrayvec::ArrayVec;
use bytes::BytesMut;
use clap::Parser;
use futures::future::Future;
use std::{fmt::Write, path::PathBuf, pin::Pin, sync::Arc};
use thrussh::{
    server::{self, Auth, Session},
    ChannelId, CryptoVec,
};
use thrussh_keys::{key, PublicKeyBase64};
use tokio_util::codec::{Decoder, Encoder as TokioEncoder};
use tracing::{debug, error, info, warn, Instrument};
use url::Url;

#[derive(Parser)]
#[clap(version = clap::crate_version!(), author = clap::crate_authors!())]
pub struct Opts {
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    #[clap(short, long)]
    config: PathBuf,
}

#[tokio::main]
#[allow(clippy::semicolon_if_nothing_returned)] // broken clippy lint
async fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();

    std::env::set_var(
        "RUST_LOG",
        match opts.verbose {
            1 => "debug",
            2 => "trace",
            _ => "info",
        },
    );

    let config: config::Config = toml::from_slice(&std::fs::read(&opts.config)?)?;

    tracing_subscriber::fmt::init();

    let trussh_config = Arc::new(thrussh::server::Config {
        methods: thrussh::MethodSet::PUBLICKEY,
        keys: vec![key::KeyPair::generate_ed25519().unwrap()],
        ..thrussh::server::Config::default()
    });

    let server = Server {
        db: chartered_db::init(&config.database_uri)?,
    };

    info!("SSH server listening on {}", config.bind_address);

    thrussh::server::run(trussh_config, &config.bind_address.to_string(), server).await?;

    Ok(())
}

#[derive(Clone)]
struct Server {
    db: chartered_db::ConnectionPool,
}

impl server::Server for Server {
    type Handler = Handler;

    fn new(&mut self, ip: Option<std::net::SocketAddr>) -> Self::Handler {
        let connection_id = chartered_db::uuid::Uuid::new_v4();
        let span = tracing::info_span!("ssh", "connection_id" = connection_id.to_string().as_str());

        span.in_scope(|| info!("Connection accepted from {:?}", ip));

        Handler {
            ip,
            span,
            codec: GitCodec::default(),
            input_bytes: BytesMut::default(),
            output_bytes: BytesMut::default(),
            db: self.db.clone(),
            authed: None,
            organisation: None,
            is_git_protocol_v2: false,
        }
    }
}

struct Handler {
    ip: Option<std::net::SocketAddr>,
    span: tracing::Span,
    codec: GitCodec,
    input_bytes: BytesMut,
    output_bytes: BytesMut,
    db: chartered_db::ConnectionPool,
    organisation: Option<String>,
    authed: Option<Authed>,
    is_git_protocol_v2: bool,
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

    fn env_request(
        mut self,
        _channel: ChannelId,
        name: &str,
        value: &str,
        session: Session,
    ) -> Self::FutureUnit {
        self.span.in_scope(|| debug!("env set {}={}", name, value));

        #[allow(clippy::single_match)]
        match (name, value) {
            ("GIT_PROTOCOL", "version=2") => self.is_git_protocol_v2 = true,
            _ => {}
        }

        Box::pin(futures::future::ready(Ok((self, session))))
    }

    fn shell_request(mut self, channel: ChannelId, mut session: Session) -> Self::FutureUnit {
        let span = self.span.clone();

        Box::pin(async move {
            error!("Client attempted to open a shell, closing connection");

            let username = self.authed()?.user.username.clone(); // todo
            write!(&mut self.output_bytes, "Hi there, {}! You've successfully authenticated, but chartered does not provide shell access.\r\n", username)?;
            self.flush(&mut session, channel);
            session.close(channel);
            Ok((self, session))
        }.instrument(tracing::info_span!(parent: span, "shell request")))
    }

    fn exec_request(
        mut self,
        channel: ChannelId,
        data: &[u8],
        mut session: Session,
    ) -> Self::FutureUnit {
        let span = self.span.clone();

        let data = match std::str::from_utf8(data) {
            Ok(data) => data,
            Err(e) => return Box::pin(futures::future::err(e.into())),
        };
        let args = shlex::split(data);

        Box::pin(async move {
            debug!("exec {:?}", args);

            if !self.is_git_protocol_v2 {
                anyhow::bail!("not git protocol v2");
            }

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

            self.write(PktLine::Data(b"version 2\n"))?;
            self.write(PktLine::Data(b"agent=chartered/0.1.0\n"))?; // TODO: clap::crate_name!()/clap::crate_version!()
            self.write(PktLine::Data(b"ls-refs=unborn\n"))?;
            self.write(PktLine::Data(b"fetch=shallow wait-for-done\n"))?;
            self.write(PktLine::Data(b"server-option\n"))?;
            self.write(PktLine::Data(b"object-info\n"))?;
            self.write(PktLine::Flush)?;
            self.flush(&mut session, channel);

            Ok((self, session))
        }.instrument(tracing::info_span!(parent: span, "exec")))
    }

    fn subsystem_request(
        self,
        _channel: ChannelId,
        _data: &str,
        session: Session,
    ) -> Self::FutureUnit {
        Box::pin(futures::future::ready(Ok((self, session))))
    }

    fn auth_publickey(mut self, _username: &str, key: &key::PublicKey) -> Self::FutureAuth {
        let span = self.span.clone();
        let public_key = key.public_key_bytes();

        Box::pin(
            async move {
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
            }
            .instrument(tracing::info_span!(parent: span, "auth pubkey")),
        )
    }

    fn auth_keyboard_interactive(
        self,
        _user: &str,
        _submethods: &str,
        _response: Option<server::Response<'_>>,
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
        let span = self.span.clone();
        self.input_bytes.extend_from_slice(data);

        Box::pin(
            async move {
                while let Some(frame) = self.codec.decode(&mut self.input_bytes)? {
                    debug!(
                        "decoded frame command={:?} metadata={:?}",
                        frame.command, frame.metadata
                    );

                    // if the client flushed without giving us a command, we're expected to close
                    // the connection or else the client will just hang
                    if frame.command.is_empty() {
                        session.exit_status_request(channel, 0);
                        session.eof(channel);
                        session.close(channel);
                        return Ok((self, session));
                    }

                    let authed = self.authed()?;
                    let org_name = self.org_name()?;

                    let mut packfile = GitRepository::default();
                    let config = CargoConfig::new(
                        &Url::parse("http://127.0.0.1:8888/")?,
                        &authed.auth_key,
                        org_name,
                    );
                    let config = serde_json::to_vec(&config)?;
                    packfile.insert(ArrayVec::<_, 0>::new(), "config.json", &config)?;
                    // todo: the whole tree needs caching and then we can filter in code rather than at
                    //  the database
                    let tree =
                        Tree::build(self.db.clone(), authed.user.id, org_name.to_string()).await;
                    tree.write_to_packfile(&mut packfile)?;

                    let (commit_hash, packfile_entries) =
                        packfile.commit("computer", "john@computer.no", "Update crates")?;

                    match frame.command.as_ref() {
                        b"command=ls-refs" => {
                            command_handlers::ls_refs::handle(
                                &mut self,
                                &mut session,
                                channel,
                                frame.metadata,
                                &commit_hash,
                            )?;
                        }
                        b"command=fetch" => {
                            command_handlers::fetch::handle(
                                &mut self,
                                &mut session,
                                channel,
                                frame.metadata,
                                packfile_entries,
                            )?;
                        }
                        v => {
                            error!(
                                "Client sent unknown command, ignoring command {}",
                                std::str::from_utf8(v).unwrap_or("invalid utf8")
                            );
                        }
                    }
                }

                Ok((self, session))
            }
            .instrument(tracing::info_span!(parent: span, "data")),
        )
    }
}
