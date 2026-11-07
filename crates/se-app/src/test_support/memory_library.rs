//! Fake of [`AppLibraryRepository`].

use std::{collections::BTreeMap, path::Path, sync::Mutex};

use async_trait::async_trait;

use super::lock;
use crate::ports::{AppLibraryRepository, RecentProjectRecord, StorageError};

/// Recent projects and settings in memory.
#[derive(Debug, Default)]
pub struct InMemoryAppLibrary {
    recent: Mutex<Vec<RecentProjectRecord>>,
    settings: Mutex<BTreeMap<String, String>>,
}

#[async_trait]
impl AppLibraryRepository for InMemoryAppLibrary {
    async fn upsert_recent(&self, record: &RecentProjectRecord) -> Result<(), StorageError> {
        let mut recent = lock(&self.recent);
        recent.retain(|existing| existing.path != record.path);
        recent.push(record.clone());
        Ok(())
    }

    async fn list_recent(&self) -> Result<Vec<RecentProjectRecord>, StorageError> {
        let mut recent = lock(&self.recent).clone();
        recent.sort_by_key(|record| std::cmp::Reverse(record.last_opened_at));
        Ok(recent)
    }

    async fn remove_recent(&self, path: &Path) -> Result<(), StorageError> {
        lock(&self.recent).retain(|existing| existing.path != path);
        Ok(())
    }

    async fn setting(&self, key: &str) -> Result<Option<String>, StorageError> {
        Ok(lock(&self.settings).get(key).cloned())
    }

    async fn set_setting(&self, key: &str, value: &str) -> Result<(), StorageError> {
        lock(&self.settings).insert(key.to_owned(), value.to_owned());
        Ok(())
    }
}
