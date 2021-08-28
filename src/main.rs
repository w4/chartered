pub mod git;

use crate::git::PktLine;


use std::sync::{Arc};
use thrussh::server::{Auth, Session};
use thrussh::*;
use thrussh_keys::*;
use bytes::BytesMut;
use crate::git::codec::GitCodec;
use tokio_util::codec::Decoder;

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
}

impl server::Handler for Handler {
    type Error = anyhow::Error;
    type FutureAuth = futures::future::Ready<Result<(Self, server::Auth), anyhow::Error>>;
    type FutureUnit = futures::future::Ready<Result<(Self, Session), anyhow::Error>>;
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
        futures::future::ready(Ok((self, s)))
    }

    fn shell_request(self, channel: ChannelId, mut session: Session) -> Self::FutureUnit {
        session.data(channel, CryptoVec::from_slice(b"Hi there! You've successfully authenticated, but chartered does not provide shell access.\r\n"));
        session.close(channel);
        futures::future::ready(Ok((self, session)))
    }

    fn exec_request(
        self,
        channel: ChannelId,
        data: &[u8],
        mut session: Session,
    ) -> Self::FutureUnit {
        eprintln!("exec {:x?}", data);

        if !data.starts_with(b"git-upload-pack ") {
            session.data(
                channel,
                CryptoVec::from_slice(b"Sorry, I have no clue who you are\r\n"),
            );
            session.close(channel);
        } else {
            // TODO: check GIT_PROTOCOL=version=2 set
            session.data(channel, PktLine::Data(b"version 2\n").into());
            session.data(channel, PktLine::Data(b"agent=chartered/0.1.0\n").into());
            session.data(channel, PktLine::Data(b"ls-refs=unborn\n").into());
            session.data(channel, PktLine::Data(b"fetch=shallow wait-for-done\n").into());
            session.data(channel, PktLine::Data(b"server-option\n").into());
            session.data(channel, PktLine::Data(b"object-info\n").into());
            session.data(channel, PktLine::Flush.into());
        }

        futures::future::ready(Ok((self, session)))
    }

    fn subsystem_request(
        self,
        _channel: ChannelId,
        data: &str,
        session: Session,
    ) -> Self::FutureUnit {
        eprintln!("subsystem req: {}", data);
        futures::future::ready(Ok((self, session)))
    }

    fn auth_publickey(self, _: &str, _: &key::PublicKey) -> Self::FutureAuth {
        eprintln!("finished auth pubkey");
        self.finished_auth(server::Auth::Accept)
    }

    fn data(mut self, channel: ChannelId, data: &[u8], mut session: Session) -> Self::FutureUnit {
        self.input_bytes.extend_from_slice(data);

        let mut ls_refs = false;

        while let Some(frame) = self.codec.decode(&mut self.input_bytes).unwrap() {
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
            session.data(channel, PktLine::Data(b"1a1b25ae7c87a0e87b7a9aa478a6bc4331c6b954 HEAD symref-target:refs/heads/master\n").into());
            session.data(channel, PktLine::Flush.into());

            // next command will be a fetch like above
        }

        futures::future::ready(Ok((self, session)))
    }

    fn extended_data(
        self,
        _channel: ChannelId,
        code: u32,
        data: &[u8],
        session: Session,
    ) -> Self::FutureUnit {
        eprintln!("got extended data: {:x?} ({})", data, code);
        futures::future::ready(Ok((self, session)))
    }

    fn signal(self, _channel: ChannelId, signal_name: Sig, session: Session) -> Self::FutureUnit {
        eprintln!("signal: {:#?}", signal_name);
        futures::future::ready(Ok((self, session)))
    }

    fn window_change_request(
        self,
        _channel: ChannelId,
        _col_width: u32,
        _row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        session: Session,
    ) -> Self::FutureUnit {
        eprintln!("window change req");
        futures::future::ready(Ok((self, session)))
    }

    fn env_request(
        self,
        _channel: ChannelId,
        variable_name: &str,
        variable_value: &str,
        session: Session,
    ) -> Self::FutureUnit {
        eprintln!("set env {} = {}", variable_name, variable_value);
        futures::future::ready(Ok((self, session)))
    }

    fn x11_request(
        self,
        _channel: ChannelId,
        _single_connection: bool,
        _x11_auth_protocol: &str,
        _x11_auth_cookie: &str,
        _x11_screen_number: u32,
        session: Session,
    ) -> Self::FutureUnit {
        eprintln!("x11 req");
        futures::future::ready(Ok((self, session)))
    }

    fn pty_request(
        self,
        _channel: ChannelId,
        _term: &str,
        _col_width: u32,
        _row_height: u32,
        _pix_width: u32,
        _pix_height: u32,
        _modes: &[(Pty, u32)],
        session: Session,
    ) -> Self::FutureUnit {
        eprintln!("pty req");
        futures::future::ready(Ok((self, session)))
    }

    fn channel_open_direct_tcpip(
        self,
        _channel: ChannelId,
        _host_to_connect: &str,
        _port_to_connect: u32,
        _originator_address: &str,
        _originator_port: u32,
        session: Session,
    ) -> Self::FutureUnit {
        eprintln!("direct tcpip");
        futures::future::ready(Ok((self, session)))
    }

    fn channel_eof(self, _channel: ChannelId, session: Session) -> Self::FutureUnit {
        eprintln!("eof");
        futures::future::ready(Ok((self, session)))
    }
}
