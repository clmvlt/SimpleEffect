//! AI task DTOs (`docs/architecture.md` §9.3, ADR-0027): a task is a user request run by an AI
//! agent in the background, possibly several times (retries, follow-ups), with its event log.

use se_domain::{ChangesetId, Conflict, ItemId, ProjectId, SequenceId, TaskId, TimeRange, TrackId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{AgentEvent, AgentFailureKind, EffortLevel, ProviderId};

/// What part of the project a task may change (checked by MCP write tools, prompt 19).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TaskScope {
    /// The sequence the task works on.
    pub sequence_id: SequenceId,
    /// The time range it may change; `null` for the whole sequence.
    pub range: Option<TimeRange>,
    /// The tracks it may change; `null` for every track.
    pub tracks: Option<Vec<TrackId>>,
}

/// Where a task (or one of its runs) stands: `Queued → Running → Completed | Failed | Cancelled`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TaskStatus {
    /// Waiting for a free slot (the number of tasks running at once is limited).
    Queued,
    /// The agent is working.
    Running,
    /// The agent finished.
    Completed,
    /// The run failed.
    Failed {
        /// Why.
        failure: AgentFailureKind,
        /// What happened, actionable.
        message: String,
    },
    /// Cancelled by the user or because its project closed.
    Cancelled,
}

/// Why a run was started.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum RunKind {
    /// The task's first run (a new agent session).
    Initial,
    /// A new attempt of the previous run, with the same prompt.
    Retry,
    /// A new prompt resuming the agent's session.
    FollowUp,
}

/// One execution of the agent for a task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TaskRun {
    /// Why it was started.
    pub run_kind: RunKind,
    /// The prompt it sent.
    pub prompt: String,
    /// Where it stands.
    pub status: TaskStatus,
    /// When it was queued (milliseconds since the Unix epoch).
    pub queued_at: i64,
    /// When the agent was started.
    pub started_at: Option<i64>,
    /// When it ended.
    pub finished_at: Option<i64>,
}

/// An AI task as clients see it (its events come apart: `getTaskEvents`, `taskEventAppended`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    /// Stable id.
    pub id: TaskId,
    /// The project it belongs to.
    pub project_id: ProjectId,
    /// The user's request (the first run's prompt).
    pub prompt: String,
    /// What it may change.
    pub scope: TaskScope,
    /// The provider running it.
    pub provider: ProviderId,
    /// The model asked for; `null` lets the CLI decide.
    pub model: Option<String>,
    /// The effort asked for; `null` lets the CLI decide.
    pub effort: Option<EffortLevel>,
    /// Status of its latest run.
    pub status: TaskStatus,
    /// The agent session its runs share, once the CLI announced it.
    pub agent_session_id: Option<String>,
    /// When it was submitted (milliseconds since the Unix epoch).
    pub created_at: i64,
    /// When it last changed.
    pub updated_at: i64,
    /// Its runs, oldest first (never empty).
    pub runs: Vec<TaskRun>,
    /// The changeset holding its edits since it started or was last reverted (`null` while it
    /// changed nothing). Created by its first edit (prompt 19).
    #[serde(default)]
    pub changeset_id: Option<ChangesetId>,
}

/// What reverting an AI task did ([`crate::Command::RevertTask`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TaskRevertReport {
    /// The task.
    pub task_id: TaskId,
    /// Whether anything was undone: the revert is then one user transaction (undoable), and
    /// the task's changeset is closed.
    pub reverted: bool,
    /// Items the revert restored, removed or put back.
    pub reverted_items: Vec<ItemId>,
    /// What stays as the task left it, and why (items changed by someone else since, edits that
    /// no longer apply).
    pub conflicts: Vec<Conflict>,
}

/// One entry of a task's event log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TaskEventRecord {
    /// Position in the task's log, from 0.
    pub sequence: u32,
    /// Index of the run that produced it in [`Task::runs`].
    pub run: u32,
    /// When it was received (milliseconds since the Unix epoch).
    pub at: i64,
    /// What the agent reported.
    pub event: AgentEvent,
}

/// Arguments of [`crate::Command::SubmitTask`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SubmitTaskRequest {
    /// The open project.
    pub project_id: ProjectId,
    /// What the agent must do.
    pub prompt: String,
    /// What it may change.
    pub scope: TaskScope,
    /// The provider; omitted for the one chosen in the settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub provider: Option<ProviderId>,
    /// The model; omitted for the settings' default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub model: Option<String>,
    /// The effort; omitted for the settings' default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub effort: Option<EffortLevel>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_are_tagged_by_kind() {
        assert_eq!(
            serde_json::to_string(&TaskStatus::Queued).unwrap(),
            r#"{"kind":"queued"}"#
        );
        let failed = TaskStatus::Failed {
            failure: AgentFailureKind::Interrupted,
            message: "the app stopped".to_owned(),
        };
        assert_eq!(
            serde_json::to_string(&failed).unwrap(),
            r#"{"kind":"failed","failure":"interrupted","message":"the app stopped"}"#
        );
    }

    #[test]
    fn submit_requests_may_omit_their_options() {
        let json = r#"{"projectId":"00000000-0001-7000-8000-000000000000","prompt":"Cut the silences","scope":{"sequenceId":"00000000-0002-7000-8000-000000000000","range":null,"tracks":null}}"#;
        let request: SubmitTaskRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.provider, None);
        assert_eq!(request.effort, None);
        assert_eq!(serde_json::to_string(&request).unwrap(), json);
    }

    #[test]
    fn event_records_keep_the_event_kind() {
        let record = TaskEventRecord {
            sequence: 0,
            run: 0,
            at: 5,
            event: AgentEvent::AssistantText {
                text: "Done".to_owned(),
            },
        };
        let json: serde_json::Value = serde_json::to_value(&record).unwrap();
        assert_eq!(json["event"]["kind"], "assistantText");
    }
}
