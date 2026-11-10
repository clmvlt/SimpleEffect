//! [`FsProjectStore`]: one open, locked project folder.

use std::{
    path::Path,
    sync::{Arc, Mutex, PoisonError},
};

use async_trait::async_trait;
use se_app::ports::{
    AnalysisStore, ChangesetWrite, JournalEntry, LoadedSnapshot, ProjectStore, StorageError,
    TaskStore, TranscriptStore,
};
use se_domain::{Changeset, Project};

use crate::{
    fs_analysis_store::FsAnalysisStore,
    fs_task_store::FsTaskStore,
    fs_transcript_store::FsTranscriptStore,
    io_errors::blocking,
    layout::ProjectLayout,
    project_db::ProjectDatabase,
    project_lock::ProjectLock,
    snapshot_file::{read_snapshot, write_snapshot},
};

/// An open project folder: its lock and its database connection.
pub struct FsProjectStore {
    layout: ProjectLayout,
    database: Arc<Mutex<ProjectDatabase>>,
    lock: Mutex<Option<ProjectLock>>,
    /// Revision of the snapshot on disk (read or last written): the journal keeps every entry
    /// after it when the next snapshot is written, so the newest backup plus the journal always
    /// rebuild the document (prompt 29).
    snapshot_revision: Arc<Mutex<Option<u64>>>,
}

impl FsProjectStore {
    /// Wraps a folder whose lock is already held.
    pub fn new(layout: ProjectLayout, database: ProjectDatabase, lock: ProjectLock) -> Self {
        Self {
            layout,
            database: Arc::new(Mutex::new(database)),
            lock: Mutex::new(Some(lock)),
            snapshot_revision: Arc::new(Mutex::new(None)),
        }
    }

    /// Runs `work` with the database on the blocking pool.
    async fn with_database<T: Send + 'static>(
        &self,
        work: impl FnOnce(&ProjectDatabase) -> Result<T, StorageError> + Send + 'static,
    ) -> Result<T, StorageError> {
        let database = Arc::clone(&self.database);
        blocking(self.layout.root(), move || {
            work(&database.lock().unwrap_or_else(PoisonError::into_inner))
        })
        .await
    }
}

/// Locks the snapshot revision, recovering it after a panic elsewhere.
fn lock_revision(revision: &Mutex<Option<u64>>) -> std::sync::MutexGuard<'_, Option<u64>> {
    revision.lock().unwrap_or_else(PoisonError::into_inner)
}

#[async_trait]
impl ProjectStore for FsProjectStore {
    fn path(&self) -> &Path {
        self.layout.root()
    }

    async fn load_snapshot(&self) -> Result<LoadedSnapshot, StorageError> {
        let layout = self.layout.clone();
        let loaded = blocking(self.layout.root(), move || read_snapshot(&layout)).await?;
        *lock_revision(&self.snapshot_revision) = Some(loaded.project.revision);
        Ok(loaded)
    }

    async fn read_journal_since(&self, revision: u64) -> Result<Vec<JournalEntry>, StorageError> {
        self.with_database(move |database| database.entries_since(revision))
            .await
    }

    async fn append_journal(&self, entry: &JournalEntry) -> Result<(), StorageError> {
        let entry = entry.clone();
        self.with_database(move |database| database.append(&entry))
            .await
    }

    async fn append_journal_with_changeset(
        &self,
        entry: &JournalEntry,
        change: &ChangesetWrite,
    ) -> Result<(), StorageError> {
        let (entry, change) = (entry.clone(), change.clone());
        self.with_database(move |database| database.append_with_changeset(&entry, &change))
            .await
    }

    async fn load_changesets(&self) -> Result<Vec<Changeset>, StorageError> {
        self.with_database(ProjectDatabase::open_changesets).await
    }

    async fn save_snapshot(&self, project: &Project) -> Result<(), StorageError> {
        let (layout, project) = (self.layout.clone(), project.clone());
        let revision = Arc::clone(&self.snapshot_revision);
        self.with_database(move |database| {
            write_snapshot(&layout, &project)?;
            // The snapshot just replaced became `.bak.1`: keep the entries after it.
            let previous = lock_revision(&revision).replace(project.revision);
            database.truncate_through(previous.unwrap_or(0).min(project.revision))
        })
        .await
    }

    fn analyses(&self) -> Arc<dyn AnalysisStore> {
        Arc::new(FsAnalysisStore::new(
            self.layout.root().to_path_buf(),
            Arc::clone(&self.database),
        ))
    }

    fn tasks(&self) -> Arc<dyn TaskStore> {
        Arc::new(FsTaskStore::new(
            self.layout.root().to_path_buf(),
            Arc::clone(&self.database),
        ))
    }

    fn transcripts(&self) -> Arc<dyn TranscriptStore> {
        Arc::new(FsTranscriptStore::new(
            self.layout.root().to_path_buf(),
            Arc::clone(&self.database),
        ))
    }

    async fn close(&self) -> Result<(), StorageError> {
        let lock = self
            .lock
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        match lock {
            Some(lock) => lock.release(&self.layout.lock()),
            None => Ok(()),
        }
    }
}
