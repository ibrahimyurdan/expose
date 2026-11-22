// reads one file out of riot's wad archives (version 3), the format league
// keeps its interface files in. an entry is keyed by the xxh64 hash of its
// lowercased path and stored raw or zstd-compressed.

use std::io::{Read, Seek, SeekFrom};

use ruzstd::decoding::StreamingDecoder;
use twox_hash::XxHash64;

use crate::error::{Error, Result};

// the entry count sits after the magic, the version, a 256 byte signature and
// an 8 byte checksum; the table of entries follows it.
const ENTRY_COUNT_AT: u64 = 4 + 256 + 8;
const ENTRY_SIZE: usize = 32;

// a corrupt header must not make us allocate without bound. league's archives
// hold thousands of entries, and these files are kilobytes.
const MAX_ENTRIES: usize = 1_000_000;
const MAX_FILE: usize = 64 * 1024 * 1024;

// entry types this reader unpacks.
const STORED: u8 = 0;
const ZSTD: u8 = 3;

// the key an archive files an entry under.
pub fn path_key(path: &str) -> u64 {
    XxHash64::oneshot(0, path.to_lowercase().as_bytes())
}

// the file at `path`, unpacked, or none when the archive has no such entry.
// an archive that is not wad version 3, an entry of another type, or one that
// doesn't unpack to its listed size is an error.
pub fn read_file<R: Read + Seek>(archive: &mut R, path: &str) -> Result<Option<Vec<u8>>> {
    let mut head = [0u8; 4];
    archive.read_exact(&mut head)?;
    if &head[..2] != b"RW" || head[2] != 3 {
        return Err(Error::Message("not a version 3 wad archive".to_string()));
    }
    archive.seek(SeekFrom::Start(ENTRY_COUNT_AT))?;
    let count = read_u32(archive)? as usize;
    if count > MAX_ENTRIES {
        return Err(Error::Message(format!("wad archive lists {count} entries")));
    }
    let mut table = vec![0u8; count * ENTRY_SIZE];
    archive.read_exact(&mut table)?;

    let key = path_key(path);
    let (entries, _) = table.as_chunks::<ENTRY_SIZE>();
    let Some(entry) = entries
        .iter()
        .find(|entry| u64::from_le_bytes(field(entry.as_slice(), 0)) == key)
    else {
        return Ok(None);
    };
    let offset = u32::from_le_bytes(field(entry, 8));
    let stored = u32::from_le_bytes(field(entry, 12)) as usize;
    let size = u32::from_le_bytes(field(entry, 16)) as usize;
    let kind = entry[20] & 0x0f;
    if stored > MAX_FILE || size > MAX_FILE {
        return Err(Error::Message(format!("wad entry for {path} is too large")));
    }

    archive.seek(SeekFrom::Start(u64::from(offset)))?;
    let mut data = vec![0u8; stored];
    archive.read_exact(&mut data)?;
    let bytes = match kind {
        STORED => data,
        ZSTD => unzstd(&data)?,
        other => {
            return Err(Error::Message(format!(
                "wad entry type {other} is not read"
            )))
        }
    };
    if bytes.len() != size {
        return Err(Error::Message(format!(
            "wad entry for {path} unpacked to {} bytes, not {size}",
            bytes.len()
        )));
    }
    Ok(Some(bytes))
}

fn read_u32<R: Read>(reader: &mut R) -> Result<u32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

// n bytes of an entry from `at`; every entry is a full 32 bytes.
fn field<const N: usize>(entry: &[u8], at: usize) -> [u8; N] {
    let mut bytes = [0u8; N];
    bytes.copy_from_slice(&entry[at..at + N]);
    bytes
}

fn unzstd(data: &[u8]) -> Result<Vec<u8>> {
    let mut source = data;
    let mut decoder = StreamingDecoder::new(&mut source)
        .map_err(|error| Error::Message(format!("zstd: {error}")))?;
    let mut out = Vec::new();
    decoder.read_to_end(&mut out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    // a version 3.4 archive holding the given entries: (path, entry type,
    // stored bytes, size once unpacked).
    fn archive(entries: &[(&str, u8, &[u8], usize)]) -> Vec<u8> {
        let mut out = b"RW".to_vec();
        out.extend([3, 4]);
        out.extend([0u8; 256 + 8]);
        out.extend(u32::try_from(entries.len()).unwrap().to_le_bytes());
        let mut offset = out.len() + entries.len() * 32;
        let mut data = Vec::new();
        for (path, kind, stored, size) in entries {
            out.extend(path_key(path).to_le_bytes());
            out.extend(u32::try_from(offset).unwrap().to_le_bytes());
            out.extend(u32::try_from(stored.len()).unwrap().to_le_bytes());
            out.extend(u32::try_from(*size).unwrap().to_le_bytes());
            out.push(*kind);
            out.extend([0u8; 11]);
            data.extend_from_slice(stored);
            offset += stored.len();
        }
        out.extend(data);
        out
    }

    // "<svg/>" as one zstd frame.
    const SVG_ZSTD: [u8; 15] = [40, 181, 47, 253, 32, 6, 49, 0, 0, 60, 115, 118, 103, 47, 62];

    #[test]
    fn path_keys_match_the_ones_in_league_archives() {
        assert_eq!(path_key("abc"), 0x44bc_2cf5_ad77_0999);
        assert_eq!(
            path_key("plugins/rcp-fe-lol-static-assets/global/default/images/ranked-mini-crests/gold.svg"),
            0xf0dd_5df1_a5d6_3edf
        );
        // keys come from the lowercased path.
        assert_eq!(
            path_key("PLUGINS/RCP-FE-LOL-STATIC-ASSETS/GLOBAL/DEFAULT/SVG/POSITION-TOP.SVG"),
            0xe0d2_77b1_26b1_c69f
        );
    }

    #[test]
    fn reads_a_stored_entry() {
        let bytes = archive(&[("a/raw.png", 0, b"png bytes", 9)]);
        assert_eq!(
            read_file(&mut Cursor::new(bytes), "a/raw.png").unwrap(),
            Some(b"png bytes".to_vec())
        );
    }

    #[test]
    fn unpacks_a_zstd_entry() {
        let bytes = archive(&[("a/raw.png", 0, b"png", 3), ("b/icon.svg", 3, &SVG_ZSTD, 6)]);
        assert_eq!(
            read_file(&mut Cursor::new(bytes), "b/icon.svg").unwrap(),
            Some(b"<svg/>".to_vec())
        );
    }

    #[test]
    fn a_path_the_archive_lacks_is_none() {
        let bytes = archive(&[("a/raw.png", 0, b"png", 3)]);
        assert_eq!(
            read_file(&mut Cursor::new(bytes), "a/other.png").unwrap(),
            None
        );
    }

    #[test]
    fn refuses_what_it_cannot_read() {
        let mut not_wad = archive(&[("a.png", 0, b"x", 1)]);
        not_wad[0] = b'X';
        assert!(read_file(&mut Cursor::new(not_wad), "a.png").is_err());

        let mut older = archive(&[("a.png", 0, b"x", 1)]);
        older[2] = 2;
        assert!(read_file(&mut Cursor::new(older), "a.png").is_err());

        // gzip entries never hold these files; the reader leaves them alone.
        let gzip = archive(&[("a.png", 1, b"x", 1)]);
        assert!(read_file(&mut Cursor::new(gzip), "a.png").is_err());

        // a size that does not match what was unpacked.
        let short = archive(&[("a.png", 0, b"x", 5)]);
        assert!(read_file(&mut Cursor::new(short), "a.png").is_err());
    }
}
