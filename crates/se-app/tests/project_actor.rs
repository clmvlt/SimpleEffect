//! The project actor against the in-memory store: commit order, journal, events, debounced
//! snapshots (tokio's paused clock) and close.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{sync::Arc, time::Duration};

use se_app::{
    ActorDeps, AppError, ProjectActorHandle,
    ports::{ProjectRepository, StorageError},
    test_support::{FixedClock, InMemoryProjectRepository, RecordingEventSink, SequentialIds},
};
use se_domain::{
    Actor, DomainError, ItemId, MoveItem, Project, RemoveItem, Transaction, TransactionId,
    test_support::{example_project, millis},
};
use se_protocol::CoreEvent;
use tokio::time::sleep;

const PATH: &str = "/projects/Example.seproj";
const SAVE_DELAY: Duration = Duration::from_secs(2);

struct Harness {
    actor: ProjectActorHandle,
    repository: InMemoryProjectRepository,
    events: Arc<RecordingEventSink>,
    start_revision: u64,
}

async fn start() -> Harness {
    let repository = InMemoryProjectRepository::default();
    let project = example_project();
    let start_revision = project.revision;
    repository.insert(PATH, project.clone(), Vec::new());
    let store = repository.open(PATH.as_ref()).await.unwrap();
    let events = Arc::new(RecordingEventSink::default());
    let deps = ActorDeps {
        events: events.clone(),
        clock: Arc::new(FixedClock::default()),
        ids: Arc::new(SequentialIds::default()),
        save_delay: SAVE_DELAY,
    };
    Harness {
        actor: ProjectActorHandle::spawn(project, Vec::new(), store, deps),
        repository,
        events,
        start_revision,
    }
}

fn move_text(index: u64, start_ms: i64) -> Transaction {
    let project = example_project();
    Transaction {
        id: TransactionId::from_test_index(index),
        actor: Actor::User,
        label: format!("Move {index}"),
        ops: vec![
            MoveItem {
                item: ItemId::from_test_index(3),
                track: project.sequences[0].tracks[1].id,
                start: millis(start_ms),
                expected_revision: None,
            }
            .into(),
        ],
    }
}

fn text_start(project: &Project) -> se_domain::Flicks {
    project.sequences[0].tracks[1]
        .items
        .iter()
        .find(|item| item.id == ItemId::from_test_index(3))
        .unwrap()
        .timeline_range
        .start()
}

#[tokio::test(start_paused = true)]
async fn commits_are_journaled_and_announced_in_order() {
    let harness = start().await;
    let first = harness.start_revision + 1;
    for (index, start_ms) in [(1, 8000), (2, 9000), (3, 10_000)] {
        harness
            .actor
            .apply(move_text(index, start_ms))
            .await
            .unwrap();
    }
    let folder = harness.repository.folder(PATH).unwrap();
    let revisions: Vec<u64> = folder.journal.iter().map(|entry| entry.revision).collect();
    assert_eq!(revisions, [first, first + 1, first + 2]);
    assert_eq!(folder.journal[2].transaction.label, "Move 3");

    let events = harness.events.take();
    let patched: Vec<u64> = events
        .iter()
        .filter_map(|event| match event {
            CoreEvent::ProjectPatched(patch) => Some(patch.revision),
            _ => None,
        })
        .collect();
    assert_eq!(patched, revisions);
    // Each patch is followed by the new undo state.
    let CoreEvent::HistoryChanged(history) = &events[1] else {
        panic!("a history change must follow the first patch: {events:?}");
    };
    assert!(history.history.can_undo);
    assert_eq!(history.history.undo_label.as_deref(), Some("Move 1"));

    let view = harness.actor.view().await.unwrap();
    assert_eq!(view.project.revision, first + 2);
    assert_eq!(text_start(&view.project), millis(10_000));
}

#[tokio::test(start_paused = true)]
async fn a_refused_transaction_changes_nothing() {
    let harness = start().await;
    let mut transaction = move_text(1, 8000);
    transaction.ops = vec![
        RemoveItem {
            item: ItemId::from_test_index(999),
            expected_revision: None,
        }
        .into(),
    ];
    let error = harness.actor.apply(transaction).await.unwrap_err();
    assert!(matches!(
        error,
        AppError::Domain(DomainError::ItemNotFound { .. })
    ));
    assert_eq!(harness.repository.folder(PATH).unwrap().journal, []);
    assert_eq!(harness.events.take(), []);
}

