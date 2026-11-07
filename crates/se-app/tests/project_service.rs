//! The project use cases against in-memory ports.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use se_app::{
    ProjectService, ProjectServiceConfig, ProjectServiceDeps,
    ports::{AppLibraryRepository, Clock, JournalEntry, ProjectRepository},
    test_support::{
        FixedClock, InMemoryAppLibrary, InMemoryProjectRepository, RecordingEventSink,
        SequentialIds,
    },
};
use se_domain::{
    Actor, Format, FormatPreset, FrameRate, RenameProject, Transaction, TransactionId,
    test_support::example_project,
};
use se_protocol::{CoreEvent, ErrorCode, ProtocolError};

const PARENT: &str = "/projects";

struct Harness {
    service: ProjectService,
    repository: InMemoryProjectRepository,
    library: Arc<InMemoryAppLibrary>,
    events: Arc<RecordingEventSink>,
    clock: Arc<FixedClock>,
}

fn harness() -> Harness {
    let repository = InMemoryProjectRepository::default();
    let library = Arc::new(InMemoryAppLibrary::default());
    let events = Arc::new(RecordingEventSink::default());
    let clock = Arc::new(FixedClock::default());
    let deps = ProjectServiceDeps {
        repository: Arc::new(repository.clone()),
        library: library.clone(),
        events: events.clone(),
        clock: clock.clone(),
        ids: Arc::new(SequentialIds::default()),
    };
    let config = ProjectServiceConfig {
        default_parent_dir: PathBuf::from("/home/me/Documents/SimpleEffect"),
        save_delay: Duration::from_secs(2),
    };
    Harness {
        service: ProjectService::new(deps, config),
        repository,
        library,
        events,
        clock,
    }
}

fn hd() -> Format {
    Format::from_preset(FormatPreset::Hd16x9, FrameRate::FPS_25).unwrap()
}

fn code(error: se_app::AppError) -> ErrorCode {
    ProtocolError::from(error).code
}

fn example_path() -> PathBuf {
    Path::new(PARENT).join("Example.seproj")
}

#[tokio::test]
async fn creating_a_project_opens_it_with_a_default_sequence() {
    let harness = harness();
    let defaults = harness.service.new_project_defaults().await.unwrap();
    assert_eq!(defaults.parent_dir, "/home/me/Documents/SimpleEffect");
    assert_eq!(defaults.frame_rates, FrameRate::STANDARD);

    let session = harness
        .service
        .create_project("  Film  ", Path::new(PARENT), hd())
        .await
        .unwrap();
    assert_eq!(session.path, "/projects/Film.seproj");
    let snapshot = &session.snapshot;
    assert_eq!(snapshot.project.name, "Film");
    assert_eq!(snapshot.sequences.len(), 1);
    let names: Vec<_> = snapshot.tracks.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["V1", "V2", "A1", "A2"]);
    assert!(
        harness
            .repository
            .folder("/projects/Film.seproj")
            .unwrap()
            .locked
    );

    let events = harness.events.take();
    assert!(matches!(&events[..], [CoreEvent::ProjectOpened(opened)] if opened == &session));
    let recent = harness.service.list_recent().await.unwrap();
    assert_eq!(recent[0].name, "Film");
    assert!(recent[0].available);
    let defaults = harness.service.new_project_defaults().await.unwrap();
    assert_eq!(
        defaults.parent_dir, PARENT,
        "the last folder used is proposed next"
    );
}

#[tokio::test]
async fn creation_refuses_empty_names_invalid_formats_and_taken_folders() {
    let harness = harness();
    let parent = Path::new(PARENT);
    let empty = harness.service.create_project(" ", parent, hd()).await;
    assert_eq!(code(empty.unwrap_err()), ErrorCode::InvalidRequest);
    let odd = Format {
        width: 1921,
        ..hd()
    };
    let invalid = harness.service.create_project("Odd", parent, odd).await;
    assert_eq!(code(invalid.unwrap_err()), ErrorCode::InvalidRequest);
    harness
        .service
        .create_project("Film", parent, hd())
        .await
        .unwrap();
    let taken = harness.service.create_project("Film", parent, hd()).await;
    assert_eq!(code(taken.unwrap_err()), ErrorCode::ProjectAlreadyExists);
}

