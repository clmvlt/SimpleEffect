//! [`History`]: the user's undo and redo stacks (ADR-0005).

use std::collections::VecDeque;

use crate::{
    Actor, Committed, DomainError, Operation, Project, Transaction, TransactionId, commit,
};

/// Undo/redo of the user's own transactions. AI transactions are never undone here (they are
/// reverted per task through their [`crate::Changeset`]), so an undo never removes AI work.
///
/// Each entry keeps the operations that undo (or redo) it, computed when it was committed. Undo
/// and redo commit them as new user transactions, so they are journaled like any other change
/// and bump revisions; they may fail if another actor changed the document meanwhile (the entry
/// then stays in place).
#[derive(Debug, Clone, PartialEq)]
pub struct History {
    undo: VecDeque<HistoryEntry>,
    redo: Vec<HistoryEntry>,
    max_depth: usize,
}

/// A transaction that can be undone or redone.
#[derive(Debug, Clone, PartialEq)]
struct HistoryEntry {
    /// The transaction's label, shown in the menus.
    label: String,
    /// The operations that undo it (on the undo stack) or redo it (on the redo stack).
    ops: Vec<Operation>,
}

/// An undo or redo that was committed.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryStep {
    /// The user transaction that was committed (to journal like any other).
    pub transaction: Transaction,
    /// Its outcome.
    pub committed: Committed,
}

impl History {
    /// Default number of transactions kept on the undo stack.
    pub const DEFAULT_DEPTH: usize = 200;

    /// An empty history keeping at most `max_depth` undoable transactions.
    #[must_use]
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo: VecDeque::new(),
            redo: Vec::new(),
            max_depth,
        }
    }

    /// Records a committed transaction. Only user transactions that changed something are kept
    /// (a ripple delete over empty time has nothing to undo); a new one empties the redo stack
    /// and the oldest entries go beyond the maximum depth.
    pub fn record(&mut self, transaction: &Transaction, committed: &Committed) {
        if transaction.actor != Actor::User || committed.inverse.ops.is_empty() {
            return;
        }
        self.redo.clear();
        self.undo.push_back(HistoryEntry {
            label: transaction.label.clone(),
            ops: committed.inverse.ops.clone(),
        });
        while self.undo.len() > self.max_depth {
            self.undo.pop_front();
        }
    }

    /// Undoes the latest user transaction by committing its inverse as transaction `id`.
    ///
    /// # Errors
    /// [`DomainError::NothingToUndo`], or the commit error (the entry stays undoable).
    pub fn undo(
        &mut self,
        project: &Project,
        id: TransactionId,
    ) -> Result<HistoryStep, DomainError> {
        let entry = self.undo.back().ok_or(DomainError::NothingToUndo)?;
        let step = commit_entry(project, entry, id)?;
        if let Some(entry) = self.undo.pop_back() {
            self.redo.push(HistoryEntry {
                label: entry.label,
                ops: step.committed.inverse.ops.clone(),
            });
        }
        Ok(step)
    }

    /// Redoes the latest undone transaction by committing it again as transaction `id`.
    ///
    /// # Errors
    /// [`DomainError::NothingToRedo`], or the commit error (the entry stays redoable).
    pub fn redo(
        &mut self,
        project: &Project,
        id: TransactionId,
    ) -> Result<HistoryStep, DomainError> {
        let entry = self.redo.last().ok_or(DomainError::NothingToRedo)?;
        let step = commit_entry(project, entry, id)?;
        if let Some(entry) = self.redo.pop() {
            self.undo.push_back(HistoryEntry {
                label: entry.label,
                ops: step.committed.inverse.ops.clone(),
            });
        }
        Ok(step)
    }

    /// Whether [`History::undo`] has something to undo.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether [`History::redo`] has something to redo.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Label of the transaction the next undo reverts.
    #[must_use]
    pub fn undo_label(&self) -> Option<&str> {
        self.undo.back().map(|entry| entry.label.as_str())
    }

    /// Label of the transaction the next redo replays.
    #[must_use]
    pub fn redo_label(&self) -> Option<&str> {
        self.redo.last().map(|entry| entry.label.as_str())
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new(Self::DEFAULT_DEPTH)
    }
}

