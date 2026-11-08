//! SHA-256 of files, on the blocking pool.

use std::{io::Read, path::Path};

use sha2::{Digest, Sha256};

/// The SHA-256 of the file at `path`, lowercase hex.
///
/// # Errors
/// A message naming the file when it cannot be read.
pub async fn sha256_file(path: &Path) -> Result<String, String> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let error = |error: std::io::Error| format!("{} cannot be read: {error}", path.display());
        let mut file = std::fs::File::open(&path).map_err(error)?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0_u8; 1 << 20];
        loop {
            let read = file.read(&mut buffer).map_err(error)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        Ok(hex::encode(hasher.finalize()))
    })
    .await
    .map_err(|error| format!("hashing failed: {error}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn hashes_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("abc");
        std::fs::write(&path, b"abc").unwrap();
        assert_eq!(
            sha256_file(&path).await.unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert!(sha256_file(&dir.path().join("none")).await.is_err());
    }
}