#[tokio::test]
async fn opening_replays_the_journal_and_saves_the_recovered_state() {
    let harness = harness();
    let snapshot = example_project();
    let entry = JournalEntry {
        revision: snapshot.revision + 1,
        committed_at: 0,
        transaction: Transaction {
            id: TransactionId::from_test_index(1),
            actor: Actor::User,
            label: "Rename".to_owned(),
            ops: vec![
                RenameProject {
                    name: "Recovered".to_owned(),
                }
                .into(),
            ],
        },
    };
    harness
        .repository
        .insert(example_path(), snapshot.clone(), vec![entry]);
    let session = harness.service.open_project(&example_path()).await.unwrap();
    assert_eq!(session.snapshot.revision, snapshot.revision + 1);
    assert_eq!(session.snapshot.project.name, "Recovered");
    let folder = harness.repository.folder(example_path()).unwrap();
    assert_eq!(folder.snapshot_saves, 1);
    assert_eq!(folder.journal, []);
}

#[tokio::test]
async fn a_project_open_elsewhere_is_refused_but_reopening_here_is_not() {
    let harness = harness();
    harness
        .repository
        .insert(example_path(), example_project(), Vec::new());
    let first = harness.service.open_project(&example_path()).await.unwrap();
    let again = harness.service.open_project(&example_path()).await.unwrap();
    assert_eq!(first, again);

    let other = Path::new(PARENT).join("Other.seproj");
    let mut project = example_project();
    project.id = se_domain::ProjectId::from_test_index(77);
    harness
        .repository
        .insert(other.clone(), project, Vec::new());
    // Another process holds the lock.
    let held = harness.repository.open(&other).await.unwrap();
    let locked = harness.service.open_project(&other).await;
    assert_eq!(code(locked.unwrap_err()), ErrorCode::ProjectLocked);
    held.close().await.unwrap();
    harness.service.open_project(&other).await.unwrap();
}

#[tokio::test]
async fn a_copy_of_an_open_project_made_outside_the_app_is_refused() {
    let harness = harness();
    harness
        .repository
        .insert(example_path(), example_project(), Vec::new());
    let copy = Path::new(PARENT).join("Example copy.seproj");
    harness
        .repository
        .insert(copy.clone(), example_project(), Vec::new());
    harness.service.open_project(&example_path()).await.unwrap();
    let refused = harness.service.open_project(&copy).await;
    assert_eq!(code(refused.unwrap_err()), ErrorCode::ProjectLocked);
    assert!(!harness.repository.folder(&copy).unwrap().locked);
}

#[tokio::test]
async fn closing_saves_releases_and_announces() {
    let harness = harness();
    let session = harness
        .service
        .create_project("Film", Path::new(PARENT), hd())
        .await
        .unwrap();
    let id = session.snapshot.project.id;
    harness
        .service
        .create_sequence(id, "Short", None)
        .await
        .unwrap();
    harness.events.take();
    harness.service.close_project(id).await.unwrap();
    let folder = harness.repository.folder("/projects/Film.seproj").unwrap();
    assert!(!folder.locked);
    assert_eq!(folder.snapshot.sequences.len(), 2);
    assert_eq!(
        harness.events.take(),
        [CoreEvent::ProjectClosed { project_id: id }]
    );
    let undo = harness.service.undo(id).await;
    assert_eq!(code(undo.unwrap_err()), ErrorCode::ProjectNotOpen);
}

#[tokio::test]
async fn sequences_can_be_created_undone_and_redone() {
    let harness = harness();
    let session = harness
        .service
        .create_project("Film", Path::new(PARENT), hd())
        .await
        .unwrap();
    let id = session.snapshot.project.id;
    let nothing = harness.service.undo(id).await;
    assert_eq!(code(nothing.unwrap_err()), ErrorCode::NothingToUndo);
    let vertical = Format::from_preset(FormatPreset::Vertical9x16, FrameRate::FPS_30).unwrap();
    let sequence = harness
        .service
        .create_sequence(id, "Short", Some(vertical))
        .await
        .unwrap();
    let after = harness.service.project_session(id).await.unwrap();
    let created = after.snapshot.sequences.iter().find(|s| s.id == sequence);
    assert_eq!(created.unwrap().format, vertical);
    assert_eq!(after.history.undo_label.as_deref(), Some("Create sequence"));

    harness.service.undo(id).await.unwrap();
    let undone = harness.service.project_session(id).await.unwrap();
    assert_eq!(undone.snapshot.sequences.len(), 1);
    assert!(undone.history.can_redo);
    harness.service.redo(id).await.unwrap();
    let redone = harness.service.project_session(id).await.unwrap();
    assert_eq!(redone.snapshot.sequences.len(), 2);
    let empty = harness.service.create_sequence(id, "", None).await;
    assert_eq!(code(empty.unwrap_err()), ErrorCode::InvalidRequest);
}

