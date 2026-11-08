//! Events pushed by `se-core` to its clients.

use se_domain::{ProjectId, TaskId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    AiStatus, FrameRenderRequest, JobProgress, JobSnapshot, ProjectHistory, ProjectPatch,
    ProjectSession, Task, TaskEventRecord,
};

/// An event pushed inside a [`crate::EventEnvelope`], tagged by `kind` (camelCase).
///
/// Named `CoreEvent` rather than `Event` so the generated TypeScript type does not shadow the DOM
/// `Event` type in the renderer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CoreEvent {
    /// First message of every new connection: the core is ready to execute commands.
    CoreReady {
        /// Version of the running `se-core` binary.
        version: String,
    },
    /// The core is stopping (the app is quitting or restarting it): the connection closes next.
    CoreShuttingDown,
    /// This connection fell behind and the core dropped events for it: refetch any state built
    /// from events (e.g. request a fresh project snapshot).
    ResyncRequired {
        /// Number of events that were dropped.
        missed_events: u64,
    },
    /// A project was opened or created: its whole session.
    ProjectOpened(ProjectSession),
    /// A commit changed an open project (an edit, an undo or a redo): whole entities to put in
    /// place and ids to remove. Patches of one project arrive in revision order.
    ProjectPatched(ProjectPatch),
    /// A project was saved and closed.
    ProjectClosed {
        /// The project.
        project_id: ProjectId,
    },
    /// What undo and redo would do changed for an open project.
    HistoryChanged(ProjectHistory),
    // Job events carry the snapshot in a `job` field: flattened, its own `kind` (the job kind)
    // would collide with the event tag.
    /// A background job was queued (again, with `attempt: 2`, for its automatic retry).
    JobQueued {
        /// The job.
        job: JobSnapshot,
    },
    /// A job started or made progress.
    JobProgress(JobProgress),
    /// A job finished; its outputs are served by `/media`.
    JobCompleted {
        /// The job.
        job: JobSnapshot,
    },
    /// A job failed after its retry.
    JobFailed {
        /// The job.
        job: JobSnapshot,
    },
    /// A job was cancelled (by the user or because its project closed).
    JobCancelled {
        /// The job.
        job: JobSnapshot,
    },
    /// An AI task was submitted or changed (status, run added, agent session announced).
    TaskUpdated {
        /// The task as it is now.
        task: Task,
    },
    /// An AI task's agent reported something (appended to the task's log).
    TaskEventAppended {
        /// The task's project.
        project_id: ProjectId,
        /// The task.
        task_id: TaskId,
        /// The new entry of its log.
        event: TaskEventRecord,
    },
    /// What is known about the AI provider changed (installed, logged in or out…).
    AiStatusChanged(AiStatus),
    /// An AI agent asked for frames of a sequence: the renderer showing that project draws them
    /// and uploads one JPEG per time to `/media/renders/{requestId}/{index}`, or answers
    /// [`crate::Command::ReportFrameRenderFailure`] (ADR-0028).
    RenderFramesRequested(FrameRenderRequest),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struct_variant_fields_are_camel_case() {
        let json = serde_json::to_string(&CoreEvent::ResyncRequired { missed_events: 3 }).unwrap();
        assert_eq!(json, r#"{"kind":"resyncRequired","missedEvents":3}"#);
    }

    #[test]
    fn project_closed_names_its_project() {
        let json = serde_json::to_string(&CoreEvent::ProjectClosed {
            project_id: ProjectId::from_test_index(1),
        })
        .unwrap();
        assert_eq!(
            json,
            r#"{"kind":"projectClosed","projectId":"00000000-0001-7000-8000-000000000000"}"#
        );
    }

    #[test]
    fn job_events_keep_their_own_kind() {
        let job = JobSnapshot {
            id: se_domain::JobId::from_test_index(1),
            project_id: ProjectId::from_test_index(1),
            asset_id: None,
            kind: crate::JobKind::Thumbnails,
            state: crate::JobState::Completed,
            attempt: 1,
        };
        let json: serde_json::Value =
            serde_json::to_value(CoreEvent::JobCompleted { job }).unwrap();
        assert_eq!(json["kind"], "jobCompleted");
        assert_eq!(json["job"]["kind"], "thumbnails");
    }

    #[test]
    fn every_event_round_trips() {
        let events = [
            CoreEvent::CoreReady {
                version: "0.1.0".to_owned(),
            },
            CoreEvent::CoreShuttingDown,
            CoreEvent::ResyncRequired { missed_events: 12 },
            CoreEvent::ProjectClosed {
                project_id: ProjectId::from_test_index(1),
            },
            CoreEvent::HistoryChanged(ProjectHistory {
                project_id: ProjectId::from_test_index(1),
                history: crate::HistoryState {
                    can_undo: true,
                    can_redo: false,
                    undo_label: Some("Create sequence".to_owned()),
                    redo_label: None,
                },
            }),
        ];
        for event in events {
            let json = serde_json::to_string(&event).unwrap();
            assert_eq!(serde_json::from_str::<CoreEvent>(&json).unwrap(), event);
        }
    }
}
