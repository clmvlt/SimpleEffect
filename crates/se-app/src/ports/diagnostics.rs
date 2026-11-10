//! Diagnostics files port (prompt 30): reading the core's recent log files and writing a zip
//! archive, implemented by se-store (`FsDiagnosticsFiles`).

use std::path::Path;

use async_trait::async_trait;

use super::StorageError;

/// One log file, possibly cut to its end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogFile {
    /// Its file name (`se-core.log.2026-11-10`).
    pub name: String,
    /// Its text (lossy UTF-8), the last bytes only when it was longer than the limit.
    pub text: String,
}

/// One file of an archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    /// Its path inside the archive (`logs/se-core.log`), `/`-separated.
    pub name: String,
    /// Its content.
    pub bytes: Vec<u8>,
}

/// Reads logs and writes archives.
#[async_trait]
pub trait DiagnosticsFiles: Send + Sync {
    /// The newest `max_files` files of `dir` (newest first), each cut to its last `max_bytes`.
    /// A missing folder gives no file.
    async fn recent_logs(
        &self,
        dir: &Path,
        max_files: usize,
        max_bytes: usize,
    ) -> Result<Vec<LogFile>, StorageError>;

    /// Writes `entries` as a zip archive at `path` (replaced when it exists); returns its size.
    async fn write_archive(
        &self,
        path: &Path,
        entries: Vec<ArchiveEntry>,
    ) -> Result<u64, StorageError>;
}
