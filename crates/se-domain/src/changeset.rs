//! [`Changeset`]: every transaction of one AI task, revertable as a unit even after other edits
//! (ADR-0005). [`plan_revert`] computes what can be undone and what conflicts.

mod conflict;
mod revert;

pub use conflict::Conflict;
pub use revert::{RevertPlan, plan_revert};

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    Actor, ChangesetId, Committed, DomainError, ItemId, Project, TaskId, Transaction,
    operation::lookup::all_items,
};

/// The transactions an AI task committed, in order, with what undoes each of them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Changeset {
    /// Stable id.
    pub id: ChangesetId,
    /// The task whose transactions these are.
    pub task_id: TaskId,
    /// The task's transactions, oldest first.
    pub entries: Vec<ChangesetEntry>,
}

/// One committed transaction of a task.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ChangesetEntry {
    /// The transaction as committed.
    pub transaction: Transaction,
    /// The transaction undoing it, computed at commit time.
    pub inverse: Transaction,
    /// Revision of each item the transaction touched, right after it (`null`: it removed the
    /// item). A later different revision means someone else changed the item since.
    pub touched_revisions: BTreeMap<ItemId, Option<u64>>,
}

impl Changeset {
    /// An empty changeset for `task_id`.
    #[must_use]
    pub const fn new(id: ChangesetId, task_id: TaskId) -> Self {
        Self {
            id,
            task_id,
            entries: Vec::new(),
        }
    }

    /// Appends a transaction of the task, just committed with outcome `committed`.
    ///
    /// # Errors
    /// [`DomainError::WrongChangeset`] if the transaction is not by this changeset's task.
    pub fn record(
        &mut self,
        transaction: &Transaction,
        committed: &Committed,
    ) -> Result<(), DomainError> {
        let by_this_task = matches!(
            transaction.actor,
            Actor::AiTask { task_id } if task_id == self.task_id
        );
        if !by_this_task {
            return Err(DomainError::WrongChangeset { task: self.task_id });
        }
        self.entries.push(ChangesetEntry {
            transaction: transaction.clone(),
            inverse: committed.inverse.clone(),
            touched_revisions: revisions_of(&committed.project, committed.touched.items.iter()),
        });
        Ok(())
    }
}

/// The revision of each of `items` in `project` (`None` when absent).
fn revisions_of<'a>(
    project: &Project,
    items: impl Iterator<Item = &'a ItemId>,
) -> BTreeMap<ItemId, Option<u64>> {
    let mut revisions: BTreeMap<ItemId, Option<u64>> = items.map(|id| (*id, None)).collect();
    for item in all_items(project) {
        if let Some(revision) = revisions.get_mut(&item.id) {
            *revision = Some(item.revision);
        }
    }
    revisions
}
