//! `SqliteAppLibrary` against a temporary `library.db`.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;

use se_app::ports::{AppLibraryRepository, RecentProjectRecord};
use se_domain::{Format, FormatPreset, FrameRate, ProjectId};
use se_store::SqliteAppLibrary;

fn record(name: &str, opened_at: i64) -> RecentProjectRecord {
    RecentProjectRecord {
        path: PathBuf::from(format!("/projects/{name}.seproj")),
        id: ProjectId::from_test_index(u64::try_from(opened_at).unwrap()),
        name: name.to_owned(),
        format: Format::from_preset(FormatPreset::Vertical9x16, FrameRate::FPS_29_97).unwrap(),
        last_opened_at: opened_at,
    }
}

#[tokio::test]
async fn recent_projects_are_listed_newest_first_and_survive_reopening() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("nested").join("library.db");
    let library = SqliteAppLibrary::open(&path).unwrap();
    library.upsert_recent(&record("Old", 1)).await.unwrap();
    library.upsert_recent(&record("New", 3)).await.unwrap();
    library.upsert_recent(&record("Middle", 2)).await.unwrap();
    let mut renamed = record("Old", 4);
    renamed.name = "Old renamed".to_owned();
    library.upsert_recent(&renamed).await.unwrap();
    drop(library);

    let library = SqliteAppLibrary::open(&path).unwrap();
    let names: Vec<String> = library
        .list_recent()
        .await
        .unwrap()
        .into_iter()
        .map(|record| record.name)
        .collect();
    assert_eq!(names, ["Old renamed", "New", "Middle"]);
    assert_eq!(library.list_recent().await.unwrap()[1], record("New", 3));

    library.remove_recent(&record("New", 3).path).await.unwrap();
    assert_eq!(library.list_recent().await.unwrap().len(), 2);
}

#[tokio::test]
async fn settings_are_key_values() {
    let folder = tempfile::tempdir().unwrap();
    let library = SqliteAppLibrary::open(&folder.path().join("library.db")).unwrap();
    assert_eq!(library.setting("a").await.unwrap(), None);
    library.set_setting("a", "1").await.unwrap();
    library.set_setting("a", "2").await.unwrap();
    assert_eq!(library.setting("a").await.unwrap().as_deref(), Some("2"));
}
