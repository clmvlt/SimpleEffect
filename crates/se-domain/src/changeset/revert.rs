//! [`plan_revert`]: undoing an AI task's changeset against the current document.

use std::collections::{BTreeMap, BTreeSet};

use super::{Changeset, Conflict, revisions_of};
use crate::{Actor, ItemId, Operation, Project, Transaction, TransactionId, apply};

/// What reverting a changeset will do: the operations to commit and what is left as it is.
#[derive(Debug, Clone, PartialEq)]
pub struct RevertPlan {
    /// The inverse operations that still apply, in order. Committing them on the project the
    /// plan was computed from succeeds.
    pub ops: Vec<Operation>,
    /// What cannot be reverted, and why.
    pub conflicts: Vec<Conflict>,
}

impl RevertPlan {
    /// The user transaction performing the revert (so the revert itself can be undone). Do not
    /// commit it when `ops` is empty.
    #[must_use]
    pub fn into_transaction(self, id: TransactionId, label: String) -> Transaction {
        Transaction {
            id,
            actor: Actor::User,
            label,
            ops: self.ops,
        }
    }
}

/// Plans the revert of `changeset` on the current `project`.
///
/// An item touched by the task whose revision differs from the one the task left (changed or
/// removed by someone else since) is a conflict: inverse operations involving it are skipped.
/// The other inverse operations are tried last transaction first, each on the result of the
/// previous ones; those that no longer apply, or that would affect items the task never
/// touched, are skipped and reported. Without interleaved edits the plan restores the project
/// as it was before the task (revisions aside).
#[must_use]
pub fn plan_revert(project: &Project, changeset: &Changeset) -> RevertPlan {
    let task_revisions = last_revisions(changeset);
    let current = revisions_of(project, task_revisions.keys());
    let mut changed = BTreeSet::new();
    let mut conflicts = Vec::new();
    for (item, task_revision) in &task_revisions {
        let current_revision = current.get(item).copied().flatten();
        if current_revision != *task_revision {
            changed.insert(*item);
            conflicts.push(Conflict::ItemChanged {
                item: *item,
                task_revision: *task_revision,
                current_revision,
            });
        }
    }
    let inverse_ops = changeset
        .entries
        .iter()
        .rev()
        .flat_map(|entry| &entry.inverse.ops);
    let mut working = project.clone();
    let mut ops = Vec::new();
    for operation in inverse_ops {
        if named_items(operation)
            .iter()
            .any(|item| changed.contains(item))
        {
            continue;
        }
        match try_revert(&working, operation, &task_revisions, &changed) {
            Ok(Some(reverted)) => {
                working = reverted;
                ops.push(operation.clone());
            }
            Ok(None) => {}
            Err(conflict) => conflicts.push(conflict),
        }
    }
    RevertPlan { ops, conflicts }
}

/// The project after `operation`, `None` to skip it silently (it involves a changed item), or
/// the conflict it raises.
fn try_revert(
    working: &Project,
    operation: &Operation,
    task_revisions: &BTreeMap<ItemId, Option<u64>>,
    changed: &BTreeSet<ItemId>,
) -> Result<Option<Project>, Conflict> {
    let applied = apply(working, operation).map_err(|error| Conflict::CannotApply {
        code: error.code(),
        message: error.to_string(),
    })?;
    if applied
        .touched
        .items
        .iter()
        .any(|item| changed.contains(item))
    {
        return Ok(None);
    }
    let foreign: Vec<ItemId> = applied
        .touched
        .items
        .iter()
        .filter(|item| !task_revisions.contains_key(item))
        .copied()
        .collect();
    if foreign.is_empty() {
        Ok(Some(applied.project))
    } else {
        Err(Conflict::ForeignItems { items: foreign })
    }
}

/// For each item the task touched, its revision after the task's last edit of it.
fn last_revisions(changeset: &Changeset) -> BTreeMap<ItemId, Option<u64>> {
    let mut revisions = BTreeMap::new();
    for entry in &changeset.entries {
        revisions.extend(entry.touched_revisions.iter().map(|(id, rev)| (*id, *rev)));
    }
    revisions
}

