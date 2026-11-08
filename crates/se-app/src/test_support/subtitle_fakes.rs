//! [`FakeSubtitleFiles`]: SRT files kept in memory as cues.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, PoisonError},
};

use async_trait::async_trait;

use crate::ports::{SubtitleCue, SubtitleError, SubtitleFiles};

/// Subtitle files by path: written cues are kept as they are; reading an unknown path fails like
/// a missing file. Clones share the files.
#[derive(Debug, Clone, Default)]
pub struct FakeSubtitleFiles {
    files: Arc<Mutex<BTreeMap<PathBuf, Vec<SubtitleCue>>>>,
}

impl FakeSubtitleFiles {
    /// Places a file holding `cues` at `path`.
    pub fn put(&self, path: impl Into<PathBuf>, cues: Vec<SubtitleCue>) {
        self.lock().insert(path.into(), cues);
    }

    /// The cues written at `path`, if any.
    #[must_use]
    pub fn written(&self, path: impl AsRef<Path>) -> Option<Vec<SubtitleCue>> {
        self.lock().get(path.as_ref()).cloned()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, BTreeMap<PathBuf, Vec<SubtitleCue>>> {
        self.files.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[async_trait]
impl SubtitleFiles for FakeSubtitleFiles {
    async fn write_srt(&self, path: &Path, cues: &[SubtitleCue]) -> Result<(), SubtitleError> {
        self.lock().insert(path.to_path_buf(), cues.to_vec());
        Ok(())
    }

    async fn read_srt(&self, path: &Path) -> Result<Vec<SubtitleCue>, SubtitleError> {
        self.lock()
            .get(path)
            .cloned()
            .ok_or_else(|| SubtitleError::Read {
                path: path.display().to_string(),
                reason: "no such file".to_owned(),
            })
    }
}
