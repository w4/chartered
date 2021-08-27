use thrussh::CryptoVec;

pub const END_OF_MESSAGE: &'static [u8] = b"0000";

pub struct PktLine<'a>(pub &'a [u8]);

impl PktLine<'_> {
    // todo: encode to connection's `bytes::BytesMut`
    pub fn encode(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(format!("{:04x}", self.0.len() + 4).as_ref());
        v.extend_from_slice(self.0);
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
        PktLine(val.as_bytes())
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
