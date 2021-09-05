use bytes::{BufMut, BytesMut};
use flate2::{write::ZlibEncoder, Compression};
use sha1::{
    digest::{generic_array::GenericArray, FixedOutputDirty},
    Digest, Sha1,
};
use std::{convert::TryInto, fmt::Write, io::Write as IoWrite};

// The packfile itself is a very simple format. There is a header, a
// series of packed objects (each with it's own header and body) and
// then a checksum trailer. The first four bytes is the string 'PACK',
// which is sort of used to make sure you're getting the start of the
// packfile correctly. This is followed by a 4-byte packfile version
// number and then a 4-byte number of entries in that file.
pub struct PackFile<'a> {
    entries: Vec<PackFileEntry<'a>>,
}

impl<'a> PackFile<'a> {
    #[must_use]
    pub fn new(entries: Vec<PackFileEntry<'a>>) -> Self {
        Self { entries }
    }

    #[must_use]
    pub const fn header_size() -> usize {
        "PACK".len() + std::mem::size_of::<u32>() + std::mem::size_of::<u32>()
    }

    #[must_use]
    pub const fn footer_size() -> usize {
        20
    }

    pub fn encode_to(&self, original_buf: &mut BytesMut) -> Result<(), anyhow::Error> {
        let mut buf = original_buf.split_off(original_buf.len());
        buf.reserve(Self::header_size() + Self::footer_size());

        // header
        buf.extend_from_slice(b"PACK"); // magic header
        buf.put_u32(2); // version
        buf.put_u32(self.entries.len().try_into()?); // number of entries in the packfile

        // body
        for entry in &self.entries {
            entry.encode_to(&mut buf)?;
        }

        // footer
        buf.extend_from_slice(&sha1::Sha1::digest(&buf[..]));

        original_buf.unsplit(buf);

        Ok(())
    }
}

#[derive(Debug)]
pub struct Commit<'a> {
    pub tree: GenericArray<u8, <Sha1 as FixedOutputDirty>::OutputSize>, // [u8; 20], but sha-1 returns a GenericArray
    // pub parent: [u8; 20],
    pub author: CommitUserInfo<'a>,
    pub committer: CommitUserInfo<'a>,
    // pub gpgsig: &str,
    pub message: &'a str,
}

impl Commit<'_> {
    fn encode_to(&self, out: &mut BytesMut) -> Result<(), anyhow::Error> {
        let mut tree_hex = [0_u8; 20 * 2];
        hex::encode_to_slice(self.tree, &mut tree_hex)?;

        out.write_str("tree ")?;
        out.extend_from_slice(&tree_hex);
        out.write_char('\n')?;

        writeln!(out, "author {}", self.author.encode())?;
        writeln!(out, "committer {}", self.committer.encode())?;
        write!(out, "\n{}", self.message)?;

        Ok(())
    }

    #[must_use]
    pub fn size(&self) -> usize {
        let mut len = 0;
        len += "tree ".len() + (self.tree.len() * 2) + "\n".len();
        len += "author ".len() + self.author.size() + "\n".len();
        len += "committer ".len() + self.committer.size() + "\n".len();
        len += "\n".len() + self.message.len();
        len
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CommitUserInfo<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub time: chrono::DateTime<chrono::Utc>,
}

impl CommitUserInfo<'_> {
    fn encode(&self) -> String {
        // TODO: remove `format!`, `format_args!`?
        format!(
            "{} <{}> {} +0000",
            self.name,
            self.email,
            self.time.timestamp()
        )
    }

    #[must_use]
    pub fn size(&self) -> usize {
        let timestamp_len = itoa::Buffer::new().format(self.time.timestamp()).len();

        self.name.len()
            + "< ".len()
            + self.email.len()
            + "> ".len()
            + timestamp_len
            + " +0000".len()
    }
}

#[derive(Debug)]
pub enum TreeItemKind {
    File,
    Directory,
}

impl TreeItemKind {
    #[must_use]
    pub const fn mode(&self) -> &'static str {
        match self {
            Self::File => "100644",
            Self::Directory => "40000",
        }
    }
}

#[derive(Debug)]
pub struct TreeItem<'a> {
    pub kind: TreeItemKind,
    pub name: &'a str,
    pub hash: GenericArray<u8, <Sha1 as FixedOutputDirty>::OutputSize>, // [u8; 20] - but we have to deal with GenericArrays
}

// `[mode] [name]\0[hash]`
impl TreeItem<'_> {
    fn encode_to(&self, out: &mut BytesMut) -> Result<(), anyhow::Error> {
        out.write_str(self.kind.mode())?;
        write!(out, " {}\0", self.name)?;
        out.extend_from_slice(&self.hash);
        Ok(())
    }

    #[must_use]
    pub fn size(&self) -> usize {
        self.kind.mode().len() + " ".len() + self.name.len() + "\0".len() + self.hash.len()
    }
}

