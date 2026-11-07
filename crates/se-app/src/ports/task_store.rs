//! [`TaskStore`]: the AI tasks of a project and their event logs, kept in its database (`tasks`
//! and `task_events` tables of `project.db`, implemented by se-store).

use async_trait::async_trait;
use se_domain::TaskId;
use se_protocol::{AgentEvent, Task, TaskEventRecord};

use super::StorageError;

/// AI tasks of one open project. Not document state: written by the task service, outside the
/// project's actor.
#[async_trait]
pub trait TaskStore: Send + Sync {
    /// Saves `task`, replacing its previous version.
    ///
    /// # Errors
    /// IO errors.
    async fn save_task(&self, task: &Task) -> Result<(), StorageError>;

    /// Every stored task, oldest first. Tasks written by a newer version are skipped.
    ///
    /// # Errors
    /// [`StorageError::Corrupted`] for an unreadable row; IO errors.
    async fn load_tasks(&self) -> Result<Vec<Task>, StorageError>;

    /// Appends `event` (produced by run number `run`, received at `at`) to the task's log and
    /// returns the stored record, numbered after the last one.
    ///
    /// # Errors
    /// IO errors (an unknown task among them).
    async fn append_task_event(
        &self,
        task_id: TaskId,
        run: u32,
        at: i64,
        event: &AgentEvent,
    ) -> Result<TaskEventRecord, StorageError>;

    /// The task's log, in order.
    ///
    /// # Errors
    /// [`StorageError::Corrupted`] for an unreadable row; IO errors.
    async fn load_task_events(&self, task_id: TaskId)
    -> Result<Vec<TaskEventRecord>, StorageError>;
}
