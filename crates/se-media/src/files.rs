//! [`FsMediaFiles`]: the `MediaFiles` port over the file system.

use std::{
    io,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use se_app::ports::{MediaError, MediaFiles};
use se_domain::ContentFingerprint;

use crate::fingerprint::fingerprint_file;

/// Existence checks and fingerprints of real files.
#[derive(Debug, Clone, Copy, Default)]
pub struct FsMediaFiles;

#[async_trait]
impl MediaFiles for FsMediaFiles {
    async fn is_file(&self, path: &Path) -> bool {
        tokio::fs::metadata(path)
            .await
            .is_ok_and(|metadata| metadata.is_file())
    }

    async fn fingerprint(&self, path: &Path) -> Result<ContentFingerprint, MediaError> {
        let owned = path.to_path_buf();
        tokio::task::spawn_blocking(move || fingerprint_file(&owned))
            .await
            .map_err(|error| MediaError::Failed(format!("fingerprinting stopped: {error}")))?
    }

    async fn directory_size(&self, dir: &Path) -> u64 {
        let owned = dir.to_path_buf();
        tokio::task::spawn_blocking(move || size_of(&owned))
            .await
            .unwrap_or(0)
    }

    async fn remove_directory(&self, dir: &Path) -> Result<(), MediaError> {
        let owned = dir.to_path_buf();
        let failure = tokio::task::spawn_blocking(move || remove_all(&owned))
            .await
            .map_err(|error| MediaError::Failed(format!("deleting stopped: {error}")))?;
        match failure {
            None => Ok(()),
            Some((path, error)) => Err(MediaError::Io {
                path,
                message: format!("{error} (the file may be in use; other files were deleted)"),
            }),
        }
    }
}

/// Bytes of the files under `path` (symbolic links are not followed); 0 when it does not exist.
fn size_of(path: &Path) -> u64 {
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return 0;
    };
    if !metadata.is_dir() {
        return metadata.len();
    }
    std::fs::read_dir(path).map_or(0, |entries| {
        entries
            .filter_map(Result::ok)
            .map(|entry| size_of(&entry.path()))
            .sum()
    })
}

/// Deletes `path` and everything under it, going on past failures; returns the first failure.
fn remove_all(path: &Path) -> Option<(PathBuf, io::Error)> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return None,
        Err(error) => return Some((path.to_path_buf(), error)),
    };
    if !metadata.is_dir() {
        return std::fs::remove_file(path)
            .err()
            .map(|error| (path.to_path_buf(), error));
    }
    let mut first_failure = None;
    match std::fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries.filter_map(Result::ok) {
                let failure = remove_all(&entry.path());
                first_failure = first_failure.or(failure);
            }
        }
        Err(error) => first_failure = Some((path.to_path_buf(), error)),
    }
    if first_failure.is_none()
        && let Err(error) = std::fs::remove_dir(path)
    {
        first_failure = Some((path.to_path_buf(), error));
    }
    first_failure
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn folders_are_measured_and_removed_with_their_content() {
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("cache");
        std::fs::create_dir_all(cache.join("media").join("key")).unwrap();
        std::fs::write(cache.join("media").join("key").join("a.bin"), [0_u8; 100]).unwrap();
        std::fs::write(cache.join("b.bin"), [0_u8; 50]).unwrap();
        let files = FsMediaFiles;
        assert_eq!(files.directory_size(&cache).await, 150);
        files.remove_directory(&cache).await.unwrap();
        assert!(!cache.exists());
        assert_eq!(files.directory_size(&cache).await, 0);
        files.remove_directory(&cache).await.unwrap();
    }
}