fn commit_entry(
    project: &Project,
    entry: &HistoryEntry,
    id: TransactionId,
) -> Result<HistoryStep, DomainError> {
    let transaction = Transaction {
        id,
        actor: Actor::User,
        label: entry.label.clone(),
        ops: entry.ops.clone(),
    };
    let committed = commit(project, &transaction)?;
    Ok(HistoryStep {
        transaction,
        committed,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::{
        ItemId, MoveItem, RemoveItem, RippleDeleteRange, TaskId,
        operation::testing::{SEQUENCE, V2, find},
        test_support::{example_project, millis, range_ms},
    };

    fn transaction(index: u64, actor: Actor, op: impl Into<Operation>) -> Transaction {
        Transaction {
            id: TransactionId::from_test_index(index),
            actor,
            label: format!("Edit {index}"),
            ops: vec![op.into()],
        }
    }

    fn move_text(start_ms: i64) -> MoveItem {
        MoveItem {
            item: ItemId::from_test_index(3),
            track: V2,
            start: millis(start_ms),
            expected_revision: None,
        }
    }

    /// Commits `transaction` on `project` and records it.
    fn run(history: &mut History, project: &Project, transaction: &Transaction) -> Project {
        let committed = commit(project, transaction).unwrap();
        history.record(transaction, &committed);
        committed.project
    }

    #[test]
    fn undoes_and_redoes_user_transactions() {
        let mut history = History::default();
        let start = example_project();
        let moved = run(
            &mut history,
            &start,
            &transaction(1, Actor::User, move_text(8000)),
        );
        assert_eq!(history.undo_label(), Some("Edit 1"));
        let undone = history
            .undo(&moved, TransactionId::from_test_index(2))
            .unwrap();
        assert_eq!(
            find(&undone.committed.project, 3).timeline_range,
            range_ms(1000, 3000)
        );
        assert_eq!(find(&undone.committed.project, 3).revision, 3);
        assert_eq!(undone.transaction.actor, Actor::User);
        assert!(!history.can_undo());
        assert_eq!(history.redo_label(), Some("Edit 1"));
        let redone = history
            .redo(&undone.committed.project, TransactionId::from_test_index(3))
            .unwrap();
        assert_eq!(
            find(&redone.committed.project, 3).timeline_range,
            range_ms(8000, 3000)
        );
        assert!(history.can_undo() && !history.can_redo());
    }

    #[test]
    fn ignores_ai_transactions_and_clears_redo_on_new_user_work() {
        let mut history = History::default();
        let task = Actor::AiTask {
            task_id: TaskId::from_test_index(1),
        };
        let project = run(
            &mut history,
            &example_project(),
            &transaction(1, task, move_text(8000)),
        );
        assert_eq!(
            history.undo(&project, TransactionId::from_test_index(2)),
            Err(DomainError::NothingToUndo)
        );
        let project = run(
            &mut history,
            &project,
            &transaction(3, Actor::User, move_text(1000)),
        );
        let project = history
            .undo(&project, TransactionId::from_test_index(4))
            .unwrap()
            .committed
            .project;
        assert!(history.can_redo());
        run(
            &mut history,
            &project,
            &transaction(5, Actor::User, move_text(2000)),
        );
        assert!(!history.can_redo());
    }

    #[test]
    fn edits_that_change_nothing_are_not_recorded() {
        let mut history = History::default();
        let nothing = RippleDeleteRange {
            sequence: SEQUENCE,
            range: range_ms(30_000, 1000),
            tracks: None,
            split_ids: BTreeMap::new(),
        };
        run(
            &mut history,
            &example_project(),
            &transaction(1, Actor::User, nothing),
        );
        assert!(!history.can_undo());
    }

    #[test]
    fn keeps_at_most_max_depth_entries() {
        let mut history = History::new(2);
        let mut project = example_project();
        for (index, start_ms) in [(1, 8000), (2, 12_000), (3, 16_000)] {
            project = run(
                &mut history,
                &project,
                &transaction(index, Actor::User, move_text(start_ms)),
            );
        }
        project = history
            .undo(&project, TransactionId::from_test_index(10))
            .unwrap()
            .committed
            .project;
        project = history
            .undo(&project, TransactionId::from_test_index(11))
            .unwrap()
            .committed
            .project;
        assert_eq!(find(&project, 3).timeline_range, range_ms(8000, 3000));
        assert!(!history.can_undo());
    }

    #[test]
    fn a_failing_undo_keeps_its_entry() {
        let mut history = History::default();
        let project = run(
            &mut history,
            &example_project(),
            &transaction(1, Actor::User, move_text(8000)),
        );
        let removed = commit(
            &project,
            &transaction(
                2,
                Actor::AiTask {
                    task_id: TaskId::from_test_index(1),
                },
                RemoveItem {
                    item: ItemId::from_test_index(3),
                    expected_revision: None,
                },
            ),
        )
        .unwrap()
        .project;
        assert!(matches!(
            history.undo(&removed, TransactionId::from_test_index(3)),
            Err(DomainError::ItemNotFound { .. })
        ));
        assert!(history.can_undo());
    }
}
