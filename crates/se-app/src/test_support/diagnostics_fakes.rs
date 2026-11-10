//! [`FakeDiagnosticsFiles`]: log files and written archives kept in memory.

use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex, PoisonError},
};

use async_trait::async_trait;

use crate::ports::{ArchiveEntry, DiagnosticsFiles, LogFile, StorageError};

#[derive(Debug, Default)]
struct State {
    logs: Vec<LogFile>,
    archives: Vec<(PathBuf, Vec<ArchiveEntry>)>,
}

/// Logs given by the test (newest first) and the archives written. Clones share them.
#[derive(Debug, Clone, Default)]
pub struct FakeDiagnosticsFiles {
    state: Arc<Mutex<State>>,
}

impl FakeDiagnosticsFiles {
    /// Adds a log file (call newest first).
    pub fn put_log(&self, name: &str, text: &str) {
        self.lock().logs.push(LogFile {
            name: name.to_owned(),
            text: text.to_owned(),
        });
    }

    /// The entries of the archive written at `path`, if any.
    #[must_use]
    pub fn archive(&self, path: impl AsRef<Path>) -> Option<Vec<ArchiveEntry>> {
        self.lock()
            .archives
            .iter()
            .rev()
            .find(|(written, _)| written == path.as_ref())
            .map(|(_, entries)| entries.clone())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[async_trait]
impl DiagnosticsFiles for FakeDiagnosticsFiles {
    async fn recent_logs(
        &self,
        _dir: &Path,
        max_files: usize,
        max_bytes: usize,
    ) -> Result<Vec<LogFile>, StorageError> {
        Ok(self
            .lock()
            .logs
            .iter()
            .take(max_files)
            .map(|log| LogFile {
                name: log.name.clone(),
                text: log
                    .text
                    .chars()
                    .rev()
                    .take(max_bytes)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect(),
            })
            .collect())
    }

    async fn write_archive(
        &self,
        path: &Path,
        entries: Vec<ArchiveEntry>,
    ) -> Result<u64, StorageError> {
        let size = entries.iter().map(|entry| entry.bytes.len() as u64).sum();
        self.lock().archives.push((path.to_path_buf(), entries));
        Ok(size)
    }
}
