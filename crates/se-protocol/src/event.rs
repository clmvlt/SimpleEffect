//! Events pushed by `se-core` to its clients.

use se_domain::ProjectId;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{ProjectHistory, ProjectPatch, ProjectSession};

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
