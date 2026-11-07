//! [`AppLibraryRepository`]: data of the app itself, shared by every project (`library.db`).

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use se_domain::{Format, ProjectId};

use super::StorageError;

/// The recent projects list and key/value settings of the app (implemented by se-store).
#[async_trait]
pub trait AppLibraryRepository: Send + Sync {
    /// Adds the project at `record.path` to the recent list, or replaces its entry.
    ///
    /// # Errors
    /// IO errors.
    async fn upsert_recent(&self, record: &RecentProjectRecord) -> Result<(), StorageError>;

    /// Every recent project, most recently opened first.
    ///
    /// # Errors
    /// IO errors.
    async fn list_recent(&self) -> Result<Vec<RecentProjectRecord>, StorageError>;

    /// Forgets the project at `path` (nothing happens when it is not listed).
    ///
    /// # Errors
    /// IO errors.
    async fn remove_recent(&self, path: &Path) -> Result<(), StorageError>;

    /// The value of setting `key`, if it was ever set.
    ///
    /// # Errors
    /// IO errors.
    async fn setting(&self, key: &str) -> Result<Option<String>, StorageError>;

    /// Sets `key` to `value`.
    ///
    /// # Errors
    /// IO errors.
    async fn set_setting(&self, key: &str, value: &str) -> Result<(), StorageError>;
}

/// A project of the recent list, as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentProjectRecord {
    /// Canonical absolute path of the project folder (the key of the list).
    pub path: PathBuf,
    /// Id of the project.
    pub id: ProjectId,
    /// Its name when it was last seen.
    pub name: String,
    /// Its default format when it was last seen.
    pub format: Format,
    /// When it was last opened, in milliseconds since the Unix epoch.
    pub last_opened_at: i64,
}
