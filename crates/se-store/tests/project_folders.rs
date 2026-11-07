//! `FsProjectRepository` and `FsProjectStore` against temporary folders: layout, atomic writes
//! and backups, journal, lock, migrations, duplication.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{fs, path::Path};

use se_app::ports::{JournalEntry, ProjectRepository, StorageError};
use se_domain::{
    Actor, Project, ProjectId, RenameProject, Transaction, TransactionId,
    test_support::example_project,
};
use se_store::{FsProjectRepository, ProjectDatabase, ProjectLayout};
use tempfile::TempDir;

fn rename_entry(revision: u64, name: &str) -> JournalEntry {
    JournalEntry {
        revision,
        committed_at: 1_700_000_000_000,
        transaction: Transaction {
            id: TransactionId::from_test_index(revision),
            actor: Actor::User,
            label: "Rename project".to_owned(),
            ops: vec![
                RenameProject {
                    name: name.to_owned(),
                }
                .into(),
            ],
        },
    }
}

async fn created(parent: &TempDir) -> (FsProjectRepository, std::path::PathBuf) {
    let repository = FsProjectRepository;
    let path = repository
        .create(&parent.path().join("projects"), &example_project())
        .await
        .unwrap();
    (repository, path)
}

#[tokio::test]
async fn creating_writes_the_documented_layout() {
    let parent = tempfile::tempdir().unwrap();
    let (repository, path) = created(&parent).await;
    assert_eq!(path.file_name().unwrap(), "Example.seproj");
    let layout = ProjectLayout::new(&path);
    assert!(layout.snapshot().is_file());
    assert!(layout.database().is_file());
    assert!(layout.cache().is_dir());
    assert!(layout.exports().is_dir());
    let json = fs::read_to_string(layout.snapshot()).unwrap();
    assert!(json.starts_with("{\n  \"id\""), "project.json is indented");
    assert_eq!(repository.locate(&path).await.unwrap(), path);

    let store = repository.open(&path).await.unwrap();
    assert_eq!(store.load_snapshot().await.unwrap(), example_project());

    let again = repository
        .create(&parent.path().join("projects"), &example_project())
        .await;
    assert!(matches!(again, Err(StorageError::AlreadyExists { .. })));
}

#[tokio::test]
async fn folders_without_a_project_are_not_found() {
    let parent = tempfile::tempdir().unwrap();
    let repository = FsProjectRepository;
    for path in [parent.path().to_path_buf(), parent.path().join("missing")] {
        assert!(matches!(
            repository.locate(&path).await,
            Err(StorageError::NotFound { .. })
        ));
        assert!(matches!(
            repository.open(&path).await,
            Err(StorageError::NotFound { .. })
        ));
    }
}

#[tokio::test]
async fn snapshots_are_replaced_atomically_and_keep_three_backups() {
    let parent = tempfile::tempdir().unwrap();
    let (repository, path) = created(&parent).await;
    let store = repository.open(&path).await.unwrap();
    let mut project = example_project();
    for revision in 43..=47 {
        project.revision = revision;
        store.save_snapshot(&project).await.unwrap();
    }
    let layout = ProjectLayout::new(&path);
    let revision_of = |file: &Path| -> u64 {
        let json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(file).unwrap()).unwrap();
        json["revision"].as_u64().unwrap()
    };
    assert_eq!(revision_of(&layout.snapshot()), 47);
    assert_eq!(revision_of(&layout.backup(1)), 46);
    assert_eq!(revision_of(&layout.backup(2)), 45);
    assert_eq!(revision_of(&layout.backup(3)), 44);
    assert!(!layout.backup(4).exists());
    assert!(!layout.snapshot_temp().exists());
}

#[tokio::test]
async fn the_journal_survives_reopening_and_is_truncated_by_snapshots() {
    let parent = tempfile::tempdir().unwrap();
    let (repository, path) = created(&parent).await;
    let store = repository.open(&path).await.unwrap();
    for (revision, name) in [(43, "A"), (44, "B"), (45, "C")] {
        store
            .append_journal(&rename_entry(revision, name))
            .await
            .unwrap();
    }
    // A crash: the store is dropped without closing (the OS releases the lock).
    drop(store);

    let store = repository.open(&path).await.unwrap();
    let journal = store.read_journal_since(42).await.unwrap();
    assert_eq!(
        journal,
        [
            rename_entry(43, "A"),
            rename_entry(44, "B"),
            rename_entry(45, "C")
        ]
    );
    assert_eq!(store.read_journal_since(44).await.unwrap().len(), 1);

    let mut project = example_project();
    project.revision = 44;
    store.save_snapshot(&project).await.unwrap();
    assert_eq!(
        store.read_journal_since(0).await.unwrap(),
        [rename_entry(45, "C")]
    );
    let duplicate = store.append_journal(&rename_entry(45, "again")).await;
    assert!(matches!(duplicate, Err(StorageError::Io { .. })));
}

#[tokio::test]
async fn an_open_project_cannot_be_opened_again_until_released() {
    let parent = tempfile::tempdir().unwrap();
    let (repository, path) = created(&parent).await;
    let store = repository.open(&path).await.unwrap();
    assert!(matches!(
        repository.open(&path).await,
        Err(StorageError::Locked { .. })
    ));
    store.close().await.unwrap();
    let reopened = repository.open(&path).await.unwrap();
    drop(reopened);
    repository.open(&path).await.unwrap();
}

/// Loads `tests/fixtures/<name>` as the snapshot of a new project.
async fn load_fixture(name: &str) -> se_domain::Project {
    let parent = tempfile::tempdir().unwrap();
    let (repository, path) = created(&parent).await;
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    fs::copy(fixture, ProjectLayout::new(&path).snapshot()).unwrap();
    let store = repository.open(&path).await.unwrap();
    store.load_snapshot().await.unwrap()
}

