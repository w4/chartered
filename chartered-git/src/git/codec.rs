#![allow(clippy::module_name_repetitions)]

use bytes::{Buf, Bytes, BytesMut};
use tokio_util::codec;

use super::PktLine;

pub struct Encoder {
    // buf: BytesMut,
}

impl codec::Encoder<PktLine<'_>> for Encoder {
    type Error = anyhow::Error;

    fn encode(&mut self, item: PktLine<'_>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.encode_to(dst)?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct GitCommand {
    pub command: Bytes,
    pub metadata: Vec<Bytes>,
}

#[derive(Default)]
pub struct GitCodec {
    command: GitCommand,
}

impl codec::Decoder for GitCodec {
    type Item = GitCommand;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            if src.len() < 4 {
                return Ok(None);
            }

            let mut length_bytes = [0_u8; 4];
            length_bytes.copy_from_slice(&src[..4]);
            let length = u16::from_str_radix(std::str::from_utf8(&length_bytes)?, 16)? as usize;

            if length == 0 {
                // flush
                src.advance(4);
                return Ok(Some(std::mem::take(&mut self.command)));
            } else if length == 1 || length == 2 {
                src.advance(4);
                eprintln!("magic packet = {}", length);
                continue;
            } else if !(4..=65520).contains(&length) {
                eprintln!("protocol abuse");
                return Err(
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "protocol abuse").into(),
                );
            }

            // not enough bytes in the buffer yet, ask for more
            if src.len() < length {
                src.reserve(length - src.len());
                return Ok(None);
            }

            // length is inclusive of the 4 bytes that makes up itself
            let mut data = src.split_to(length).freeze();
            data.advance(4);

            // strip newlines for conformity
            if data.ends_with(b"\n") {
                data.truncate(data.len() - 1);
            }

            if self.command.command.is_empty() {
                self.command.command = data;
            } else {
                self.command.metadata.push(data);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use bytes::{Bytes, BytesMut};
    use std::fmt::Write;
    use tokio_util::codec::Decoder;

    #[test]
    fn decode() {
        let mut codec = super::GitCodec::default();

        let mut bytes = BytesMut::new();

        bytes.write_str("0015agent=git/2.32.0").unwrap();
        let res = codec.decode(&mut bytes).unwrap();
        assert_eq!(res, None);

        bytes.write_char('\n').unwrap();
        let res = codec.decode(&mut bytes).unwrap();
        assert_eq!(res, None);

        bytes.write_str("0000").unwrap();
        let res = codec.decode(&mut bytes).unwrap();
        assert_eq!(
            res,
            Some(super::GitCommand {
                command: Bytes::from_static(b"agent=git/2.32.0\n"),
                metadata: vec![],
            })
        );

        bytes.write_str("0000").unwrap();
        let res = codec.decode(&mut bytes).unwrap();
        assert_eq!(
            res,
            Some(super::GitCommand {
                command: Bytes::new(),
                metadata: vec![],
            })
        );

        bytes.write_str("0002").unwrap();
        bytes.write_str("0005a").unwrap();
        bytes.write_str("0001").unwrap();
        bytes.write_str("0005b").unwrap();
        bytes.write_str("0000").unwrap();

        let res = codec.decode(&mut bytes).unwrap();
        assert_eq!(
            res,
            Some(super::GitCommand {
                command: Bytes::from_static(b"a"),
                metadata: vec![Bytes::from_static(b"b")],
            })
        );
    }
}
