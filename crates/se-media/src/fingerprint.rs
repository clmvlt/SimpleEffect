//! Content fingerprints (ADR-0018): size + modification time + xxh3-64 of the first and last
//! mebibyte. Reading 2 MiB at most keeps it instant even for 100 GB camera files.

use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::Path,
    time::UNIX_EPOCH,
};

use se_app::ports::MediaError;
use se_domain::ContentFingerprint;
use xxhash_rust::xxh3::Xxh3;

/// Bytes hashed at each end of the file.
pub const SAMPLE_BYTES: u64 = 1024 * 1024;

/// The fingerprint of the file at `path` (blocking IO: call from `spawn_blocking`).
///
/// # Errors
/// [`MediaError::NotFound`] or [`MediaError::Io`].
pub fn fingerprint_file(path: &Path) -> Result<ContentFingerprint, MediaError> {
    let io_error = |error: io::Error| {
        if error.kind() == io::ErrorKind::NotFound {
            MediaError::NotFound {
                path: path.to_path_buf(),
            }
        } else {
            MediaError::Io {
                path: path.to_path_buf(),
                message: error.to_string(),
            }
        }
    };
    let mut file = File::open(path).map_err(io_error)?;
    let metadata = file.metadata().map_err(io_error)?;
    let size = metadata.len();
    let modified_unix_ms = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .and_then(|elapsed| i64::try_from(elapsed.as_millis()).ok())
        .unwrap_or(0);
    let hash = sample_hash(&mut file, size).map_err(io_error)?;
    ContentFingerprint::new(size, modified_unix_ms, format!("{hash:016x}"))
        .map_err(|error| MediaError::Failed(error.to_string()))
}

/// xxh3-64 of the first [`SAMPLE_BYTES`] and the last [`SAMPLE_BYTES`] (the whole file when it is
/// smaller than both together).
fn sample_hash(reader: &mut (impl Read + Seek), size: u64) -> io::Result<u64> {
    let mut hasher = Xxh3::new();
    let mut buffer = Vec::new();
    reader.take(SAMPLE_BYTES).read_to_end(&mut buffer)?;
    hasher.update(&buffer);
    if size > SAMPLE_BYTES {
        let tail_start = size.saturating_sub(SAMPLE_BYTES).max(SAMPLE_BYTES);
        reader.seek(SeekFrom::Start(tail_start))?;
        buffer.clear();
        reader.take(SAMPLE_BYTES).read_to_end(&mut buffer)?;
        hasher.update(&buffer);
    }
    Ok(hasher.digest())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn only_the_ends_of_large_files_count() {
        let size = usize::try_from(SAMPLE_BYTES * 3).unwrap();
        let mut original = vec![7_u8; size];
        let mut edited_middle = original.clone();
        edited_middle[size / 2] = 0;
        let hash = |bytes: &mut Vec<u8>| {
            let len = bytes.len() as u64;
            sample_hash(&mut Cursor::new(bytes), len).unwrap()
        };
        let reference = hash(&mut original);
        assert_eq!(hash(&mut edited_middle), reference);
        let mut edited_end = original.clone();
        edited_end[size - 1] = 0;
        assert_ne!(hash(&mut edited_end), reference);
    }

    #[test]
    fn small_files_are_hashed_whole() {
        let mut short = b"abc".to_vec();
        let len = short.len() as u64;
        let first = sample_hash(&mut Cursor::new(&mut short), len).unwrap();
        short[1] = b'x';
        assert_ne!(
            sample_hash(&mut Cursor::new(&mut short), len).unwrap(),
            first
        );
    }

    #[test]
    fn files_get_a_valid_fingerprint() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("clip.bin");
        std::fs::write(&path, b"media bytes").unwrap();
        let fingerprint = fingerprint_file(&path).unwrap();
        assert_eq!(fingerprint.size_bytes(), 11);
        assert!(fingerprint.modified_unix_ms() > 0);
        let missing = fingerprint_file(&dir.path().join("gone")).unwrap_err();
        assert!(matches!(missing, MediaError::NotFound { .. }));
    }
}
