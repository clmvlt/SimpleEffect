//! Changesets of AI tasks in a real `project.db`: written with their journal entry in one
//! transaction, reloaded after a crash, closed by a revert.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use se_app::ports::{ChangesetWrite, JournalEntry, ProjectRepository};
use se_domain::{
    Actor, Changeset, ChangesetId, ItemId, MoveItem, Project, TaskId, Transaction, TransactionId,
    commit,
    test_support::{example_project, millis},
};
use se_store::FsProjectRepository;

const TASK: TaskId = TaskId::from_test_index(7);
const CHANGESET: ChangesetId = ChangesetId::from_test_index(8);

/// The task's transaction moving the example's text to `start_ms`, committed on `project`.
fn task_edit(project: &Project, index: u64, start_ms: i64) -> (Transaction, Project, Changeset) {
    let transaction = Transaction {
        id: TransactionId::from_test_index(index),
        actor: Actor::AiTask { task_id: TASK },
        label: "move_item".to_owned(),
        ops: vec![
            MoveItem {
                item: ItemId::from_test_index(3),
                track: project.sequences[0].tracks[1].id,
                start: millis(start_ms),
                expected_revision: None,
            }
            .into(),
        ],
    };
    let committed = commit(project, &transaction).unwrap();
    let mut changeset = Changeset::new(CHANGESET, TASK);
    changeset.record(&transaction, &committed).unwrap();
    (transaction, committed.project, changeset)
}

fn journal(revision: u64, transaction: &Transaction) -> JournalEntry {
    JournalEntry {
        revision,
        committed_at: 1_700_000_000_000,
        transaction: transaction.clone(),
    }
}

#[tokio::test]
async fn changesets_survive_a_crash_with_their_journal_and_close_when_reverted() {
    let parent = tempfile::tempdir().unwrap();
    let repository = FsProjectRepository;
    let project = example_project();
    let path = repository.create(parent.path(), &project).await.unwrap();
    let store = repository.open(&path).await.unwrap();
    assert_eq!(store.load_changesets().await.unwrap(), []);

    let (first, after_first, first_changeset) = task_edit(&project, 1, 8000);
    let (second, _, second_changeset) = task_edit(&after_first, 2, 9000);
    for (revision, transaction, changeset) in [
        (project.revision + 1, &first, &first_changeset),
        (project.revision + 2, &second, &second_changeset),
    ] {
        let change = ChangesetWrite::Record {
            changeset_id: CHANGESET,
            task_id: TASK,
            created_at: 5,
            entry: Box::new(changeset.entries[0].clone()),
        };
        store
            .append_journal_with_changeset(&journal(revision, transaction), &change)
            .await
            .unwrap();
    }
    // A crash: dropped without closing.
    drop(store);

    let store = repository.open(&path).await.unwrap();
    let journal_entries = store.read_journal_since(project.revision).await.unwrap();
    assert_eq!(journal_entries.len(), 2);
    let changesets = store.load_changesets().await.unwrap();
    assert_eq!(changesets.len(), 1);
    assert_eq!(changesets[0].id, CHANGESET);
    assert_eq!(changesets[0].task_id, TASK);
    let transactions: Vec<TransactionId> = changesets[0]
        .entries
        .iter()
        .map(|entry| entry.transaction.id)
        .collect();
    assert_eq!(transactions, [first.id, second.id]);
    assert_eq!(changesets[0].entries[1], second_changeset.entries[0]);

    let revert = ChangesetWrite::Revert {
        changeset_id: CHANGESET,
        at: 9,
    };
    let undo = Transaction {
        id: TransactionId::from_test_index(3),
        actor: Actor::User,
        label: "Revert AI task".to_owned(),
        ops: Vec::new(),
    };
    store
        .append_journal_with_changeset(&journal(project.revision + 3, &undo), &revert)
        .await
        .unwrap();
    assert_eq!(store.load_changesets().await.unwrap(), []);
    store.close().await.unwrap();
}

#[tokio::test]
async fn a_failed_changeset_write_leaves_no_journal_entry() {
    let parent = tempfile::tempdir().unwrap();
    let repository = FsProjectRepository;
    let project = example_project();
    let path = repository.create(parent.path(), &project).await.unwrap();
    let store = repository.open(&path).await.unwrap();
    let (transaction, _, _) = task_edit(&project, 1, 8000);
    let revert_unknown = ChangesetWrite::Revert {
        changeset_id: CHANGESET,
        at: 9,
    };
    let refused = store
        .append_journal_with_changeset(
            &journal(project.revision + 1, &transaction),
            &revert_unknown,
        )
        .await;
    assert!(refused.is_err(), "reverting an unknown changeset must fail");
    assert_eq!(
        store.read_journal_since(project.revision).await.unwrap(),
        [],
        "the journal entry was rolled back with the changeset write"
    );
    store.close().await.unwrap();
}