#[derive(Debug)]
pub enum PackFileEntry<'a> {
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
    Commit(Commit<'a>),
    // jordan@Jordans-MacBook-Pro-2 0d % printf "\x1f\x8b\x08\x00\x00\x00\x00\x00" | cat - 0d/586b48bc42e8591773d3d8a7223551c39d453c | gzip -dc
    // tree 20940000 .cargo���CYy��Ve�������100644 .gitignore�K��_ow�]����4�n�ݺ100644 Cargo.lock�7�3-�?/��
    // kt��c0C�100644 Cargo.toml�6�&(��]\8@�SHA�]f40000 src0QW��ƅ���b[�!�S&N�100644 test�G2Y�gN�b9vj?��Ut�
    Tree(Vec<TreeItem<'a>>),
    // jordan@Jordans-MacBook-Pro-2 objects % printf "\x1f\x8b\x08\x00\x00\x00\x00\x00" | cat - f5/473259d9674ed66239766a013f96a3550374e3| gzip -dc
    // blob 23try and find me in .git
    Blob(&'a [u8]),
    // Tag,
    // OfsDelta,
    // RefDelta,
}

impl PackFileEntry<'_> {
    fn write_header(&self, buf: &mut BytesMut) {
        let mut size = self.uncompressed_size();

        // write header
        {
            let mut val = 0b1000_0000_u8;

            val |= match self {
                Self::Commit(_) => 0b001,
                Self::Tree(_) => 0b010,
                Self::Blob(_) => 0b011,
                // Self::Tag => 0b100,
                // Self::OfsDelta => 0b110,
                // Self::RefDelta => 0b111,
            } << 4;

            // pack the 4 LSBs of the size into the header
            #[allow(clippy::cast_possible_truncation)] // value is masked
            {
                val |= (size & 0b1111) as u8;
            }
            size >>= 4;

            buf.put_u8(val);
        }

        // write size bytes
        while size != 0 {
            // read 7 LSBs from the `size` and push them off for the next iteration
            #[allow(clippy::cast_possible_truncation)] // value is masked
            let mut val = (size & 0b111_1111) as u8;
            size >>= 7;

            if size != 0 {
                // MSB set to 1 implies there's more size bytes to come, otherwise
                // the data starts after this byte
                val |= 1 << 7;
            }

            buf.put_u8(val);
        }
    }

    pub fn encode_to(&self, original_out: &mut BytesMut) -> Result<(), anyhow::Error> {
        self.write_header(original_out); // TODO: this needs space reserving for it

        // todo is there a way to stream through the zlibencoder so we don't have to
        // have this intermediate bytesmut and vec?
        let mut out = BytesMut::new();

        let size = self.uncompressed_size();
        original_out.reserve(size);
        // the data ends up getting compressed but we'll need at least this many bytes
        out.reserve(size);

        match self {
            Self::Commit(commit) => {
                commit.encode_to(&mut out)?;
            }
            Self::Tree(items) => {
                for item in items {
                    item.encode_to(&mut out)?;
                }
            }
            Self::Blob(data) => {
                out.extend_from_slice(data);
            }
        }

        debug_assert_eq!(out.len(), size);

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(&out)?;
        let compressed_data = e.finish()?;

        original_out.extend_from_slice(&compressed_data);

        Ok(())
    }

    #[must_use]
    pub fn uncompressed_size(&self) -> usize {
        match self {
            Self::Commit(commit) => commit.size(),
            Self::Tree(items) => items.iter().map(TreeItem::size).sum(),
            Self::Blob(data) => data.len(),
        }
    }

    // wen const generics for RustCrypto? :-(
    pub fn hash(
        &self,
    ) -> Result<GenericArray<u8, <Sha1 as FixedOutputDirty>::OutputSize>, anyhow::Error> {
        let size = self.uncompressed_size();

        let file_prefix = match self {
            Self::Commit(_) => "commit",
            Self::Tree(_) => "tree",
            Self::Blob(_) => "blob",
        };

        let size_len = itoa::Buffer::new().format(size).len();

        let mut out =
            BytesMut::with_capacity(file_prefix.len() + " ".len() + size_len + "\n".len() + size);

        write!(out, "{} {}\0", file_prefix, size)?;
        match self {
            Self::Commit(commit) => {
                commit.encode_to(&mut out)?;
            }
            Self::Tree(items) => {
                for item in items {
                    item.encode_to(&mut out)?;
                }
            }
            Self::Blob(blob) => {
                out.extend_from_slice(blob);
            }
        }

        Ok(sha1::Sha1::digest(&out))
    }
}
