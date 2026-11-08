//! The `tasks` and `task_events` tables: AI tasks (one JSON row each, replaced as they change)
//! and their append-only event logs.

use rusqlite::{OptionalExtension, params};
use se_app::ports::StorageError;
use se_domain::TaskId;
use se_protocol::{AgentEvent, Task, TaskEventRecord};

use super::ProjectDatabase;
use crate::io_errors::sqlite_error;

/// Version of the JSON stored in `tasks.payload` (`se_protocol::Task`) and `task_events.payload`
/// (`se_protocol::AgentEvent`). Rows of a newer version are skipped; bump it when a format
/// changes incompatibly (new optional fields need no bump).
const TASK_FORMAT_VERSION: u32 = 1;

impl ProjectDatabase {
    /// Saves `task`, replacing its previous row.
    ///
    /// # Errors
    /// IO errors.
    pub fn save_task(&self, task: &Task) -> Result<(), StorageError> {
        let payload = self.json_of("serialise a task for", task)?;
        let status = self.json_of("serialise a task status for", &task.status)?;
        let status_kind = serde_json::from_str::<serde_json::Value>(&status)
            .ok()
            .and_then(|value| value["kind"].as_str().map(str::to_owned))
            .unwrap_or_default();
        self.connection
            .execute(
                "INSERT INTO tasks (id, created_at, status, payload, format_version)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT (id) DO UPDATE SET status = excluded.status,
                 payload = excluded.payload, format_version = excluded.format_version",
                params![
                    task.id.to_string(),
                    task.created_at,
                    status_kind,
                    payload,
                    TASK_FORMAT_VERSION
                ],
            )
            .map(drop)
            .map_err(|error| sqlite_error("save an AI task in", &self.path, &error))
    }

    /// Every task, oldest first; rows of a newer format are skipped.
    ///
    /// # Errors
    /// [`StorageError::Corrupted`] for an unreadable row; IO errors.
    pub fn load_tasks(&self) -> Result<Vec<Task>, StorageError> {
        let error =
            |error: rusqlite::Error| sqlite_error("read the AI tasks of", &self.path, &error);
        let mut statement = self
            .connection
            .prepare("SELECT id, payload, format_version FROM tasks ORDER BY created_at, id")
            .map_err(error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, u32>(2)?,
                ))
            })
            .map_err(error)?;
        let mut tasks = Vec::new();
        for row in rows {
            let (id, payload, format_version) = row.map_err(error)?;
            if format_version > TASK_FORMAT_VERSION {
                tracing::warn!(task = %id, format_version, "AI task written by a newer version skipped");
                continue;
            }
            tasks.push(self.parse_json(&payload, || format!("AI task {id}"))?);
        }
        Ok(tasks)
    }

    /// Appends `event` to the task's log, numbered after its last entry.
    ///
    /// # Errors
    /// IO errors (a task that was never saved among them).
    pub fn append_task_event(
        &self,
        task_id: TaskId,
        run: u32,
        at: i64,
        event: &AgentEvent,
    ) -> Result<TaskEventRecord, StorageError> {
        let payload = self.json_of("serialise an AI task event for", event)?;
        let sequence: u32 = self
            .connection
            .query_row(
                "INSERT INTO task_events (task_id, sequence, at, payload, run, format_version)
                 VALUES (?1, (SELECT COALESCE(MAX(sequence) + 1, 0) FROM task_events
                 WHERE task_id = ?1), ?2, ?3, ?4, ?5)
                 RETURNING sequence",
                params![task_id.to_string(), at, payload, run, TASK_FORMAT_VERSION],
                |row| row.get(0),
            )
            .map_err(|error| sqlite_error("append an AI task event to", &self.path, &error))?;
        Ok(TaskEventRecord {
            sequence,
            run,
            at,
            event: event.clone(),
        })
    }

    /// The task's log in order; rows of a newer format are skipped.
    ///
    /// # Errors
    /// [`StorageError::Corrupted`] for an unreadable row; IO errors.
    pub fn load_task_events(&self, task_id: TaskId) -> Result<Vec<TaskEventRecord>, StorageError> {
        let error =
            |error: rusqlite::Error| sqlite_error("read the AI task events of", &self.path, &error);
        let mut statement = self
            .connection
            .prepare(
                "SELECT sequence, run, at, payload, format_version FROM task_events
                 WHERE task_id = ?1 ORDER BY sequence",
            )
            .map_err(error)?;
        let rows = statement
            .query_map([task_id.to_string()], |row| {
                Ok((
                    row.get::<_, u32>(0)?,
                    row.get::<_, u32>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, u32>(4)?,
                ))
            })
            .map_err(error)?;
        let mut events = Vec::new();
        for row in rows {
            let (sequence, run, at, payload, format_version) = row.map_err(error)?;
            if format_version > TASK_FORMAT_VERSION {
                continue;
            }
            let event = self.parse_json(&payload, || {
                format!("event {sequence} of AI task {task_id}")
            })?;
            events.push(TaskEventRecord {
                sequence,
                run,
                at,
                event,
            });
        }
        Ok(events)
    }

    /// Number of stored tasks (tests and diagnostics).
    ///
    /// # Errors
    /// IO errors.
    pub fn task_count(&self) -> Result<u64, StorageError> {
        let count: Option<i64> = self
            .connection
            .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
            .optional()
            .map_err(|error| sqlite_error("count the AI tasks of", &self.path, &error))?;
        Ok(count
            .and_then(|count| u64::try_from(count).ok())
            .unwrap_or(0))
    }

    pub(super) fn json_of<T: serde::Serialize>(
        &self,
        action: &'static str,
        value: &T,
    ) -> Result<String, StorageError> {
        serde_json::to_string(value).map_err(|error| StorageError::Io {
            action,
            path: self.path.clone(),
            reason: error.to_string(),
        })
    }

    pub(super) fn parse_json<T: serde::de::DeserializeOwned>(
        &self,
        payload: &str,
        what: impl FnOnce() -> String,
    ) -> Result<T, StorageError> {
        serde_json::from_str(payload).map_err(|error| StorageError::Corrupted {
            path: self.path.clone(),
            reason: format!("{} is unreadable: {error}", what()),
        })
    }
}

