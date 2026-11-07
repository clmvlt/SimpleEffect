//! [`FsProjectStore`]: one open, locked project folder.

use std::{
    path::Path,
    sync::{Arc, Mutex, PoisonError},
};

use async_trait::async_trait;
use se_app::ports::{AnalysisStore, JournalEntry, ProjectStore, StorageError, TaskStore};
use se_domain::Project;

use crate::{
    fs_analysis_store::FsAnalysisStore,
    fs_task_store::FsTaskStore,
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
}

impl FsProjectStore {
    /// Wraps a folder whose lock is already held.
    pub fn new(layout: ProjectLayout, database: ProjectDatabase, lock: ProjectLock) -> Self {
        Self {
            layout,
            database: Arc::new(Mutex::new(database)),
            lock: Mutex::new(Some(lock)),
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

#[async_trait]
impl ProjectStore for FsProjectStore {
    fn path(&self) -> &Path {
        self.layout.root()
    }

    async fn load_snapshot(&self) -> Result<Project, StorageError> {
        let layout = self.layout.clone();
        blocking(self.layout.root(), move || read_snapshot(&layout)).await
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

    async fn save_snapshot(&self, project: &Project) -> Result<(), StorageError> {
        let (layout, project) = (self.layout.clone(), project.clone());
        self.with_database(move |database| {
            write_snapshot(&layout, &project)?;
            database.truncate_through(project.revision)
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
