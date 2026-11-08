//! Fake of [`TaskStore`]: the tasks of a fake project folder.

use std::path::PathBuf;

use async_trait::async_trait;
use se_domain::TaskId;
use se_protocol::{AgentEvent, Task, TaskEventRecord};

use super::InMemoryProjectRepository;
use crate::ports::{StorageError, TaskStore};

/// The tasks and event logs of one fake folder.
pub(super) struct InMemoryTaskStore {
    repository: InMemoryProjectRepository,
    path: PathBuf,
}

impl InMemoryTaskStore {
    pub(super) fn new(repository: InMemoryProjectRepository, path: PathBuf) -> Self {
        Self { repository, path }
    }
}

#[async_trait]
impl TaskStore for InMemoryTaskStore {
    async fn save_task(&self, task: &Task) -> Result<(), StorageError> {
        self.repository.with_folder(&self.path, |folder| {
            match folder.tasks.iter_mut().find(|stored| stored.id == task.id) {
                Some(stored) => *stored = task.clone(),
                None => folder.tasks.push(task.clone()),
            }
            Ok(())
        })
    }

    async fn load_tasks(&self) -> Result<Vec<Task>, StorageError> {
        self.repository
            .with_folder(&self.path, |folder| Ok(folder.tasks.clone()))
    }

    async fn append_task_event(
        &self,
        task_id: TaskId,
        run: u32,
        at: i64,
        event: &AgentEvent,
    ) -> Result<TaskEventRecord, StorageError> {
        self.repository.with_folder(&self.path, |folder| {
            let log = folder.task_events.entry(task_id).or_default();
            let record = TaskEventRecord {
                sequence: u32::try_from(log.len()).unwrap_or(u32::MAX),
                run,
                at,
                event: event.clone(),
            };
            log.push(record.clone());
            Ok(record)
        })
    }

    async fn load_task_events(
        &self,
        task_id: TaskId,
    ) -> Result<Vec<TaskEventRecord>, StorageError> {
        self.repository.with_folder(&self.path, |folder| {
            Ok(folder
                .task_events
                .get(&task_id)
                .cloned()
                .unwrap_or_default())
        })
    }
}
