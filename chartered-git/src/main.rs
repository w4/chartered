#![deny(clippy::pedantic)]
#![deny(rust_2018_idioms)]
mod command_handlers;
mod config;
mod generators;
mod tree;

use crate::{generators::CargoConfig, tree::Tree};

use bytes::BytesMut;
use chartered_db::server_private_key::ServerPrivateKey;
use clap::Parser;
use futures::future::Future;
use packfile::{
    codec::{Encoder, GitCodec},
    high_level::GitRepository,
    PktLine,
};
use std::{fmt::Write, path::PathBuf, pin::Pin, sync::Arc};
use thrussh::{
    server::{self, Auth, Session},
    ChannelId, CryptoVec,
};
use thrussh_keys::{key, PublicKeyBase64};
use tokio_util::codec::{Decoder, Encoder as TokioEncoder};
use tracing::{debug, error, info, warn, Instrument};

const AGENT: &str = concat!(
    "agent=",
    clap::crate_name!(),
    "/",
    clap::crate_version!(),
    "\n"
);

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

    let db = chartered_db::init(&config.database_uri)?;

    ServerPrivateKey::create_if_not_exists(db.clone()).await?;
    let keys = ServerPrivateKey::fetch_all(db.clone()).await?;

    let trussh_config = Arc::new(thrussh::server::Config {
        methods: thrussh::MethodSet::PUBLICKEY,
        keys: keys
            .into_iter()
            .map(ServerPrivateKey::into_private_key)
            .collect::<Result<_, _>>()?,
        ..thrussh::server::Config::default()
    });

    let bind_address = config.bind_address;

    let server = Server {
        db,
        config: Box::leak(Box::new(config)),
    };

    info!("SSH server listening on {}", bind_address);

    thrussh::server::run(trussh_config, &bind_address.to_string(), server).await?;

    Ok(())
}

#[derive(Clone)]
struct Server {
    db: chartered_db::ConnectionPool,
    config: &'static config::Config,
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
            config: self.config,
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
    config: &'static config::Config,
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
        Ok(Encoder.encode(packet, &mut self.output_bytes)?)
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

    fn auth_none(self, _user: &str) -> Self::FutureAuth {
        self.finished_auth(server::Auth::UnsupportedMethod)
    }

    fn auth_password(self, _user: &str, _password: &str) -> Self::FutureAuth {
        self.finished_auth(server::Auth::UnsupportedMethod)
    }

    /// User is attempting to connect via pubkey, we'll lookup the key in the
    /// user database and create a new session if it exists, otherwise we'll
    /// reject the authentication attempt.
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

                    // start building the packfile we're going to send to the user
                    let mut packfile = GitRepository::default();

                    // write the config.json to the root of the repository
                    let config =
                        CargoConfig::new(&self.config.web_base_uri, &authed.auth_key, org_name);
                    let config = serde_json::to_vec(&config)?;
                    packfile.insert(&[], "config.json", config.into())?;

                    // build the tree of all the crates the user has access to, then write them
                    // to the in-memory repository.
                    // todo: the whole tree needs caching and then we can filter in code rather than at
                    //  the database
                    let tree =
                        Tree::build(self.db.clone(), authed.user.id, org_name.to_string()).await;
                    tree.write_to_packfile(&mut packfile)?;

                    // finalises the git repository, creating a commit and fetching the finalised
                    // packfile and commit hash to return in `ls-refs` calls.
                    let (commit_hash, packfile_entries) = packfile.commit(
                        &self.config.committer.name,
                        &self.config.committer.email,
                        &self.config.committer.message,
                    )?;

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

    /// Initially when setting up the SSH connection, the remote Git client will send us an
    /// exec request (instead of the usual shell request that is sent when invoking `ssh`).
    ///
    /// The client will set `git-upload-pack` as the requested executable to run and also
    /// sends the path that was appended to the end of the connection string defined in
    /// cargo.
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
        // parses the given args in the same fashion as a POSIX shell
        let args = shlex::split(data);

        Box::pin(async move {
            debug!("exec {:?}", args);

            // if the client didn't send `GIT_PROTOCOL=version=2` as an environment
            // variable when connecting, we'll just close the connection
            if !self.is_git_protocol_v2 {
                anyhow::bail!("not git protocol v2");
            }

            let mut args = args.into_iter().flat_map(Vec::into_iter);

            // check the executable requested to be ran is the `git-upload-pack` we
            // expect. we're not actually going to execute this, but we'll pretend
            // to be it instead in `data`.
            if args.next().as_deref() != Some("git-upload-pack") {
                anyhow::bail!("not git-upload-pack");
            }

            // parse the requested organisation from the given path (the argument
            // given to `git-upload-pack`)
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

            // preamble, sending our capabilities and what have you
            self.write(PktLine::Data(b"version 2\n"))?;
            self.write(PktLine::Data(AGENT.as_bytes()))?;
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
}
