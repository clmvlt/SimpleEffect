//! [`FsMediaFiles`]: the `MediaFiles` port over the file system.

use std::path::Path;

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
}
