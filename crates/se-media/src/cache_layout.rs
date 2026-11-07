//! Files of an asset's cache folder (`<project>/cache/media/<fingerprint key>/`, ADR-0018) and how
//! they are written: to a temporary name, then renamed, so a file that exists is complete.

use std::{
    io,
    path::{Path, PathBuf},
};

use se_protocol::JobKind;

use crate::peaks::PEAK_LEVELS;

/// The thumbnail sheet (JPEG, tiles of 90 px high).
pub const THUMBNAIL_SHEET: &str = "thumbnails.jpg";
/// The poster (JPEG, up to 360 px high).
pub const THUMBNAIL_POSTER: &str = "poster.jpg";
/// The sheet's layout (`se_protocol::ThumbnailSheet` as JSON), written last.
pub const THUMBNAIL_INFO: &str = "thumbnails.json";
/// Working folder of a thumbnails job (removed when it ends).
pub const THUMBNAIL_WORK_DIR: &str = "thumbnails.tmp";

/// The `SEPK` file of one peak level (`peaks-256.sepk`).
#[must_use]
pub fn peaks_file(samples_per_peak: u32) -> String {
    format!("peaks-{samples_per_peak}.sepk")
}

/// Files whose presence means a cache job of `kind` completed; `None` for jobs whose outputs are
/// not cache files (mezzanines and proxies are tracked by the document, analyses by the project
/// database).
#[must_use]
pub fn completion_files(kind: JobKind) -> Option<Vec<String>> {
    match kind {
        JobKind::Thumbnails => Some(vec![
            THUMBNAIL_SHEET.to_owned(),
            THUMBNAIL_POSTER.to_owned(),
            THUMBNAIL_INFO.to_owned(),
        ]),
        JobKind::WaveformPeaks => {
            Some(PEAK_LEVELS.iter().map(|level| peaks_file(*level)).collect())
        }
        JobKind::Mezzanine | JobKind::Proxy | JobKind::AudioAnalysis | JobKind::SceneDetection => {
            None
        }
    }
}

/// Whether every output of a cache job of `kind` exists in `dir` (always `false` for other kinds).
pub async fn outputs_ready(kind: JobKind, dir: &Path) -> bool {
    let Some(files) = completion_files(kind) else {
        return false;
    };
    for file in files {
        if !tokio::fs::try_exists(dir.join(file)).await.unwrap_or(false) {
            return false;
        }
    }
    true
}

/// Writes `bytes` to `path` through a temporary file renamed over it.
///
/// # Errors
/// IO errors.
pub async fn write_atomically(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let temporary = temporary_path(path);
    tokio::fs::write(&temporary, bytes).await?;
    tokio::fs::rename(&temporary, path).await
}

/// `<path>.part`.
fn temporary_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".part");
    path.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn outputs_are_ready_only_when_every_file_exists() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!outputs_ready(JobKind::WaveformPeaks, dir.path()).await);
        for level in PEAK_LEVELS {
            write_atomically(&dir.path().join(peaks_file(level)), b"x")
                .await
                .unwrap();
        }
        assert!(outputs_ready(JobKind::WaveformPeaks, dir.path()).await);
        assert!(!outputs_ready(JobKind::Thumbnails, dir.path()).await);
        assert!(!outputs_ready(JobKind::Mezzanine, dir.path()).await);
        assert!(!dir.path().join("peaks-256.sepk.part").exists());
    }
}
