//! [`Conflict`]: a part of a task's changes that a revert leaves in place, and why.

use std::fmt;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{DomainErrorCode, ItemId};

/// A part of a task's changes that a revert leaves in place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Conflict {
    /// Someone else changed or removed the item after the task's last edit of it: the task's
    /// edits of this item are kept.
    ItemChanged {
        /// The item.
        item: ItemId,
        /// Its revision right after the task's last edit (`null`: the task removed it).
        task_revision: Option<u64>,
        /// Its current revision (`null`: it no longer exists).
        current_revision: Option<u64>,
    },
    /// Reverting would also change or remove items the task never touched (such as the user's
    /// items on a track the task created): that part is kept.
    ForeignItems {
        /// The items that would have been affected.
        items: Vec<ItemId>,
    },
    /// An inverse operation no longer applies to the current document.
    CannotApply {
        /// Why, as a stable code.
        code: DomainErrorCode,
        /// Why, for people and agents.
        message: String,
    },
}

impl fmt::Display for Conflict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ItemChanged {
                item,
                current_revision: None,
                ..
            } => write!(
                formatter,
                "item {item} was removed after the task changed it; the task's edits of it stay"
            ),
            Self::ItemChanged { item, .. } => write!(
                formatter,
                "item {item} was changed after the task changed it; the task's edits of it stay"
            ),
            Self::ForeignItems { items } => {
                let ids: Vec<String> = items.iter().map(ToString::to_string).collect();
                write!(
                    formatter,
                    "reverting would also affect items the task did not touch ({}); that part \
                     stays",
                    ids.join(", ")
                )
            }
            Self::CannotApply { message, .. } => {
                write!(formatter, "part of the task cannot be reverted: {message}")
            }
        }
    }
}
