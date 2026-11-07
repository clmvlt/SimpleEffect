//! Transactions: atomic groups of operations by one actor, and [`commit`] (ADR-0005).

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    DomainError, ItemId, Operation, Project, TaskId, TouchedEntities, TransactionId, apply_all,
};

/// Operations applied together, all or none, by one actor. The unit of undo, of changesets and
/// of the operation journal (prompt 07), which stores it in this serialized form.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    /// Stable id.
    pub id: TransactionId,
    /// Who performs it.
    pub actor: Actor,
    /// What it does, for the undo menu and task logs (`Move clip`).
    pub label: String,
    /// The operations, applied in order.
    pub ops: Vec<Operation>,
}

/// Who performs a transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Actor {
    /// The user, through the UI (undo/redo covers only these transactions).
    User,
    /// An AI agent working on a task (its transactions form the task's changeset).
    AiTask {
        /// The task.
        task_id: TaskId,
    },
    /// The application itself, recording facts it observed (missing media at opening,
    /// decodability reported by the renderer). Journaled like any commit, never undone.
    System,
}

/// The outcome of [`commit`].
#[derive(Debug, Clone, PartialEq)]
pub struct Committed {
    /// The project after the transaction, revisions bumped.
    pub project: Project,
    /// The transaction undoing it: same id, actor and label, inverse operations in order.
    /// Give it a new id before committing it.
    pub inverse: Transaction,
    /// Entities the transaction added, changed or removed.
    pub touched: TouchedEntities,
}

/// Applies every operation of `transaction` in order, or none ([`apply_all`]: each operation
/// checks its preconditions, the document is validated once after the last one). On success the
/// project revision grows by one and every touched item that still exists gets its revision
/// bumped by one.
///
/// # Errors
/// [`DomainError::EmptyTransaction`] without operations; otherwise the error of the first
/// operation that fails, and nothing is changed.
pub fn commit(project: &Project, transaction: &Transaction) -> Result<Committed, DomainError> {
    if transaction.ops.is_empty() {
        return Err(DomainError::EmptyTransaction);
    }
    let applied = apply_all(project, &transaction.ops)?;
    let (mut draft, touched) = (applied.project, applied.touched);
    bump_revisions(&mut draft, &touched.items);
    let inverse = Transaction {
        ops: applied.inverse,
        ..transaction.clone()
    };
    Ok(Committed {
        project: draft,
        inverse,
        touched,
    })
}

