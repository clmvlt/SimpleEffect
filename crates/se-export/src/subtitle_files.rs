//! [`FsSubtitleFiles`]: the subtitle file port on the local file system.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use se_app::ports::{SubtitleCue, SubtitleError, SubtitleFiles};

use crate::srt::{parse_srt, write_srt};

/// Reads and writes SRT files with blocking IO on tokio's blocking pool. Writes go to a
/// temporary file next to the target, then replace it.
#[derive(Debug, Default, Clone, Copy)]
pub struct FsSubtitleFiles;

#[async_trait]
impl SubtitleFiles for FsSubtitleFiles {
    async fn write_srt(&self, path: &Path, cues: &[SubtitleCue]) -> Result<(), SubtitleError> {
        let text = write_srt(cues);
        let path = path.to_path_buf();
        let shown = path.display().to_string();
        let result = tokio::task::spawn_blocking(move || write_atomically(&path, text.as_bytes()))
            .await
            .map_err(|error| error.to_string())
            .and_then(|written| written.map_err(|error| error.to_string()));
        result.map_err(|reason| SubtitleError::Write {
            path: shown,
            reason,
        })
    }

    async fn read_srt(&self, path: &Path) -> Result<Vec<SubtitleCue>, SubtitleError> {
        let owned = path.to_path_buf();
        let shown = path.display().to_string();
        let bytes = tokio::task::spawn_blocking(move || std::fs::read(owned))
            .await
            .map_err(|error| error.to_string())
            .and_then(|read| read.map_err(|error| error.to_string()))
            .map_err(|reason| SubtitleError::Read {
                path: shown.clone(),
                reason,
            })?;
        let text = decode(&bytes);
        parse_srt(&text).map_err(|error| SubtitleError::Invalid {
            path: shown,
            reason: error.to_string(),
        })
    }
}

/// UTF-8 when valid, else Windows-1252/Latin-1 (older subtitle files), byte for byte.
fn decode(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec())
        .unwrap_or_else(|_| bytes.iter().map(|&byte| char::from(byte)).collect())
}

fn write_atomically(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let temporary: PathBuf = {
        let mut name = path.file_name().unwrap_or_default().to_os_string();
        name.push(".tmp");
        path.with_file_name(name)
    };
    std::fs::write(&temporary, bytes)?;
    std::fs::rename(&temporary, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&temporary);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use se_domain::Flicks;

    #[tokio::test]
    async fn writes_then_reads_a_file() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("sous-titres.srt");
        let cues = vec![SubtitleCue {
            start: Flicks::ZERO,
            end: Flicks::one_second(),
            text: "Déjà là".to_owned(),
        }];
        FsSubtitleFiles.write_srt(&path, &cues).await.unwrap();
        assert_eq!(FsSubtitleFiles.read_srt(&path).await.unwrap(), cues);
        let missing = FsSubtitleFiles
            .read_srt(&folder.path().join("absent.srt"))
            .await;
        assert!(matches!(missing, Err(SubtitleError::Read { .. })));
    }

    #[tokio::test]
    async fn reads_latin1_files_and_reports_invalid_ones() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("ancien.srt");
        std::fs::write(&path, b"1\n00:00:00,000 --> 00:00:01,000\nD\xe9j\xe0\n").unwrap();
        let cues = FsSubtitleFiles.read_srt(&path).await.unwrap();
        assert_eq!(cues[0].text, "Déjà");
        std::fs::write(&path, "pas des sous-titres").unwrap();
        let invalid = FsSubtitleFiles.read_srt(&path).await;
        assert!(matches!(invalid, Err(SubtitleError::Invalid { .. })));
    }
}
