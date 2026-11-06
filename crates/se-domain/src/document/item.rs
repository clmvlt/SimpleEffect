//! [`Item`]: anything placed on a track, and its [`Origin`].

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::ItemKind;
use crate::{
    ids::{ItemId, LinkGroupId, TaskId},
    time::TimeRange,
};

/// Something placed on a track over a time range: a media clip, a text, a template…
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    /// Stable id, unique in the project.
    pub id: ItemId,
    /// Bumped on every change to this item; agents send it back for optimistic concurrency.
    pub revision: u64,
    /// Who created the item, kept for traceability and changeset reverts.
    pub origin: Origin,
    /// Where the item sits in its sequence; never empty, never before zero.
    pub timeline_range: TimeRange,
    /// Items sharing a link group move and trim together (a clip's picture and sound).
    pub link_group: Option<LinkGroupId>,
    /// What the item shows or plays.
    pub kind: ItemKind,
}

/// Who created an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Origin {
    /// The user, through the UI.
    User,
    /// An AI agent working on a task.
    AiTask {
        /// The task whose changeset contains the creation.
        task_id: TaskId,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origins_are_tagged_by_kind() {
        assert_eq!(
            serde_json::to_string(&Origin::User).unwrap(),
            r#"{"kind":"user"}"#
        );
        let origin = Origin::AiTask {
            task_id: TaskId::from_test_index(1),
        };
        assert_eq!(
            serde_json::to_string(&origin).unwrap(),
            r#"{"kind":"aiTask","taskId":"00000000-0001-7000-8000-000000000000"}"#
        );
    }
}
