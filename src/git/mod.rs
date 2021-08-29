pub mod codec;
pub mod packfile;

use bytes::BytesMut;
use std::fmt::Write;

pub enum PktLine<'a> {
    Data(&'a [u8]),
    Flush,
    Delimiter,
    ResponseEnd,
}

impl PktLine<'_> {
    pub fn encode_to(&self, buf: &mut BytesMut) -> Result<(), anyhow::Error> {
        match self {
            Self::Data(data) => {
                write!(buf, "{:04x}", data.len() + 4)?;
                buf.extend_from_slice(&data);
            }
            Self::Flush => buf.extend_from_slice(b"0000"),
            Self::Delimiter => buf.extend_from_slice(b"0001"),
            Self::ResponseEnd => buf.extend_from_slice(b"0002"),
        }

        Ok(())
    }
}

// impl From<PktLine<'_>> for CryptoVec {
//     fn from(val: PktLine<'_>) -> Self {
//         Self::from(val.encode())
//     }
// }

impl<'a> From<&'a str> for PktLine<'a> {
    fn from(val: &'a str) -> Self {
        PktLine::Data(val.as_bytes())
    }
}

#[cfg(test)]
mod test {
    use bytes::BytesMut;

    #[test]
    fn test_pkt_line() {
        let mut buffer = BytesMut::new();
        super::PktLine::Data(b"agent=git/2.32.0\n")
            .encode_to(&mut buffer)
            .unwrap();
        assert_eq!(buffer.as_ref(), b"0015agent=git/2.32.0\n");
    }
}
