//! Reading and writing `project.json`: atomic replacement, backups, migrations.

use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use se_app::ports::{LoadedSnapshot, RestoredSnapshot, StorageError};
use se_domain::{CURRENT_SCHEMA_VERSION, Project};

use crate::{
    document_migrations::{MIGRATIONS, MigrationError, migrate_document},
    io_errors::io_error,
    layout::{BACKUP_COUNT, ProjectLayout},
    platform,
};

/// Reads the snapshot of the project at `layout`, migrated to the current format. When
/// `project.json` is unreadable, the newest readable backup is read instead and the unreadable
/// file is renamed `project.json.corrupted` (so the next save does not rotate it into the
/// backups).
///
/// # Errors
/// [`StorageError::NotFound`] without `project.json`, [`StorageError::SchemaTooNew`],
/// [`StorageError::Corrupted`] when no backup is readable either, IO errors.
pub fn read_snapshot(layout: &ProjectLayout) -> Result<LoadedSnapshot, StorageError> {
    let path = layout.snapshot();
    let text = fs::read_to_string(&path).map_err(|error| match error.kind() {
        io::ErrorKind::NotFound => StorageError::NotFound {
            path: layout.root().to_path_buf(),
        },
        _ => io_error("read", &path, &error),
    })?;
    match parse_document(&text) {
        Ok(project) => Ok(LoadedSnapshot::current(project)),
        Err(DocumentProblem::TooNew { found, supported }) => Err(StorageError::SchemaTooNew {
            path,
            found,
            supported,
        }),
        Err(DocumentProblem::Unreadable(reason)) => restore_backup(layout, &reason),
    }
}

/// Reads the newest readable backup in place of an unreadable `project.json`.
fn restore_backup(layout: &ProjectLayout, problem: &str) -> Result<LoadedSnapshot, StorageError> {
    for number in 1..=BACKUP_COUNT {
        let backup = layout.backup(number);
        let Ok(text) = fs::read_to_string(&backup) else {
            continue;
        };
        let Ok(project) = parse_document(&text) else {
            tracing::warn!(backup = %backup.display(), "a backup is unreadable too");
            continue;
        };
        fs::rename(layout.snapshot(), layout.corrupted_snapshot())
            .map_err(|error| io_error("set aside", &layout.snapshot(), &error))?;
        tracing::warn!(%problem, backup = %backup.display(), "project.json was unreadable; restored a backup");
        return Ok(LoadedSnapshot {
            project,
            restored: Some(RestoredSnapshot {
                backup: format!("project.json.bak.{number}"),
                problem: problem.to_owned(),
            }),
        });
    }
    Err(StorageError::Corrupted {
        path: layout.snapshot(),
        reason: format!(
            "{problem}; no readable backup either (project.json.bak.1 to .bak.{BACKUP_COUNT}):              restore the folder from a copy of your own"
        ),
    })
}

/// Why a snapshot's text could not become a document.
enum DocumentProblem {
    /// Written by a newer version of the app.
    TooNew { found: u32, supported: u32 },
    /// Not JSON, not a project, or not upgradable.
    Unreadable(String),
}

/// Parses and migrates a snapshot's text.
fn parse_document(text: &str) -> Result<Project, DocumentProblem> {
    let document: serde_json::Value = serde_json::from_str(text)
        .map_err(|error| DocumentProblem::Unreadable(format!("invalid JSON ({error})")))?;
    let document = migrate_document(document, MIGRATIONS, CURRENT_SCHEMA_VERSION).map_err(
        |error| match error {
            MigrationError::TooNew { found, supported } => {
                DocumentProblem::TooNew { found, supported }
            }
            MigrationError::MissingVersion => DocumentProblem::Unreadable(
                "schemaVersion is missing or not a positive integer".to_owned(),
            ),
            MigrationError::Failed { from, reason } => DocumentProblem::Unreadable(format!(
                "cannot upgrade it from format version {from}: {reason}"
            )),
        },
    )?;
    serde_json::from_value(document).map_err(|error| DocumentProblem::Unreadable(error.to_string()))
}

/// Replaces the snapshot with `project`: the current file becomes `project.json.bak.1` (older
/// backups shift, the oldest is dropped), then the new JSON is written to a temporary file,
/// flushed to disk and renamed over `project.json`. A crash at any point leaves a complete
/// `project.json`.
///
/// # Errors
/// IO errors; the previous `project.json` is then intact.
pub fn write_snapshot(layout: &ProjectLayout, project: &Project) -> Result<(), StorageError> {
    let mut json = serde_json::to_vec_pretty(project).map_err(|error| StorageError::Io {
        action: "serialise the project for",
        path: layout.snapshot(),
        reason: error.to_string(),
    })?;
    json.push(b'\n');
    rotate_backups(layout).map_err(|error| io_error("back up", &layout.snapshot(), &error))?;
    write_atomically(&layout.snapshot(), &layout.snapshot_temp(), &json)
        .map_err(|error| io_error("write", &layout.snapshot(), &error))
}

/// `project.json` → `.bak.1` → `.bak.2` → … → `.bak.<BACKUP_COUNT>` (dropped next time).
fn rotate_backups(layout: &ProjectLayout) -> io::Result<()> {
    let current = layout.snapshot();
    if !current.exists() {
        return Ok(());
    }
    for number in (1..BACKUP_COUNT).rev() {
        let older = layout.backup(number);
        if older.exists() {
            fs::rename(&older, layout.backup(number + 1))?;
        }
    }
    // A copy, not a rename: `project.json` must exist at every instant.
    fs::copy(&current, layout.backup(1)).map(drop)
}

/// Writes `bytes` to `temp`, flushes it, renames it over `target` and flushes the folder.
pub fn write_atomically(target: &Path, temp: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = File::create(temp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(temp, target)?;
    match target.parent() {
        Some(folder) => platform::sync_directory(folder),
        None => Ok(()),
    }
}
