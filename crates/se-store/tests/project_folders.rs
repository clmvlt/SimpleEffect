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

/// The example project as the version-5 fixture holds it: the fast interview clip fits the
/// frame (version 5 had no focus to animate).
fn example_project_v5() -> se_domain::Project {
    let mut project = example_project_v6();
    let second_clip = &mut project.sequences[0].tracks[0].items[1].kind;
    if let se_domain::ItemKind::MediaClip(clip) = second_clip {
        clip.reframe = se_domain::Reframe::Fit;
    }
    project
}

/// The example project as the version-9 fixture holds it, once migrated: a skeleton kit
/// completed (yellow and dark palette, Inter, no logo, no source), its template under `other`
/// without library link nor brand role.
fn example_project_v9() -> se_domain::Project {
    let mut project = example_project();
    let style =
        se_domain::CaptionStyleId::parse(se_store::MIGRATED_BRAND_CAPTION_STYLE_ID).unwrap();
    let mut caption_style = se_domain::caption_style_from_preset("minimal", style).unwrap();
    "Inter".clone_into(&mut caption_style.font_family);
    project.brand_kit = Some(se_domain::BrandKit {
        source: None,
        name: se_store::MIGRATED_BRAND_KIT_NAME.to_owned(),
        fonts: se_domain::BrandFonts {
            heading: "Inter".to_owned(),
            body: "Inter".to_owned(),
        },
        colors: se_domain::BrandPalette {
            primary: se_domain::Color::opaque(0xff, 0xcc, 0x00),
            secondary: se_domain::Color::opaque(0x1a, 0x1a, 0x1a),
            accent: se_domain::Color::opaque(0xff, 0xcc, 0x00),
            text: se_domain::Color::WHITE,
            background: se_domain::Color::BLACK,
        },
        logos: Vec::new(),
        text_style: se_domain::TextStyle::default(),
        caption_style,
        intro_templates: Vec::new(),
        outro_templates: Vec::new(),
    });
    for template in &mut project.templates {
        template.category = se_domain::TemplateCategory::Other;
        template.library = None;
        template.source = template
            .source
            .replace(", category: 'lowerThird'", "")
            .replace(", brand: 'accent'", "");
        for param in &mut template.params {
            param.brand = None;
        }
    }
    project
}

/// The example project as the version-8 fixture holds it, once migrated: its template a skeleton
/// that draws nothing, its instance without values, centred and opaque.
fn example_project_v8() -> se_domain::Project {
    let mut project = example_project_v9();
    for template in &mut project.templates {
        *template = se_domain::MotionTemplate {
            id: template.id,
            name: template.name.clone(),
            description: String::new(),
            version: 1,
            source: se_store::MIGRATED_TEMPLATE_SOURCE.to_owned(),
            params: Vec::new(),
            default_duration: se_domain::Flicks::from_millis(5000),
            origin: se_domain::Origin::User,
            category: se_domain::TemplateCategory::Other,
            library: None,
        };
    }
    for item in project
        .sequences
        .iter_mut()
        .flat_map(|sequence| &mut sequence.tracks)
        .flat_map(|track| &mut track.items)
    {
        if let se_domain::ItemKind::MotionTemplate(template) = &mut item.kind {
            template.params.clear();
            template.transform = se_domain::Transform2D::default();
        }
    }
    project
}

/// The example project as the version-7 fixture holds it, once migrated: its caption « Merci
/// d'être là » spread equally over its two seconds, drawn with the migrated `minimal` style.
fn example_project_v7() -> se_domain::Project {
    let mut project = example_project_v8();
    let style = se_domain::CaptionStyleId::parse(se_store::MIGRATED_CAPTION_STYLE_ID).unwrap();
    project.caption_styles[0].id = style;
    for item in project
        .sequences
        .iter_mut()
        .flat_map(|sequence| &mut sequence.tracks)
        .flat_map(|track| &mut track.items)
    {
        let duration = item.timeline_range.duration();
        if let se_domain::ItemKind::Caption(caption) = &mut item.kind {
            caption.style_id = style;
            caption.words = se_domain::captions::distribute_words(
                &["Merci", "d'être", "là"],
                se_domain::Flicks::ZERO,
                duration,
                se_domain::captions::Spread::Equal,
            );
        }
    }
    project
}

