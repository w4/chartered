use bytes::{BufMut, BytesMut};
use const_sha1::{sha1, ConstBuffer};
use flate2::{write::ZlibEncoder, Compression};
use std::convert::TryInto;
use std::io::Write as IoWrite;
use sha1::{Sha1, Digest};

// The offset/sha1[] tables are sorted by sha1[] values (this is to
// allow binary search of this table), and fanout[] table points at
// the offset/sha1[] table in a specific way (so that part of the
// latter table that covers all hashes that start with a given byte
// can be found to avoid 8 iterations of the binary search).
pub struct PackFileIndex<'a> {
    pub packfile: &'a PackFile,
}

impl<'a> PackFileIndex<'a> {
    pub fn encode_to(self, original_buf: &mut BytesMut) -> Result<(), anyhow::Error> {
        // split the buffer so we can hash only what we're currently generating at the
        // end of this function
        let mut buf = original_buf.split_off(original_buf.len());

        buf.extend_from_slice(b"\xfftOc"); // magic header
        buf.put_u32(2); // version

        // calculate total `PackFileEntry` hashes beginning with the same first byte
        let mut totals_by_first_byte = [0u32; 256];
        for entry in &self.packfile.entries {
            totals_by_first_byte[entry.uncompressed_sha1[0] as usize] += 1;
        }

        // calculate fanout value by taking cumulative totals of first byte counts
        let mut cumulative = 0;
        for i in 0..256usize {
            cumulative += totals_by_first_byte[i];
            buf.put_u32(cumulative);
        }

        // write all the sha hashes out, this needs to be sorted by the hash which should've
        // been done by `PackFile::new()`
        for entry in &self.packfile.entries {
            buf.extend_from_slice(&entry.uncompressed_sha1);
        }

        for entry in &self.packfile.entries {
            buf.put_u32(entry.compressed_crc32);
        }

        let mut offset = PackFile::header_size();

        // encode offsets into the packfile
        for entry in &self.packfile.entries {
            offset += entry.compressed_data.len();

            let mut offset_be = offset.to_be();

            while offset_be != 0 {
                // read 7 LSBs from the `offset_be` and push them off for the next iteration
                let mut val = (offset_be & 0b1111111) as u8;
                offset_be >>= 7;

                if offset_be != 0 {
                    // MSB set to 1 implies there's more offset_be bytes to come, otherwise
                    // the data starts after this byte
                    val |= 1 << 7;
                }

                buf.put_u8(val);
            }
        }

        // push a copy of the hash that appears at the end of the packfile
        buf.extend_from_slice(&self.packfile.hash);

        // hash of the whole buffer we've just generated for the index
        let mut hasher = Sha1::new();
        hasher.update(&buf);
        let result = hasher.finalize();
        buf.extend_from_slice(result.as_ref());

        // put the buffer we've just generated back into the mutable buffer we were passed
        original_buf.unsplit(buf);

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
    entries: Vec<PackFileEntry>,
    hash: [u8; 20],
}

impl PackFile {
    pub fn new(mut entries: Vec<PackFileEntry>) -> Self {
        entries.sort_unstable_by_key(|v| v.uncompressed_sha1[0]);
        let hash_buffer = entries.iter().fold(ConstBuffer::new(), |acc, curr| {
            acc.push_slice(&curr.uncompressed_sha1)
        });

        Self {
            entries,
            hash: sha1(&hash_buffer).bytes(),
        }
    }

    pub const fn header_size() -> usize {
        4 + std::mem::size_of::<u32>() + std::mem::size_of::<u32>()
    }

    pub fn encode_to(&self, original_buf: &mut BytesMut) -> Result<(), anyhow::Error> {
        let mut buf = original_buf.split_off(original_buf.len());

        buf.extend_from_slice(b"PACK"); // magic header
        buf.put_u32(2); // version
        buf.put_u32(self.entries.len().try_into().unwrap()); // number of entries in the packfile

        for entry in &self.entries {
            entry.encode_to(&mut buf)?;
        }

        buf.extend_from_slice(&sha1::Sha1::digest(&buf[..]));

        original_buf.unsplit(buf);

        Ok(())
    }
}

pub enum PackFileEntryType {
    // jordan@Jordans-MacBook-Pro-2 0d % printf "\x1f\x8b\x08\x00\x00\x00\x00\x00" | cat - f5/473259d9674ed66239766a013f96a3550374e3 | gzip -dc
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
    entry_type: PackFileEntryType,
    compressed_data: Vec<u8>,
    compressed_crc32: u32,
    uncompressed_sha1: [u8; 20],
    uncompressed_size: usize,
}

impl PackFileEntry {
    pub fn new(entry_type: PackFileEntryType, data: &[u8]) -> Result<Self, anyhow::Error> {
        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(&data)?;
        let compressed_data = e.finish()?;

        let compressed_crc32 = crc::Crc::<u32>::new(&crc::CRC_32_CKSUM).checksum(&compressed_data);

        Ok(Self {
            entry_type,
            compressed_data,
            compressed_crc32,
            uncompressed_sha1: sha1(&ConstBuffer::new().push_slice(data)).bytes(),
            uncompressed_size: data.len(),
        })
    }

    // fn size_of_data_be(&self) -> usize {
    //     self.uncompressed_size.to_be()
    // }

    // The object header is a series of one or more 1 byte (8 bit) hunks
    // that specify the type of object the following data is, and the size
    // of the data when expanded. Each byte is really 7 bits of data, with
    // the first bit being used to say if that hunk is the last one or not
    // before the data starts. If the first bit is a 1, you will read another
    // byte, otherwise the data starts next. The first 3 bits in the first
    // byte specifies the type of data, according to the table below.
    fn write_header(&self, buf: &mut BytesMut) {
        let mut size = self.uncompressed_size;

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

            // pack the 4 LSBs of the size into the header
            val |= (size & 0b1111) as u8;
            size >>= 4;

            buf.put_u8(val);
        }

        // write size bytes
        while size != 0 {
            // read 7 LSBs from the `size` and push them off for the next iteration
            let mut val = (size & 0b1111111) as u8;
            size >>= 7;

            if size != 0 {
                // MSB set to 1 implies there's more size bytes to come, otherwise
                // the data starts after this byte
                val |= 1 << 7;
            }

            buf.put_u8(val);
        }
    }

    pub fn encode_to(&self, buf: &mut BytesMut) -> Result<(), anyhow::Error> {
        self.write_header(buf);
        buf.extend_from_slice(&self.compressed_data);

        Ok(())
    }
}
