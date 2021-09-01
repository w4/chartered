#![deny(clippy::pedantic)]
#[allow(clippy::missing_errors_doc)]
pub mod git;

use crate::git::{
    codec::{Encoder, GitCodec},
    packfile::{Commit, CommitUserInfo, PackFileEntry, TreeItem, TreeItemKind},
    PktLine,
};

use bytes::BytesMut;
use futures::future::Future;
use std::{fmt::Write, pin::Pin, sync::Arc};
use thrussh::{
    server::{self, Auth, Session},
    ChannelId, CryptoVec,
};
use thrussh_keys::key;
use tokio_util::codec::{Decoder, Encoder as TokioEncoder};

#[tokio::main]
#[allow(clippy::semicolon_if_nothing_returned)] // broken clippy lint
async fn main() {
    env_logger::init();

    let mut config = thrussh::server::Config::default();
    config.keys.push(key::KeyPair::generate_ed25519().unwrap());
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
        );
    }
}

type AsyncHandlerFn = Pin<
    Box<
        dyn Future<Output = Result<(Handler, Session), <Handler as server::Handler>::Error>> + Send,
    >,
>;

impl server::Handler for Handler {
    type Error = anyhow::Error;
    type FutureAuth = futures::future::Ready<Result<(Self, server::Auth), anyhow::Error>>;
    type FutureUnit = AsyncHandlerFn;
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

            let test_crate_file = PackFileEntry::Blob(br#"{"name":"charteredtest","vers":"1.0.0","deps":[],"cksum":"7b821735f0211fd00032a9892d1bf2323c9d05d9c59b9303eb382f5ec1898bfc","features":{},"yanked":false,"links":null}"#);
            let config_file = PackFileEntry::Blob(
                br#"{
                "dl": "http://127.0.0.1:8888/a/abc/api/v1/crates",
                "api": "http://127.0.0.1:8888/a/abc"
            }"#,
            );

            let ch_ar_tree = PackFileEntry::Tree(vec![TreeItem {
                kind: TreeItemKind::File,
                name: "charteredtest",
                hash: test_crate_file.hash()?,
            }]);

            let ch_tree = PackFileEntry::Tree(vec![TreeItem {
                kind: TreeItemKind::Directory,
                name: "ar",
                hash: ch_ar_tree.hash()?,
            }]);

            let root_tree = PackFileEntry::Tree(vec![
                TreeItem {
                    kind: TreeItemKind::Directory,
                    name: "ch",
                    hash: ch_tree.hash()?,
                },
                TreeItem {
                    kind: TreeItemKind::File,
                    name: "config.json",
                    hash: config_file.hash()?,
                },
            ]);

            let commit_user = CommitUserInfo {
                name: "Jordan Doyle",
                email: "jordan@doyle.la",
                time: chrono::Utc::now(),
            };

            let commit = PackFileEntry::Commit(Commit {
                tree: root_tree.hash()?,
                author: commit_user,
                committer: commit_user,
                message: "cool commit",
            });

            // echo -ne "0014command=ls-refs\n0014agent=git/2.321\n00010009peel\n000csymrefs\n000bunborn\n0014ref-prefix HEAD\n0019ref-prefix refs/HEAD\n001eref-prefix refs/tags/HEAD\n001fref-prefix refs/heads/HEAD\n0021ref-prefix refs/remotes/HEAD\n0026ref-prefix refs/remotes/HEAD/HEAD\n001aref-prefix refs/tags/\n0000"
            // GIT_PROTOCOL=version=2 ssh -o SendEnv=GIT_PROTOCOL git@github.com git-upload-pack '/w4/chartered.git'
            // ''.join([('{:04x}'.format(len(v) + 5)), v, "\n"])
            // echo -ne "0012command=fetch\n0001000ethin-pack\n0010no-progress\n0010include-tag\n000eofs-delta\n0032want f6046cf6372e0d8ab845f6dec1602c303a66ee91\n"
            // sends a 000dpackfile back
            // https://shafiul.github.io/gitbook/7_the_packfile.html
            if ls_refs {
                let commit_hash = hex::encode(&commit.hash()?);
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

                let packfile = git::packfile::PackFile::new(vec![
                    commit,
                    test_crate_file,
                    ch_tree,
                    ch_ar_tree,
                    config_file,
                    root_tree,
                ]);
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
