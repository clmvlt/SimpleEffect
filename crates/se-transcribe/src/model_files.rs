//! [`FsModelFiles`]: the `ModelFiles` port over the models folder.

use std::path::Path;

use async_trait::async_trait;
use se_app::ports::{MediaError, ModelFile, ModelFiles};

use crate::{download::part_path, hashing::sha256_file};

/// First bytes of a ggml file: the magic `0x67676d6c` written little-endian.
const GGML_MAGIC: [u8; 4] = *b"lmgg";
/// Whisper models start with their vocabulary size after the magic (51864–51866); the voice model
/// and other ggml files hold much smaller numbers there.
const MIN_WHISPER_VOCABULARY: u32 = 50_000;

/// The models folder on disk.
#[derive(Debug, Clone, Copy, Default)]
pub struct FsModelFiles;

#[async_trait]
impl ModelFiles for FsModelFiles {
    async fn list(&self, dir: &Path) -> Vec<ModelFile> {
        let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
            return Vec::new();
        };
        let mut files = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Ok(metadata) = entry.metadata().await else {
                continue;
            };
            if metadata.is_file() && name.to_ascii_lowercase().ends_with(".bin") {
                files.push(ModelFile {
                    file_name: name,
                    size: metadata.len(),
                });
            }
        }
        files.sort_by(|left, right| left.file_name.cmp(&right.file_name));
        files
    }

    async fn sha256(&self, path: &Path) -> Result<String, MediaError> {
        if !tokio::fs::try_exists(path).await.unwrap_or(false) {
            return Err(MediaError::NotFound {
                path: path.to_path_buf(),
            });
        }
        sha256_file(path).await.map_err(|message| MediaError::Io {
            path: path.to_path_buf(),
            message,
        })
    }

    async fn is_whisper_model(&self, path: &Path) -> bool {
        use tokio::io::AsyncReadExt;
        let Ok(mut file) = tokio::fs::File::open(path).await else {
            return false;
        };
        let mut header = [0_u8; 8];
        if file.read_exact(&mut header).await.is_err() {
            return false;
        }
        let vocabulary = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        header[..4] == GGML_MAGIC && vocabulary >= MIN_WHISPER_VOCABULARY
    }

    async fn copy(&self, source: &Path, destination: &Path) -> Result<(), MediaError> {
        let io = |path: &Path, error: std::io::Error| MediaError::Io {
            path: path.to_path_buf(),
            message: error.to_string(),
        };
        if let Some(dir) = destination.parent() {
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(|error| io(dir, error))?;
        }
        let part = part_path(destination);
        tokio::fs::copy(source, &part)
            .await
            .map_err(|error| io(source, error))?;
        tokio::fs::rename(&part, destination)
            .await
            .map_err(|error| io(destination, error))
    }

    async fn remove(&self, path: &Path) -> Result<(), MediaError> {
        match tokio::fs::remove_file(path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(MediaError::Io {
                path: path.to_path_buf(),
                message: error.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn lists_checks_copies_and_removes_models() {
        let dir = tempfile::tempdir().unwrap();
        let mut model = GGML_MAGIC.to_vec();
        model.extend_from_slice(&51_865_u32.to_le_bytes());
        let mut voice = GGML_MAGIC.to_vec();
        voice.extend_from_slice(&10_u32.to_le_bytes());
        let source = dir.path().join("downloaded.bin");
        std::fs::write(&source, &model).unwrap();
        std::fs::write(dir.path().join("voice.bin"), &voice).unwrap();
        std::fs::write(dir.path().join("half.bin.part"), b"x").unwrap();
        let files = FsModelFiles;
        assert!(files.is_whisper_model(&source).await);
        assert!(!files.is_whisper_model(&dir.path().join("voice.bin")).await);
        assert!(
            !files
                .is_whisper_model(&dir.path().join("half.bin.part"))
                .await
        );

        let models = dir.path().join("models");
        files
            .copy(&source, &models.join("ggml-base.bin"))
            .await
            .unwrap();
        let listed = files.list(&models).await;
        assert_eq!(
            listed,
            [ModelFile {
                file_name: "ggml-base.bin".to_owned(),
                size: 8
            }]
        );
        assert_eq!(
            files
                .sha256(&models.join("ggml-base.bin"))
                .await
                .unwrap()
                .len(),
            64
        );
        files.remove(&models.join("ggml-base.bin")).await.unwrap();
        files.remove(&models.join("ggml-base.bin")).await.unwrap();
        assert_eq!(files.list(&models).await.len(), 0);
        assert_eq!(files.list(&dir.path().join("none")).await.len(), 0);
    }
}
