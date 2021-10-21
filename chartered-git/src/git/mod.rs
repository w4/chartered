pub mod codec;
pub mod packfile;

use bytes::{BufMut, BytesMut};
use std::fmt::Write;

use self::packfile::low_level::PackFile;

/// Every packet sent to the client from us should be a `PktLine`.
pub enum PktLine<'a> {
    Data(&'a [u8]),
    /// Similar to a data packet, but used during packfile sending to indicate this
    /// packet is a block of data by appending a byte containing the u8 `1`.
    SidebandData(PackFile<'a>),
    /// Similar to a data packet, but used during packfile sending to indicate this
    /// packet is a status message by appending a byte containing the u8 `2`.
    SidebandMsg(&'a [u8]),
    Flush,
    Delimiter,
    ResponseEnd,
}

impl PktLine<'_> {
    pub fn encode_to(&self, buf: &mut BytesMut) -> Result<(), anyhow::Error> {
        match self {
            Self::Data(data) => {
                write!(buf, "{:04x}", data.len() + 4)?;
                buf.extend_from_slice(data);
            }
            Self::SidebandData(packfile) => {
                // split the buf off so the cost of counting the bytes to put in the
                // data line prefix is just the cost of `unsplit` (an atomic decrement)
                let mut data_buf = buf.split_off(buf.len());

                data_buf.put_u8(1); // sideband, 1 = data
                packfile.encode_to(&mut data_buf)?;

                // write into the buf not the data buf so it's at the start of the msg
                write!(buf, "{:04x}", data_buf.len() + 4)?;
                buf.unsplit(data_buf);
            }
            Self::SidebandMsg(msg) => {
                write!(buf, "{:04x}", msg.len() + 4 + 1)?;
                buf.put_u8(2); // sideband, 2 = msg
                buf.extend_from_slice(msg);
            }
            Self::Flush => buf.extend_from_slice(b"0000"),
            Self::Delimiter => buf.extend_from_slice(b"0001"),
            Self::ResponseEnd => buf.extend_from_slice(b"0002"),
        }

        Ok(())
    }
}

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
