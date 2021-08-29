use bytes::{BufMut, BytesMut};
use flate2::{write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::fmt::Write;
use std::io::Write as IoWrite;

// The offset/sha1[] tables are sorted by sha1[] values (this is to
// allow binary search of this table), and fanout[] table points at
// the offset/sha1[] table in a specific way (so that part of the
// latter table that covers all hashes that start with a given byte
// can be found to avoid 8 iterations of the binary search).
//
// packfile indexes are not neccesary to extract objects from a packfile
pub struct PackFileIndex<const S: usize> {
    pub fanout: [[u8; 4]; 255],
    pub size: u16,            // fanout[256] => size == S
    pub sha1: [[u8; 20]; S],  // sha listing
    pub crc: [[u8; 4]; S],    // checksum
    pub offset: [[u8; 4]; S], // packfile offsets
    // 64b_offset: [[u8; 8]; N], // for packfiles over 2gb
    pub packfile_checksum: [u8; 20], // sha1
    pub idxfiel_checksum: [u8; 20],  // sha1
}

impl<const S: usize> PackFileIndex<S> {
    pub fn encode_to(self, buf: &mut BytesMut) -> Result<(), anyhow::Error> {
        buf.extend_from_slice(b"\xFFtOc"); // magic header
        buf.put_u8(2); // version


        Ok(())
    }
}

// The packfile itself is a very simple format. There is a header, a
// series of packed objects (each with it's own header and body) and
// then a checksum trailer. The first four bytes is the string 'PACK',
// which is sort of used to make sure you're getting the start of the
// packfile correctly. This is followed by a 4-byte packfile version
// number and then a 4-byte number of entries in that file.
pub struct PackFile {
    pub entries: Vec<PackFileEntry>,
}

impl PackFile {
    pub fn encode_to(self, buf: &mut BytesMut) -> Result<(), anyhow::Error> {
        buf.extend_from_slice(b"PACK"); // magic header
        buf.extend_from_slice(b"0002"); // version
        write!(buf, "{:04x}", self.entries.len())?; // number of entries in the packfile

        for entry in &self.entries {
            entry.encode_to(buf)?;
        }

        let mut hasher = Sha1::new();
        for entry in &self.entries {
            hasher.update(entry.sha1);
        }
        let hash = hasher.finalize();
        buf.extend_from_slice(&hash.as_slice());

        Ok(())
    }
}

pub enum PackFileEntryType {
    // jordan@Jordans-MacBook-Pro-2 0d % printf "\x1f\x8b\x08\x00\x00\x00\x00\x00" | cat - f5/473259d9674ed66239766a013f96a3550374e3-test | gzip -dc
    // commit 1068tree 0d586b48bc42e8591773d3d8a7223551c39d453c
    // parent c2a862612a14346ae95234f26efae1ee69b5b7a9
    // author Jordan Doyle <jordan@doyle.la> 1630244577 +0100
    // committer Jordan Doyle <jordan@doyle.la> 1630244577 +0100
    // gpgsig -----BEGIN PGP SIGNATURE-----
    // 
    // iQIzBAABCAAdFiEEMn1zof7yzaURQBGDHqa65vZtxJoFAmErjuEACgkQHqa65vZt
    // xJqhvhAAieKXnGRjT926qzozcvarC8D3TlA+Z1wVXueTAWqfusNIP0zCun/crOb2
    // tOULO+/DXVBmwu5eInAf+t/wvlnIsrzJonhVr1ZT0f0vDX6fs2vflWg4UCVEuTsZ
    // tg+aTjcibwnmViIM9XVOzhU8Au2OIqMQLyQOMWSt8NhY0W2WhBCdQvhktvK1V8W6
    // omPs04SrR39xWBDQaxsXYxq/1ZKUYXDwudvEfv14EvrxG1vWumpUVJd7Ib5w4gXX
    // fYa95DxYL720ZaiWPIYEG8FMBzSOpo6lUzY9g2/o/wKwSQZJNvpaMGCuouy8Fb+E
    // UaqC0XPxqpKG9duXPgCldUr+P7++48CF5zc358RBGz5OCNeTREsIQQo5PUO1k+wO
    // FnGOQTT8vvNOrxBgb3QgKu67RVwWDc6JnQCNpUrhUJrXMDWnYLBqo4Y+CdKGSQ4G
    // hW8V/hVTOlJZNi8bbU4v53cxh4nXiMM6NKUblUKs65ar3/2dkojwunz7r7GVZ6mG
    // QUpr9+ybG61XDqd1ad1A/B/i3WdWixTmJS3K/4uXjFjFX1f3RAk7O0gHc9I8HYOE
    // Vd8UsHzLOWAUHeaqbsd6xx3GCXF4D5D++kh9OY9Ov7CXlqbYbHd6Atg+PQ7VnqNf
    // bDqWN0Q2qcKX3k4ggtucmkkA6gP+K3+F5ANQj3AsGMQeddowC0Y=
    // =fXoH
    // -----END PGP SIGNATURE-----
    //  
    // test
    Commit,
    // jordan@Jordans-MacBook-Pro-2 0d % printf "\x1f\x8b\x08\x00\x00\x00\x00\x00" | cat - 0d/586b48bc42e8591773d3d8a7223551c39d453c | gzip -dc
    // tree 20940000 .cargo���CYy��Ve�������100644 .gitignore�K��_ow�]����4�n�ݺ100644 Cargo.lock�7�3-�?/��
    // kt��c0C�100644 Cargo.toml�6�&(��]\8@�SHA�]f40000 src0QW��ƅ���b[�!�S&N�100644 test�G2Y�gN�b9vj?��Ut�
    Tree,
    // jordan@Jordans-MacBook-Pro-2 objects % printf "\x1f\x8b\x08\x00\x00\x00\x00\x00" | cat - f5/473259d9674ed66239766a013f96a3550374e3| gzip -dc
    // blob 23try and find me in .git
    Blob,
    // Tag,
    // OfsDelta,
    // RefDelta,
}

pub struct PackFileEntry {
    pub entry_type: PackFileEntryType,
    pub data: Vec<u8>,
    pub sha1: [u8; 20],
}

impl PackFileEntry {
    fn size_of_data(&self) -> usize {
        self.data.len() as usize
    }

    // The object header is a series of one or more 1 byte (8 bit) hunks
    // that specify the type of object the following data is, and the size
    // of the data when expanded. Each byte is really 7 bits of data, with
    // the first bit being used to say if that hunk is the last one or not
    // before the data starts. If the first bit is a 1, you will read another
    // byte, otherwise the data starts next. The first 3 bits in the first
    // byte specifies the type of data, according to the table below.
    fn write_header(&self, buf: &mut BytesMut) {
        let mut size = self.size_of_data();

        // write header
        {
            let mut val = 0b10000000u8;

            val |= match self.entry_type {
                PackFileEntryType::Commit => 0b001,
                PackFileEntryType::Tree => 0b010,
                PackFileEntryType::Blob => 0b011,
                // PackFileEntryType::Tag => 0b100,
                // PackFileEntryType::OfsDelta => 0b110,
                // PackFileEntryType::RefDelta => 0b111,
            } << 4;

            // pack the last 4 bits of the size into the header
            val |= (size & 0b1111) as u8;
            size >>= 4;

            buf.put_u8(val);
        }

        // write size bytes
        while size != 0 {
            // read 7 bits from the `size` and push them off for the next iteration
            let mut val = (size & 0b1111111) as u8;
            size >>= 7;

            if size != 0 {
                // first bit implies there's more size bytes to come, otherwise the
                // data starts after this byte
                val |= 1 << 7;
            }

            buf.put_u8(val);
        }
    }

    pub fn encode_to(&self, buf: &mut BytesMut) -> Result<(), anyhow::Error> {
        self.write_header(buf);

        let mut e = ZlibEncoder::new(buf.as_mut(), Compression::default());
        e.write_all(self.data.as_ref())?;
        e.finish()?;

        Ok(())
    }
}
