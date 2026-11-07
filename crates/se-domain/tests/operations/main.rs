//! Property tests of operations, transactions, history and changesets (prompt 06): random
//! sequences of operations on the example project keep it valid, invert exactly, undo and redo
//! consistently, and a changeset reverted without interference restores the project.

// Test helpers build values that are valid by construction; panics are test failures.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod generators;
mod picking;
mod reformat;

use generators::{Generator, KINDS, Seed, seed};
use proptest::prelude::*;
use se_domain::{
    Actor, Changeset, ChangesetId, History, Operation, Project, TaskId, Transaction, TransactionId,
    apply, commit, plan_revert, test_support::example_project, validate_project,
};

pub(crate) const CASES: u32 = 1000;

/// `project` with every revision set to zero, to compare contents only.
fn without_revisions(mut project: Project) -> Project {
    project.revision = 0;
    for track in project
        .sequences
        .iter_mut()
        .flat_map(|sequence| &mut sequence.tracks)
    {
        track.items.iter_mut().for_each(|item| item.revision = 0);
    }
    project
}

/// A one-operation transaction.
fn transaction(index: u64, actor: Actor, operation: Operation) -> Transaction {
    Transaction {
        id: TransactionId::from_test_index(index),
        actor,
        label: format!("Edit {index}"),
        ops: vec![operation],
    }
}

/// Every item revision and the project revision only grow from `before` to `after`.
fn assert_revisions_grow(before: &Project, after: &Project) {
    assert!(after.revision > before.revision);
    let revisions = |project: &Project| -> Vec<(se_domain::ItemId, u64)> {
        project
            .sequences
            .iter()
            .flat_map(|sequence| &sequence.tracks)
            .flat_map(|track| track.items.iter().map(|item| (item.id, item.revision)))
            .collect()
    };
    let previous = revisions(before);
    for (id, revision) in revisions(after) {
        if let Some((_, old)) = previous.iter().find(|(old_id, _)| *old_id == id) {
            assert!(
                revision >= *old,
                "item {id} went from revision {old} to {revision}"
            );
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    /// Every successful operation leaves a valid project, and its inverse operations, applied
    /// one by one, give back exactly the previous project.
    #[test]
    fn operations_keep_projects_valid_and_invert_exactly(seeds in prop::collection::vec(seed(), 1..16)) {
        let mut generator = Generator::new();
        let mut project = example_project();
        for seed in seeds {
            let Some(operation) = generator.operation(&project, seed) else { continue };
            let Ok(applied) = apply(&project, &operation) else { continue };
            prop_assert_eq!(validate_project(&applied.project), []);
            let mut undone = applied.project.clone();
            for inverse in &applied.inverse {
                undone = apply(&undone, inverse)
                    .unwrap_or_else(|error| panic!("inverse {inverse:?} of {operation:?} failed: {error}"))
                    .project;
            }
            prop_assert_eq!(&undone, &project, "inverse of {:?}", operation);
            project = applied.project;
        }
    }

    /// Undoing every user transaction walks back through the same states, redoing walks
    /// forward again; revisions only grow.
    #[test]
    fn undo_and_redo_walk_through_the_same_states(seeds in prop::collection::vec(seed(), 1..12)) {
        let mut generator = Generator::new();
        let mut history = History::default();
        let mut states = vec![example_project()];
        let mut project = example_project();
        for (index, seed) in (1..).zip(seeds) {
            let Some(operation) = generator.operation(&project, seed) else { continue };
            let transaction = transaction(index, Actor::User, operation);
            let Ok(committed) = commit(&project, &transaction) else { continue };
            assert_revisions_grow(&project, &committed.project);
            history.record(&transaction, &committed);
            let changed_something = !committed.inverse.ops.is_empty();
            project = committed.project;
            if changed_something {
                states.push(project.clone());
            }
        }
        let mut id = 1000;
        for expected in states.iter().rev().skip(1) {
            id += 1;
            let step = history.undo(&project, TransactionId::from_test_index(id)).unwrap();
            assert_revisions_grow(&project, &step.committed.project);
            project = step.committed.project;
            prop_assert_eq!(without_revisions(project.clone()), without_revisions(expected.clone()));
        }
        prop_assert!(!history.can_undo());
        for expected in states.iter().skip(1) {
            id += 1;
            project = history.redo(&project, TransactionId::from_test_index(id)).unwrap().committed.project;
            prop_assert_eq!(without_revisions(project.clone()), without_revisions(expected.clone()));
        }
        prop_assert!(!history.can_redo());
    }

    /// Reverting the changeset of a task nobody interfered with restores the project as it was
    /// before the task, revisions aside, without conflicts.
    #[test]
    fn reverting_an_undisturbed_changeset_restores_the_project(seeds in prop::collection::vec(seed(), 1..12)) {
        let task = TaskId::from_test_index(1);
        let mut generator = Generator::new();
        let mut changeset = Changeset::new(ChangesetId::from_test_index(1), task);
        let mut project = example_project();
        for (index, seed) in (1..).zip(seeds) {
            let Some(operation) = generator.operation(&project, seed) else { continue };
            let transaction = transaction(index, Actor::AiTask { task_id: task }, operation);
            let Ok(committed) = commit(&project, &transaction) else { continue };
            changeset.record(&transaction, &committed).unwrap();
            project = committed.project;
        }
        let plan = plan_revert(&project, &changeset);
        prop_assert_eq!(&plan.conflicts, &[]);
        if !plan.ops.is_empty() {
            let revert = plan.into_transaction(TransactionId::from_test_index(999), "Revert".to_owned());
            project = commit(&project, &revert).unwrap().project;
        }
        prop_assert_eq!(without_revisions(project), without_revisions(example_project()));
    }
}

/// The generator is not vacuous: every operation kind succeeds now and then.
#[test]
fn every_operation_kind_succeeds_sometimes() {
    let mut successes = [0_u32; KINDS as usize];
    let mut generator = Generator::new();
    let mut state: u64 = 0x2545_f491_4f6c_dd1d;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for _ in 0..200 {
        let mut project = example_project();
        for _ in 0..20 {
            let value = next();
            let seed = Seed {
                kind: u8::try_from((value >> 56) % u64::from(KINDS)).unwrap(),
                pick: u32::try_from(value >> 32).unwrap(),
                choice: u32::try_from(value & u64::from(u32::MAX)).unwrap(),
                offset_ms: i32::try_from((value >> 40) % 16_000).unwrap() - 8000,
            };
            let Some(operation) = generator.operation(&project, seed) else {
                continue;
            };
            if let Ok(applied) = apply(&project, &operation) {
                successes[usize::from(seed.kind)] += 1;
                project = applied.project;
            }
        }
    }
    for (kind, count) in successes.iter().enumerate() {
        assert!(
            *count > 0,
            "operation kind {kind} never succeeded: {successes:?}"
        );
    }
}
