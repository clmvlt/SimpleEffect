//! Reading and writing `project.json`: atomic replacement, backups, migrations.

use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use se_app::ports::StorageError;
use se_domain::{CURRENT_SCHEMA_VERSION, Project};

use crate::{
    document_migrations::{MIGRATIONS, MigrationError, migrate_document},
    io_errors::io_error,
    layout::{BACKUP_COUNT, ProjectLayout},
    platform,
};

/// Reads the snapshot of the project at `layout`, migrated to the current format.
///
/// # Errors
/// [`StorageError::NotFound`] without `project.json`, [`StorageError::SchemaTooNew`],
/// [`StorageError::Corrupted`], IO errors.
pub fn read_snapshot(layout: &ProjectLayout) -> Result<Project, StorageError> {
    let path = layout.snapshot();
    let text = fs::read_to_string(&path).map_err(|error| match error.kind() {
        io::ErrorKind::NotFound => StorageError::NotFound {
            path: layout.root().to_path_buf(),
        },
        _ => io_error("read", &path, &error),
    })?;
    let corrupted = |reason: String| StorageError::Corrupted {
        path: path.clone(),
        reason: format!(
            "{reason}; the previous versions are in project.json.bak.1 to .bak.{BACKUP_COUNT}"
        ),
    };
    let document: serde_json::Value = serde_json::from_str(&text)
        .map_err(|error| corrupted(format!("invalid JSON ({error})")))?;
    let document = migrate_document(document, MIGRATIONS, CURRENT_SCHEMA_VERSION).map_err(
        |error| match error {
            MigrationError::TooNew { found, supported } => StorageError::SchemaTooNew {
                path: path.clone(),
                found,
                supported,
            },
            MigrationError::MissingVersion => {
                corrupted("schemaVersion is missing or not a positive integer".to_owned())
            }
            MigrationError::Failed { from, reason } => corrupted(format!(
                "cannot upgrade it from format version {from}: {reason}"
            )),
        },
    )?;
    serde_json::from_value(document).map_err(|error| corrupted(format!("{error}")))
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
