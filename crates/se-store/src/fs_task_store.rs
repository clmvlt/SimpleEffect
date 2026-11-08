//! [`FsTaskStore`]: the `TaskStore` port over an open project's `project.db`.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex, PoisonError},
};

use async_trait::async_trait;
use se_app::ports::{StorageError, TaskStore};
use se_domain::TaskId;
use se_protocol::{AgentEvent, Task, TaskEventRecord};

use crate::{io_errors::blocking, project_db::ProjectDatabase};

/// The AI tasks of one open project, sharing its database connection with its store.
pub struct FsTaskStore {
    root: PathBuf,
    database: Arc<Mutex<ProjectDatabase>>,
}

impl FsTaskStore {
    /// The tasks of the project at `root`, read and written through `database`.
    pub fn new(root: PathBuf, database: Arc<Mutex<ProjectDatabase>>) -> Self {
        Self { root, database }
    }

    /// Runs `work` with the database on the blocking pool.
    async fn with_database<T: Send + 'static>(
        &self,
        work: impl FnOnce(&ProjectDatabase) -> Result<T, StorageError> + Send + 'static,
    ) -> Result<T, StorageError> {
        let database = Arc::clone(&self.database);
        blocking(&self.root, move || {
            work(&database.lock().unwrap_or_else(PoisonError::into_inner))
        })
        .await
    }
}

#[async_trait]
impl TaskStore for FsTaskStore {
    async fn save_task(&self, task: &Task) -> Result<(), StorageError> {
        let task = task.clone();
        self.with_database(move |database| database.save_task(&task))
            .await
    }

    async fn load_tasks(&self) -> Result<Vec<Task>, StorageError> {
        self.with_database(ProjectDatabase::load_tasks).await
    }

    async fn append_task_event(
        &self,
        task_id: TaskId,
        run: u32,
        at: i64,
        event: &AgentEvent,
    ) -> Result<TaskEventRecord, StorageError> {
        let event = event.clone();
        self.with_database(move |database| database.append_task_event(task_id, run, at, &event))
            .await
    }

    async fn load_task_events(
        &self,
        task_id: TaskId,
    ) -> Result<Vec<TaskEventRecord>, StorageError> {
        self.with_database(move |database| database.load_task_events(task_id))
            .await
    }
}