#[cfg(test)]
mod tests {
    use se_domain::{ProjectId, SequenceId};
    use se_protocol::{AgentFailureKind, ProviderId, RunKind, TaskRun, TaskScope, TaskStatus};

    use super::*;

    fn task(index: u64, created_at: i64) -> Task {
        Task {
            id: TaskId::from_test_index(index),
            project_id: ProjectId::from_test_index(1),
            prompt: format!("Task {index}"),
            scope: TaskScope {
                sequence_id: SequenceId::from_test_index(2),
                range: None,
                tracks: None,
            },
            provider: ProviderId("claude-code".to_owned()),
            model: Some("opus".to_owned()),
            effort: None,
            status: TaskStatus::Queued,
            agent_session_id: None,
            changeset_id: None,
            usage: se_protocol::TaskUsage::default(),
            created_at,
            updated_at: created_at,
            runs: vec![TaskRun {
                run_kind: RunKind::Initial,
                prompt: format!("Task {index}"),
                status: TaskStatus::Queued,
                queued_at: created_at,
                started_at: None,
                finished_at: None,
            }],
        }
    }

    fn database() -> (tempfile::TempDir, ProjectDatabase) {
        let dir = tempfile::tempdir().unwrap();
        let database = ProjectDatabase::open(&dir.path().join("project.db")).unwrap();
        (dir, database)
    }

    #[test]
    fn tasks_are_saved_replaced_and_listed_oldest_first() {
        let (_dir, database) = database();
        database.save_task(&task(2, 20)).unwrap();
        let mut first = task(1, 10);
        database.save_task(&first).unwrap();
        first.status = TaskStatus::Failed {
            failure: AgentFailureKind::Crashed,
            message: "boom".to_owned(),
        };
        database.save_task(&first).unwrap();
        let tasks = database.load_tasks().unwrap();
        assert_eq!(tasks, [first, task(2, 20)]);
        assert_eq!(database.task_count().unwrap(), 2);
    }

    #[test]
    fn events_are_numbered_per_task_and_read_back_in_order() {
        let (_dir, database) = database();
        let (one, two) = (task(1, 10), task(2, 20));
        database.save_task(&one).unwrap();
        database.save_task(&two).unwrap();
        let text = |text: &str| AgentEvent::AssistantText {
            text: text.to_owned(),
        };
        database
            .append_task_event(one.id, 0, 11, &text("a"))
            .unwrap();
        database
            .append_task_event(two.id, 0, 21, &text("x"))
            .unwrap();
        let record = database
            .append_task_event(one.id, 1, 12, &text("b"))
            .unwrap();
        assert_eq!((record.sequence, record.run), (1, 1));
        let events = database.load_task_events(one.id).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[1], record);
        assert_eq!(
            database
                .load_task_events(TaskId::from_test_index(9))
                .unwrap(),
            []
        );
    }

    #[test]
    fn events_of_unknown_tasks_are_refused() {
        let (_dir, database) = database();
        let event = AgentEvent::Completed {
            summary: "done".to_owned(),
        };
        assert!(
            database
                .append_task_event(TaskId::from_test_index(3), 0, 1, &event)
                .is_err()
        );
    }
}