#[tokio::test(start_paused = true)]
async fn a_commit_that_cannot_be_journaled_is_not_committed() {
    let harness = start().await;
    harness.repository.fail_journal_appends(true);
    let error = harness.actor.apply(move_text(1, 8000)).await.unwrap_err();
    assert!(matches!(error, AppError::Storage(StorageError::Io { .. })));
    let view = harness.actor.view().await.unwrap();
    assert_eq!(view.project.revision, harness.start_revision);
    assert!(!view.history.can_undo);
    assert_eq!(harness.events.take(), []);

    harness.repository.fail_journal_appends(false);
    harness.actor.apply(move_text(2, 8000)).await.unwrap();
    // An undo that cannot be journaled leaves the history where it was.
    harness.repository.fail_journal_appends(true);
    assert!(harness.actor.undo().await.is_err());
    assert!(harness.actor.view().await.unwrap().history.can_undo);
}

#[tokio::test(start_paused = true)]
async fn the_snapshot_is_saved_once_after_the_last_commit() {
    let harness = start().await;
    harness.actor.apply(move_text(1, 8000)).await.unwrap();
    sleep(Duration::from_millis(1500)).await;
    harness.actor.apply(move_text(2, 9000)).await.unwrap();
    sleep(Duration::from_millis(1900)).await;
    // 3.4 s after the first commit but only 1.9 s after the last: nothing saved yet.
    assert_eq!(harness.repository.folder(PATH).unwrap().snapshot_saves, 0);
    sleep(Duration::from_millis(200)).await;
    let folder = harness.repository.folder(PATH).unwrap();
    assert_eq!(folder.snapshot_saves, 1);
    assert_eq!(folder.snapshot.revision, harness.start_revision + 2);
    assert!(
        folder.journal.is_empty(),
        "a saved snapshot truncates the journal"
    );
    sleep(Duration::from_secs(10)).await;
    assert_eq!(harness.repository.folder(PATH).unwrap().snapshot_saves, 1);
}

#[tokio::test(start_paused = true)]
async fn a_failed_snapshot_is_retried_and_the_journal_kept() {
    let harness = start().await;
    harness.repository.fail_snapshot_saves(true);
    harness.actor.apply(move_text(1, 8000)).await.unwrap();
    sleep(SAVE_DELAY + Duration::from_millis(100)).await;
    assert_eq!(harness.repository.folder(PATH).unwrap().journal.len(), 1);
    harness.repository.fail_snapshot_saves(false);
    sleep(SAVE_DELAY).await;
    let folder = harness.repository.folder(PATH).unwrap();
    assert_eq!(folder.snapshot_saves, 1);
    assert_eq!(folder.journal, []);
}

#[tokio::test(start_paused = true)]
async fn undo_and_redo_are_journaled_user_transactions() {
    let harness = start().await;
    assert!(matches!(
        harness.actor.undo().await,
        Err(AppError::Domain(DomainError::NothingToUndo))
    ));
    harness.actor.apply(move_text(1, 8000)).await.unwrap();
    let undone = harness.actor.undo().await.unwrap();
    assert_eq!(undone, harness.start_revision + 2);
    let view = harness.actor.view().await.unwrap();
    assert_eq!(text_start(&view.project), millis(1000));
    assert!(view.history.can_redo && !view.history.can_undo);
    assert_eq!(view.history.redo_label.as_deref(), Some("Move 1"));

    harness.actor.redo().await.unwrap();
    let view = harness.actor.view().await.unwrap();
    assert_eq!(text_start(&view.project), millis(8000));
    let journal = harness.repository.folder(PATH).unwrap().journal;
    assert_eq!(journal.len(), 3);
    assert!(
        journal
            .iter()
            .all(|entry| entry.transaction.actor == Actor::User)
    );
}

#[tokio::test(start_paused = true)]
async fn closing_saves_pending_commits_and_releases_the_lock() {
    let harness = start().await;
    harness.actor.apply(move_text(1, 8000)).await.unwrap();
    harness.actor.close().await.unwrap();
    let folder = harness.repository.folder(PATH).unwrap();
    assert!(!folder.locked);
    assert_eq!(folder.snapshot.revision, harness.start_revision + 1);
    assert_eq!(folder.journal, []);
    assert!(matches!(
        harness.actor.view().await,
        Err(AppError::ActorStopped { .. })
    ));
}

#[tokio::test(start_paused = true)]
async fn dropping_every_handle_closes_the_project() {
    let harness = start().await;
    harness.actor.apply(move_text(1, 8000)).await.unwrap();
    drop(harness.actor);
    sleep(Duration::from_millis(10)).await;
    let folder = harness.repository.folder(PATH).unwrap();
    assert!(!folder.locked);
    assert_eq!(folder.snapshot.revision, harness.start_revision + 1);
}
