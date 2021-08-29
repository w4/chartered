pub mod git;

use crate::git::PktLine;

use crate::git::codec::GitCodec;
use bytes::BytesMut;
use futures::future::Future;
use git::codec::Encoder;
use std::{fmt::Write, pin::Pin, sync::Arc};
use thrussh::server::{Auth, Session};
use thrussh::*;
use thrussh_keys::*;
use tokio_util::codec::{Decoder, Encoder as TokioEncoder};

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut config = thrussh::server::Config::default();
    config
        .keys
        .push(thrussh_keys::key::KeyPair::generate_ed25519().unwrap());
    let config = Arc::new(config);
    thrussh::server::run(config, "127.0.0.1:2233", Server)
        .await
        .unwrap();
}

#[derive(Clone)]
struct Server;

impl server::Server for Server {
    type Handler = Handler;

    fn new(&mut self, _: Option<std::net::SocketAddr>) -> Self::Handler {
        Handler::default()
    }
}

#[derive(Default)]
struct Handler {
    codec: GitCodec,
    input_bytes: BytesMut,
    output_bytes: BytesMut,
}

impl Handler {
    fn write(&mut self, packet: PktLine<'_>) -> Result<(), anyhow::Error> {
        Encoder {}.encode(packet, &mut self.output_bytes)
    }

    fn flush(&mut self, session: &mut Session, channel: ChannelId) {
        session.data(
            channel,
            CryptoVec::from_slice(self.output_bytes.split().as_ref()),
        )
    }
}

impl server::Handler for Handler {
    type Error = anyhow::Error;
    type FutureAuth = futures::future::Ready<Result<(Self, server::Auth), anyhow::Error>>;
    type FutureUnit = Pin<Box<dyn Future<Output = Result<(Self, Session), Self::Error>> + Send>>;
    type FutureBool = futures::future::Ready<Result<(Self, Session, bool), anyhow::Error>>;

    fn finished_auth(self, auth: Auth) -> Self::FutureAuth {
        eprintln!("finished auth");
        futures::future::ready(Ok((self, auth)))
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
            write!(&mut self.output_bytes, "Hi there! You've successfully authenticated, but chartered does not provide shell access.\r\n")?;
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

    fn auth_publickey(self, _: &str, _: &key::PublicKey) -> Self::FutureAuth {
        eprintln!("finished auth pubkey");
        self.finished_auth(server::Auth::Accept)
    }

    fn data(mut self, channel: ChannelId, data: &[u8], mut session: Session) -> Self::FutureUnit {
        self.input_bytes.extend_from_slice(data);

        Box::pin(async move {
            let mut ls_refs = false;

            while let Some(frame) = self.codec.decode(&mut self.input_bytes)? {
                eprintln!("data: {:x?}", frame);

                if frame.as_ref() == "command=ls-refs".as_bytes() {
                    ls_refs = true;
                }
            }

            // echo -ne "0014command=ls-refs\n0014agent=git/2.321\n00010008peel000bsymrefs000aunborn0014ref-prefix HEAD\n0000"
            // GIT_PROTOCOL=version=2 ssh -o SendEnv=GIT_PROTOCOL git@github.com git-upload-pack '/w4/chartered.git'
            // ''.join([('{:04x}'.format(len(v) + 5)), v, "\n"])
            // echo -ne "0012command=fetch\n0001000ethin-pack\n0010no-progress\n0010include-tag\n000eofs-delta\n0032want 1a1b25ae7c87a0e87b7a9aa478a6bc4331c6b954\n0009done\n"
            // sends a 000dpackfile back
            // https://shafiul.github.io/gitbook/7_the_packfile.html
            if ls_refs {
                self.write(PktLine::Data(b"1a1b25ae7c87a0e87b7a9aa478a6bc4331c6b954 HEAD symref-target:refs/heads/master\n"))?;
                self.write(PktLine::Flush)?;
                self.flush(&mut session, channel);

                // next command will be a fetch like above
            }

            Ok((self, session))
        })
    }
}