/// The example project as the version-6 fixture holds it: no transcript summary (its
/// transcription settings are the defaults the migration gives).
fn example_project_v6() -> se_domain::Project {
    let mut project = example_project_v7();
    for asset in &mut project.assets {
        asset.analysis.transcript = None;
    }
    project
}

/// The example project as the version-4 fixture holds it: a plain white Inter title (regular
/// weight, no outline, no wrapping), an image fitted into the frame.
fn example_project_v4() -> se_domain::Project {
    let mut project = example_project_v5();
    for item in project
        .sequences
        .iter_mut()
        .flat_map(|sequence| &mut sequence.tracks)
        .flat_map(|track| &mut track.items)
    {
        match &mut item.kind {
            se_domain::ItemKind::Text(text) => {
                text.style = se_domain::TextStyle {
                    font_weight: 400,
                    font_size: se_domain::Animatable::constant(se_domain::DesignPixels(72.0)),
                    ..se_domain::TextStyle::default()
                };
            }
            se_domain::ItemKind::Image(image) => image.fit = se_domain::ImageFit::Fit,
            _ => {}
        }
    }
    project
}

/// The example project as the version-3 fixture holds it: no transitions, linear fades.
fn example_project_v3() -> se_domain::Project {
    let mut project = example_project_v4();
    for track in project
        .sequences
        .iter_mut()
        .flat_map(|sequence| &mut sequence.tracks)
    {
        track.transitions.clear();
        for item in &mut track.items {
            if let se_domain::ItemKind::MediaClip(clip) = &mut item.kind {
                clip.fade_in_shape = se_domain::FadeShape::Linear;
                clip.fade_out_shape = se_domain::FadeShape::Linear;
            }
        }
    }
    project
}

/// The example project as the version-2 fixture holds it: no analysis summaries, and the proxy
/// path the example used then.
fn example_project_v2() -> se_domain::Project {
    let mut project = example_project_v3();
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
async fn version_3_documents_load_through_the_migrations() {
    assert_eq!(load_fixture("project-v3.json").await, example_project_v3());
}

#[tokio::test]
async fn version_4_documents_load_through_the_migrations() {
    assert_eq!(load_fixture("project-v4.json").await, example_project_v4());
}

#[tokio::test]
async fn version_5_documents_load_through_the_migrations() {
    assert_eq!(load_fixture("project-v5.json").await, example_project_v5());
}

#[tokio::test]
async fn version_6_documents_load_through_the_migrations() {
    assert_eq!(load_fixture("project-v6.json").await, example_project_v6());
}

#[tokio::test]
async fn version_7_captions_become_equal_words_with_a_minimal_style() {
    assert_eq!(load_fixture("project-v7.json").await, example_project_v7());
}

#[tokio::test]
async fn version_8_templates_get_an_empty_definition() {
    assert_eq!(load_fixture("project-v8.json").await, example_project_v8());
}

#[tokio::test]
async fn version_9_brand_kits_are_completed() {
    assert_eq!(load_fixture("project-v9.json").await, example_project_v9());
}

#[tokio::test]
async fn version_10_documents_load_through_the_migrations() {
    // The example holds no tracked reframe: only the version number changes.
    assert_eq!(load_fixture("project-v10.json").await, example_project());
}

#[tokio::test]
async fn version_11_documents_load_as_they_are() {
    assert_eq!(load_fixture("project-v11.json").await, example_project());
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
            "INSERT INTO transcripts (asset_id, content_key, language, model, backend, created_at, format_version)
             VALUES ('a', 'k', 'fr', 'base', 'cpu', 0, 1)",
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
        "transcript_words",
        "transcript_segments",
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
            "INSERT INTO transcript_search (asset_id, segment, text) VALUES ('a', 0, 'bonjour tout le monde')",
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
