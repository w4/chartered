// The offset/sha1[] tables are sorted by sha1[] values (this is to
// allow binary search of this table), and fanout[] table points at
// the offset/sha1[] table in a specific way (so that part of the
// latter table that covers all hashes that start with a given byte
// can be found to avoid 8 iterations of the binary search).
//
// packfile indexes are not neccesary to extract objects from a packfile
pub struct PackFileIndex<const S: usize> {
    // S should be u16
    pub magic: [u8; 4],   // "\x337t0c" - header magic value
    pub version: [u8; 4], // "0002", - header version
    pub fanout: [[u8; 4]; 255],
    pub size: u16,            // fanout[256] => size == S
    pub sha1: [[u8; 20]; S],  // sha listing
    pub crc: [[u8; 4]; S],    // checksum
    pub offset: [[u8; 4]; S], // packfile offsets
    // 64b_offset: [[u8; 8]; N], // for packfiles over 2gb
    pub packfile_checksum: [u8; 20], // sha1
    pub idxfiel_checksum: [u8; 20],  // sha1
}

// The packfile itself is a very simple format. There is a header, a
// series of packed objects (each with it's own header and body) and
// then a checksum trailer. The first four bytes is the string 'PACK',
// which is sort of used to make sure you're getting the start of the
// packfile correctly. This is followed by a 4-byte packfile version
// number and then a 4-byte number of entries in that file.
