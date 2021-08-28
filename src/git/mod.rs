pub mod codec;
pub mod packfile;

use thrussh::CryptoVec;

pub enum PktLine<'a> {
    Data(&'a [u8]),
    Flush,
    Delimiter,
    ResponseEnd,
}

impl PktLine<'_> {
    // todo: encode to connection's `bytes::BytesMut`
    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::new();

        match self {
            Self::Data(data) => {
                v.extend_from_slice(format!("{:04x}", data.len() + 4).as_ref());
                v.extend_from_slice(data);    
            },
            Self::Flush => v.extend_from_slice(b"0000"),
            Self::Delimiter => v.extend_from_slice(b"0001"),
            Self::ResponseEnd => v.extend_from_slice(b"0002"),
        }

        v
    }
}

impl From<PktLine<'_>> for CryptoVec {
    fn from(val: PktLine<'_>) -> Self {
        Self::from(val.encode())
    }
}

impl<'a> From<&'a str> for PktLine<'a> {
    fn from(val: &'a str) -> Self {
        PktLine::Data(val.as_bytes())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_pkt_line() {
        let encoded = super::PktLine(b"agent=git/2.32.0\n").encode();
        assert_eq!(encoded, b"0015agent=git/2.32.0\n");
    }
}