fn bump_revisions(project: &mut Project, touched_items: &BTreeSet<ItemId>) {
    project.revision = project.revision.saturating_add(1);
    let items = project
        .sequences
        .iter_mut()
        .flat_map(|sequence| &mut sequence.tracks)
        .flat_map(|track| &mut track.items);
    for item in items.filter(|item| touched_items.contains(&item.id)) {
        item.revision = item.revision.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::{
        Animatable, Easing, ItemEdge, ItemPatch, Keyframe, MoveItem, PropertyAnimation, RemoveItem,
        RippleDeleteRange, SetKeyframes, TextPatch, TrimItem, UpdateItem,
        operation::testing::{SEQUENCE, V2, V3, find},
        test_support::{example_project, millis, range_ms},
    };

    fn item(index: u64) -> ItemId {
        ItemId::from_test_index(index)
    }

    fn transaction(actor: Actor, ops: Vec<Operation>) -> Transaction {
        Transaction {
            id: TransactionId::from_test_index(1),
            actor,
            label: "Edit".to_owned(),
            ops,
        }
    }

    fn move_text(start_ms: i64) -> Operation {
        MoveItem {
            item: item(3),
            track: V2,
            start: millis(start_ms),
            expected_revision: Some(1),
        }
        .into()
    }

    #[test]
    fn bumps_the_revisions_of_touched_items_and_of_the_project() {
        let project = example_project();
        let committed = commit(&project, &transaction(Actor::User, vec![move_text(8000)])).unwrap();
        assert_eq!(committed.project.revision, 43);
        assert_eq!(find(&committed.project, 3).revision, 2);
        assert_eq!(find(&committed.project, 4).revision, 1);
        assert_eq!(committed.touched.items, BTreeSet::from([item(3)]));
    }

    #[test]
    fn is_all_or_nothing() {
        let project = example_project();
        let ops = vec![
            move_text(8000),
            RemoveItem {
                item: item(999),
                expected_revision: None,
            }
            .into(),
        ];
        assert_eq!(
            commit(&project, &transaction(Actor::User, ops)),
            Err(DomainError::ItemNotFound { item: item(999) })
        );
        assert_eq!(
            commit(&project, &transaction(Actor::User, Vec::new())),
            Err(DomainError::EmptyTransaction)
        );
    }

    #[test]
    fn validates_the_document_once_after_the_last_operation() {
        let project = example_project();
        let through_invalid = vec![move_text(-1000), move_text(8000)];
        let committed = commit(&project, &transaction(Actor::User, through_invalid)).unwrap();
        assert_eq!(
            find(&committed.project, 3).timeline_range,
            range_ms(8000, 3000)
        );
        let left_invalid = commit(&project, &transaction(Actor::User, vec![move_text(-1000)]));
        let Err(DomainError::InvalidDocument { violations }) = left_invalid else {
            panic!("a negative start must be refused, got {left_invalid:?}");
        };
        assert_eq!(violations[0].code, crate::ViolationCode::NegativeTime);
    }

    #[test]
    fn the_inverse_transaction_undoes_every_operation_in_reverse_order() {
        let project = example_project();
        let ops = vec![
            move_text(8000),
            RemoveItem {
                item: item(4),
                expected_revision: None,
            }
            .into(),
            move_text(5000),
        ];
        let committed = commit(&project, &transaction(Actor::User, ops)).unwrap();
        assert_eq!(committed.inverse.ops.len(), 3);
        let undone = commit(&committed.project, &committed.inverse).unwrap();
        // Revisions only grow: the text was edited twice (1 → 3), the image removed then added
        // back (1 → 2).
        let mut expected = project;
        expected.revision = 44;
        expected.sequences[0].tracks[1].items[0].revision = 3;
        expected.sequences[0].tracks[1].items[1].revision = 2;
        assert_eq!(undone.project, expected);
    }

    /// Fixes the journal format (prompt 07): a change here is a change of stored transactions.
    #[test]
    fn transaction_json_format() {
        let ops = vec![
            move_text(8000),
            TrimItem {
                item: item(2),
                edge: ItemEdge::Start,
                to: millis(11_000),
                expected_revision: None,
            }
            .into(),
            UpdateItem {
                item: item(3),
                patch: ItemPatch::Text(TextPatch {
                    content: Some("Salut".to_owned()),
                    ..TextPatch::default()
                }),
                expected_revision: None,
            }
            .into(),
            SetKeyframes {
                item: item(4),
                animation: PropertyAnimation::Opacity {
                    value: Animatable::Keyframed {
                        keyframes: vec![Keyframe {
                            time: millis(0),
                            value: 0.0,
                            easing: Easing::Linear,
                        }],
                    },
                },
                expected_revision: None,
            }
            .into(),
            RippleDeleteRange {
                sequence: SEQUENCE,
                range: range_ms(5500, 1000),
                tracks: Some(vec![V3]),
                split_ids: BTreeMap::from([(item(6), item(100))]),
            }
            .into(),
        ];
        let task = Actor::AiTask {
            task_id: TaskId::from_test_index(1),
        };
        let forward = transaction(task, ops);
        insta::assert_json_snapshot!("forward_transaction", forward);
        commit(&example_project(), &forward).unwrap();
        let moved = commit(
            &example_project(),
            &transaction(Actor::User, vec![move_text(8000)]),
        );
        insta::assert_json_snapshot!("inverse_transaction", moved.unwrap().inverse);
    }
}