/// Items an inverse operation names directly (others are found by applying it).
fn named_items(operation: &Operation) -> Vec<ItemId> {
    match operation {
        Operation::AddItem(add) => vec![add.item.id],
        Operation::RemoveItem(remove) => vec![remove.item],
        Operation::RestoreItem(restore) => vec![restore.item.id],
        Operation::AddTrack(add) => add.track.items.iter().map(|item| item.id).collect(),
        Operation::AddSequence(add) => add
            .sequence
            .tracks
            .iter()
            .flat_map(|track| &track.items)
            .map(|item| item.id)
            .collect(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AddItem, AddMarker, AddTrack, ChangesetId, DomainError, DomainErrorCode, ItemPatch, Marker,
        MarkerColor, MarkerId, MoveItem, SplitItem, TaskId, TextPatch, TrackId, TrackKind,
        UpdateItem, commit,
        operation::testing::{SEQUENCE, V2, find, shape_item, track_items},
        test_support::{example_project, millis, range_ms, track},
    };

    const TASK: TaskId = TaskId::from_test_index(7);

    /// Commits each group of operations as a transaction of [`TASK`], recorded in a changeset.
    fn run_task(project: Project, transactions: Vec<Vec<Operation>>) -> (Project, Changeset) {
        let mut changeset = Changeset::new(ChangesetId::from_test_index(1), TASK);
        let mut project = project;
        for (index, ops) in transactions.into_iter().enumerate() {
            let transaction = Transaction {
                id: TransactionId::from_test_index(100 + index as u64),
                actor: Actor::AiTask { task_id: TASK },
                label: "Task edit".to_owned(),
                ops,
            };
            let committed = commit(&project, &transaction).unwrap();
            changeset.record(&transaction, &committed).unwrap();
            project = committed.project;
        }
        (project, changeset)
    }

    /// Commits `op` as a user transaction.
    fn user_edit(project: &Project, op: impl Into<Operation>) -> Project {
        let transaction = Transaction {
            id: TransactionId::from_test_index(200),
            actor: Actor::User,
            label: "User edit".to_owned(),
            ops: vec![op.into()],
        };
        commit(project, &transaction).unwrap().project
    }

    /// Commits the plan as a user transaction.
    fn commit_plan(project: &Project, plan: RevertPlan) -> Project {
        let transaction =
            plan.into_transaction(TransactionId::from_test_index(300), "Revert".into());
        commit(project, &transaction).unwrap().project
    }

    fn without_revisions(mut project: Project) -> Project {
        project.revision = 0;
        for sequence in &mut project.sequences {
            for track in &mut sequence.tracks {
                track.items.iter_mut().for_each(|item| item.revision = 0);
            }
        }
        project
    }

    fn move_item(item: u64, track: TrackId, start_ms: i64) -> Operation {
        MoveItem {
            item: ItemId::from_test_index(item),
            track,
            start: millis(start_ms),
            expected_revision: None,
        }
        .into()
    }

    #[test]
    fn reverting_without_interference_restores_the_project() {
        let marker = Marker {
            id: MarkerId::from_test_index(2),
            time: millis(500),
            label: "Beat".to_owned(),
            color: MarkerColor::Blue,
        };
        let split = SplitItem {
            item: ItemId::from_test_index(9),
            at: millis(5000),
            new_item_id: ItemId::from_test_index(100),
            expected_revision: None,
        };
        let add_marker = AddMarker {
            sequence: SEQUENCE,
            index: None,
            marker,
        };
        let (project, changeset) = run_task(
            example_project(),
            vec![
                vec![move_item(3, V2, 8000), split.into()],
                vec![add_marker.into()],
            ],
        );
        let plan = plan_revert(&project, &changeset);
        assert_eq!(plan.conflicts, []);
        let reverted = commit_plan(&project, plan);
        assert_eq!(
            without_revisions(reverted),
            without_revisions(example_project())
        );
    }

    #[test]
    fn items_changed_since_are_conflicts_and_the_rest_is_reverted() {
        let (project, changeset) = run_task(
            example_project(),
            vec![vec![move_item(3, V2, 8000), move_item(4, V2, 12_000)]],
        );
        let patch = ItemPatch::Text(TextPatch {
            content: Some("Mine".to_owned()),
            ..TextPatch::default()
        });
        let edited = user_edit(
            &project,
            UpdateItem {
                item: ItemId::from_test_index(3),
                patch,
                expected_revision: None,
            },
        );
        let plan = plan_revert(&edited, &changeset);
        assert_eq!(
            plan.conflicts,
            [Conflict::ItemChanged {
                item: ItemId::from_test_index(3),
                task_revision: Some(2),
                current_revision: Some(3),
            }]
        );
        let reverted = commit_plan(&edited, plan);
        assert_eq!(find(&reverted, 4).timeline_range, range_ms(5000, 2000));
        assert_eq!(find(&reverted, 3).timeline_range, range_ms(8000, 3000));
    }

    #[test]
    fn user_items_on_a_task_track_keep_the_track() {
        let new_track = TrackId::from_test_index(10);
        let add_track = AddTrack {
            sequence: SEQUENCE,
            index: None,
            track: track(10, TrackKind::Video, "AI titles", Vec::new()),
        };
        let add_shape = AddItem {
            track: new_track,
            item: shape_item(100, range_ms(0, 1000)),
        };
        let (project, changeset) = run_task(
            example_project(),
            vec![vec![add_track.into(), add_shape.into()]],
        );
        let user_shape = AddItem {
            track: new_track,
            item: shape_item(101, range_ms(2000, 1000)),
        };
        let edited = user_edit(&project, user_shape);
        let plan = plan_revert(&edited, &changeset);
        assert_eq!(
            plan.conflicts,
            [Conflict::ForeignItems {
                items: vec![ItemId::from_test_index(101)]
            }]
        );
        let reverted = commit_plan(&edited, plan);
        let remaining: Vec<ItemId> = track_items(&reverted, new_track)
            .iter()
            .map(|item| item.id)
            .collect();
        assert_eq!(remaining, [ItemId::from_test_index(101)]);
    }

    #[test]
    fn inverses_that_no_longer_apply_are_reported() {
        let (project, changeset) =
            run_task(example_project(), vec![vec![move_item(4, V2, 12_000)]]);
        let blocker = AddItem {
            track: V2,
            item: shape_item(100, range_ms(5000, 2000)),
        };
        let edited = user_edit(&project, blocker);
        let plan = plan_revert(&edited, &changeset);
        assert_eq!(plan.ops, []);
        let [Conflict::CannotApply { code, message }] = plan.conflicts.as_slice() else {
            panic!("expected one conflict, got {:?}", plan.conflicts);
        };
        assert_eq!(*code, DomainErrorCode::OverlappingItems);
        assert!(message.contains("would overlap"), "{message}");
    }

    #[test]
    fn only_the_task_records_in_its_changeset() {
        let mut changeset = Changeset::new(ChangesetId::from_test_index(1), TASK);
        let transaction = Transaction {
            id: TransactionId::from_test_index(1),
            actor: Actor::User,
            label: "User edit".to_owned(),
            ops: vec![move_item(3, V2, 8000)],
        };
        let committed = commit(&example_project(), &transaction).unwrap();
        assert_eq!(
            changeset.record(&transaction, &committed),
            Err(DomainError::WrongChangeset { task: TASK })
        );
    }
}