/// The example project as the version-2 fixture holds it: no analysis summaries, and the proxy
/// path the example used then.
fn example_project_v2() -> se_domain::Project {
    let mut project = example_project();
    for asset in &mut project.assets {
        asset.analysis = se_domain::AssetAnalysis::default();
        if let Some(proxy) = asset.derived.proxy.as_mut() {
            proxy.relative_path = format!("cache/proxies/{}.mp4", asset.id);
        }
    }
    project
}

#[tokio::test]
async fn version_1_documents_load_through_the_migrations() {
    // Version 1 knew neither fingerprints nor codec levels, tags, pixel formats or audio profiles.
    let mut expected = example_project_v2();
    for asset in &mut expected.assets {
        asset.fingerprint = None;
        let info = asset.media_info.as_mut().unwrap();
        if let Some(video) = info.video_stream.as_mut() {
            video.level = None;
            video.codec_tag = None;
            video.pixel_format = None;
        }
        for audio in &mut info.audio_streams {
            audio.profile = None;
        }
    }
    assert_eq!(load_fixture("project-v1.json").await, expected);
}

#[tokio::test]
async fn version_2_documents_load_through_the_migrations() {
    assert_eq!(load_fixture("project-v2.json").await, example_project_v2());
}

#[tokio::test]
async fn version_3_documents_load_as_they_are() {
    assert_eq!(load_fixture("project-v3.json").await, example_project());
}

#[tokio::test]
async fn newer_or_broken_documents_are_refused_with_explanations() {
    let parent = tempfile::tempdir().unwrap();
    let (repository, path) = created(&parent).await;
    let snapshot = ProjectLayout::new(&path).snapshot();
    let store = repository.open(&path).await.unwrap();

    let mut newer: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&snapshot).unwrap()).unwrap();
    newer["schemaVersion"] = serde_json::json!(999);
    fs::write(&snapshot, newer.to_string()).unwrap();
    let error = store.load_snapshot().await.unwrap_err();
    assert!(matches!(
        error,
        StorageError::SchemaTooNew {
            found: 999,
            supported: se_domain::CURRENT_SCHEMA_VERSION,
            ..
        }
    ));
    assert!(error.to_string().contains("update SimpleEffect"));

    fs::write(&snapshot, "{ not json").unwrap();
    let error = store.load_snapshot().await.unwrap_err();
    assert!(matches!(error, StorageError::Corrupted { .. }));
    assert!(error.to_string().contains("project.json.bak.1"));
}

#[tokio::test]
async fn duplicates_get_a_numbered_folder_their_data_and_no_journal() {
    let parent = tempfile::tempdir().unwrap();
    let (repository, path) = created(&parent).await;
    let store = repository.open(&path).await.unwrap();
    store.append_journal(&rename_entry(43, "A")).await.unwrap();
    let source_db = rusqlite::Connection::open(ProjectLayout::new(&path).database()).unwrap();
    source_db
        .execute(
            "INSERT INTO transcripts (asset_id, language, created_at, payload) VALUES ('a', 'fr', 0, '{}')",
            [],
        )
        .unwrap();

    let copy = Project {
        id: ProjectId::from_test_index(500),
        ..example_project()
    };
    let copy_path = repository.duplicate(&path, &copy).await.unwrap();
    assert_eq!(copy_path.file_name().unwrap(), "Example 2.seproj");
    let copy_store = repository.open(&copy_path).await.unwrap();
    assert_eq!(copy_store.load_snapshot().await.unwrap(), copy);
    assert_eq!(copy_store.read_journal_since(0).await.unwrap(), []);
    let copy_db = rusqlite::Connection::open(ProjectLayout::new(&copy_path).database()).unwrap();
    let transcripts: i64 = copy_db
        .query_row("SELECT COUNT(*) FROM transcripts", [], |row| row.get(0))
        .unwrap();
    assert_eq!(transcripts, 1);
    // The source keeps its journal.
    assert_eq!(store.read_journal_since(0).await.unwrap().len(), 1);
    let third = repository.duplicate(&path, &copy).await.unwrap();
    assert_eq!(third.file_name().unwrap(), "Example 3.seproj");
}

#[tokio::test]
async fn the_project_database_has_every_table_and_full_text_search() {
    let parent = tempfile::tempdir().unwrap();
    let (_, path) = created(&parent).await;
    let database_path = ProjectLayout::new(&path).database();
    let database = ProjectDatabase::open(&database_path).unwrap();
    assert_eq!(database.journal_len().unwrap(), 0);
    let connection = rusqlite::Connection::open(&database_path).unwrap();
    for table in [
        "op_journal",
        "tasks",
        "task_events",
        "transcripts",
        "analyses",
    ] {
        let count: i64 = connection
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0, "{table} exists and is empty");
    }
    connection
        .execute(
            "INSERT INTO transcript_search (asset_id, text) VALUES ('a', 'bonjour tout le monde')",
            [],
        )
        .unwrap();
    let found: String = connection
        .query_row(
            "SELECT asset_id FROM transcript_search WHERE transcript_search MATCH 'monde'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(found, "a");
}

/// Moves a folder to the real OS trash of the machine running the tests, so it does not run by
/// default: `cargo test -p se-store -- --ignored`.
#[tokio::test]
#[ignore = "moves a temporary folder to the real OS trash"]
async fn trashing_moves_the_folder_to_the_os_trash() {
    let parent = tempfile::tempdir().unwrap();
    let (repository, path) = created(&parent).await;
    repository.trash(&path).await.unwrap();
    assert!(!path.exists());
}
