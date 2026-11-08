//! Edits of AI tasks inside the actor: each one is recorded in its task's open changeset
//! (journal and changeset entry in one atomic write), and a task's changeset can be reverted as
//! one user transaction (ADR-0005, ADR-0016, ADR-0029).

use std::collections::{BTreeSet, HashMap};

use se_domain::{Actor, Changeset, ItemId, Project, TaskId, Transaction, commit, plan_revert};
use se_protocol::{CoreEvent, TaskRevertReport};

use super::{AiCommit, ChangesetItems, runner::ProjectActor};
use crate::{AppError, new_ids::NewIds, ports::ChangesetWrite};

/// The open changesets of a project's AI tasks, one per task at most.
pub(super) struct OpenChangesets {
    by_task: HashMap<TaskId, Changeset>,
}

impl OpenChangesets {
    /// The changesets loaded from the project's database.
    pub(super) fn new(changesets: Vec<Changeset>) -> Self {
        Self {
            by_task: changesets
                .into_iter()
                .map(|changeset| (changeset.task_id, changeset))
                .collect(),
        }
    }

    /// The open changeset of `task_id`.
    pub(super) fn get(&self, task_id: TaskId) -> Option<&Changeset> {
        self.by_task.get(&task_id)
    }
}

/// The items `changeset` added or changed that still exist in `project`, in id order.
fn present_items(changeset: &Changeset, project: &Project) -> Vec<ItemId> {
    let touched: BTreeSet<ItemId> = changeset
        .entries
        .iter()
        .flat_map(|entry| entry.touched_revisions.keys().copied())
        .collect();
    let present: BTreeSet<ItemId> = project.items().map(|item| item.id).collect();
    touched.intersection(&present).copied().collect()
}

impl ProjectActor {
    /// Commits an AI task's edit and records it in the task's changeset.
    pub(super) async fn apply_ai(
        &mut self,
        transaction: Transaction,
    ) -> Result<AiCommit, AppError> {
        let Actor::AiTask { task_id } = transaction.actor else {
            return Err(AppError::Internal(format!(
                "transaction « {} » is not by an AI task",
                transaction.label
            )));
        };
        let committed = commit(&self.project, &transaction)?;
        let (mut changeset, created) = match self.changesets.get(task_id) {
            Some(open) => (open.clone(), false),
            None => (
                Changeset::new(NewIds(self.deps.ids.as_ref()).changeset()?, task_id),
                true,
            ),
        };
        changeset.record(&transaction, &committed)?;
        let entry = changeset
            .entries
            .last()
            .cloned()
            .map(Box::new)
            .ok_or_else(|| AppError::Internal("a recorded changeset has no entry".to_owned()))?;
        let change = ChangesetWrite::Record {
            changeset_id: changeset.id,
            task_id,
            created_at: self.deps.clock.now_unix_millis(),
            entry,
        };
        self.journal_with_changeset(&transaction, &committed, &change)
            .await?;
        let touched_items = committed.touched.items.iter().copied().collect();
        let changeset_id = changeset.id;
        self.changesets.by_task.insert(task_id, changeset);
        let revision = self.install(committed);
        self.publish_changeset(task_id);
        Ok(AiCommit {
            revision,
            project: std::sync::Arc::clone(&self.project),
            touched_items,
            changeset_id,
            changeset_created: created,
        })
    }

    /// Reverts the open changeset of `task_id` (see [`super::ProjectActorHandle::revert_task`]).
    pub(super) async fn revert_task(
        &mut self,
        task_id: TaskId,
        label: String,
    ) -> Result<TaskRevertReport, AppError> {
        let changeset = self
            .changesets
            .get(task_id)
            .ok_or(AppError::NothingToRevert { task_id })?;
        let changeset_id = changeset.id;
        let plan = plan_revert(&self.project, changeset);
        let conflicts = plan.conflicts.clone();
        if plan.ops.is_empty() {
            return Ok(TaskRevertReport {
                task_id,
                reverted: false,
                reverted_items: Vec::new(),
                conflicts,
            });
        }
        let id = NewIds(self.deps.ids.as_ref()).transaction()?;
        let transaction = plan.into_transaction(id, label);
        let committed = commit(&self.project, &transaction)?;
        let change = ChangesetWrite::Revert {
            changeset_id,
            at: self.deps.clock.now_unix_millis(),
        };
        self.journal_with_changeset(&transaction, &committed, &change)
            .await?;
        self.history.record(&transaction, &committed);
        self.changesets.by_task.remove(&task_id);
        let reverted_items = committed.touched.items.iter().copied().collect();
        self.install(committed);
        self.publish_changeset(task_id);
        Ok(TaskRevertReport {
            task_id,
            reverted: true,
            reverted_items,
            conflicts,
        })
    }

    /// What reverting `task_id` would report now: the revert planned and committed on a copy,
    /// which is then dropped.
    pub(super) fn preview_revert(&self, task_id: TaskId) -> Result<TaskRevertReport, AppError> {
        let changeset = self
            .changesets
            .get(task_id)
            .ok_or(AppError::NothingToRevert { task_id })?;
        let plan = plan_revert(&self.project, changeset);
        let conflicts = plan.conflicts.clone();
        if plan.ops.is_empty() {
            return Ok(TaskRevertReport {
                task_id,
                reverted: false,
                reverted_items: Vec::new(),
                conflicts,
            });
        }
        // Never journaled nor installed: committed on a copy only to know what it would touch.
        let id = NewIds(self.deps.ids.as_ref()).transaction()?;
        let transaction = plan.into_transaction(id, String::new());
        let committed = commit(&self.project, &transaction)?;
        Ok(TaskRevertReport {
            task_id,
            reverted: true,
            reverted_items: committed.touched.items.iter().copied().collect(),
            conflicts,
        })
    }

    /// The items of `task_id`'s open changeset that still exist.
    pub(super) fn changeset_items(&self, task_id: TaskId) -> ChangesetItems {
        let changeset = self.changesets.get(task_id);
        ChangesetItems {
            changeset_id: changeset.map(|changeset| changeset.id),
            touched_items: changeset
                .map(|changeset| present_items(changeset, &self.project))
                .unwrap_or_default(),
        }
    }

    /// Publishes `taskChangesetUpdated` for `task_id` (its open changeset, or none).
    fn publish_changeset(&self, task_id: TaskId) {
        let items = self.changeset_items(task_id);
        self.deps.events.publish(CoreEvent::TaskChangesetUpdated {
            project_id: self.project.id,
            task_id,
            changeset_id: items.changeset_id,
            touched_items: items.touched_items,
        });
    }
}