#[tokio::test]
async fn renaming_a_closed_project_goes_through_a_temporary_actor() {
    let harness = harness();
    harness
        .repository
        .insert(example_path(), example_project(), Vec::new());
    harness.service.open_project(&example_path()).await.unwrap();
    let opened_at = harness.clock.now_unix_millis();
    harness
        .service
        .close_project(example_project().id)
        .await
        .unwrap();
    harness.clock.advance(60_000);
    harness.events.take();

    let renamed = harness
        .service
        .rename_project(&example_path(), "Renamed")
        .await
        .unwrap();
    assert_eq!(renamed.name, "Renamed");
    assert_eq!(renamed.last_opened_at, opened_at, "renaming keeps the date");
    let folder = harness.repository.folder(example_path()).unwrap();
    assert_eq!(folder.snapshot.name, "Renamed");
    assert!(!folder.locked);
    assert!(
        harness.events.take().is_empty(),
        "temporary actors stay silent"
    );
}

#[tokio::test]
async fn renaming_an_open_project_patches_its_header() {
    let harness = harness();
    harness
        .repository
        .insert(example_path(), example_project(), Vec::new());
    harness.service.open_project(&example_path()).await.unwrap();
    harness.events.take();
    harness
        .service
        .rename_project(&example_path(), "Live")
        .await
        .unwrap();
    let events = harness.events.take();
    let CoreEvent::ProjectPatched(patch) = &events[0] else {
        panic!("renaming an open project must patch it: {events:?}");
    };
    assert_eq!(patch.upserts.project.as_ref().unwrap().name, "Live");
    let empty = harness.service.rename_project(&example_path(), " ").await;
    assert_eq!(code(empty.unwrap_err()), ErrorCode::InvalidRequest);
}

#[tokio::test]
async fn duplicating_copies_the_document_with_a_new_id() {
    let harness = harness();
    harness
        .repository
        .insert(example_path(), example_project(), Vec::new());
    let copy = harness
        .service
        .duplicate_project(&example_path(), "Example")
        .await
        .unwrap();
    assert_eq!(copy.path, "/projects/Example 2.seproj");
    assert_ne!(copy.id, example_project().id);
    let folder = harness.repository.folder(&copy.path).unwrap();
    assert_eq!(folder.snapshot.id, copy.id);
    assert_eq!(folder.snapshot.sequences, example_project().sequences);
    assert!(!harness.repository.folder(example_path()).unwrap().locked);
    let recent = harness.service.list_recent().await.unwrap();
    assert_eq!(recent[0].id, copy.id);
}

#[tokio::test]
async fn trashing_closes_the_project_and_forgets_it() {
    let harness = harness();
    let session = harness
        .service
        .create_project("Film", Path::new(PARENT), hd())
        .await
        .unwrap();
    harness.events.take();
    let path = PathBuf::from(&session.path);
    harness.service.trash_project(&path).await.unwrap();
    assert_eq!(harness.repository.trashed(), [path]);
    assert_eq!(
        harness.events.take(),
        [CoreEvent::ProjectClosed {
            project_id: session.snapshot.project.id
        }]
    );
    assert_eq!(harness.service.list_recent().await.unwrap(), []);
}

#[tokio::test]
async fn missing_folders_are_listed_unavailable_and_trashing_them_forgets_them() {
    let harness = harness();
    let session = harness
        .service
        .create_project("Film", Path::new(PARENT), hd())
        .await
        .unwrap();
    let id = session.snapshot.project.id;
    harness.service.close_project(id).await.unwrap();
    let path = PathBuf::from(&session.path);
    // The folder disappears outside the app.
    harness.repository.trash(&path).await.unwrap();
    let recent = harness.service.list_recent().await.unwrap();
    assert!(!recent[0].available);
    harness.service.trash_project(&path).await.unwrap();
    assert_eq!(harness.library.list_recent().await.unwrap(), []);
}
