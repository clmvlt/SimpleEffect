//! Shared plumbing of the app library folders (prompt 25): versioned JSON files written
//! atomically, entry folders listed, folders moved to the trash.

use std::{
    fs,
    path::{Path, PathBuf},
};

use se_app::ports::StorageError;
use serde_json::Value;

use crate::{io_errors::io_error, platform, snapshot_file::write_atomically};

/// Version of the library files this build writes (`formatVersion`); newer files are skipped
/// when listing and refused when read alone.
pub const LIBRARY_FORMAT_VERSION: u64 = 1;

/// Writes `value` with its `formatVersion` as pretty JSON at `path`, atomically, creating its
/// folder.
pub fn write_json(path: &Path, mut value: Value) -> Result<(), StorageError> {
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "formatVersion".to_owned(),
            Value::from(LIBRARY_FORMAT_VERSION),
        );
    }
    let bytes = serde_json::to_vec_pretty(&value).map_err(|error| StorageError::Io {
        action: "encode",
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    write_bytes(path, &bytes)
}

/// Writes `bytes` at `path` atomically, creating its folder.
pub fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), StorageError> {
    let folder = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(folder).map_err(|error| io_error("create", folder, &error))?;
    let mut temp = path.as_os_str().to_owned();
    temp.push(".tmp");
    write_atomically(path, Path::new(&temp), bytes).map_err(|error| io_error("write", path, &error))
}

/// The JSON of `path` without its `formatVersion`, refused when it is newer than this build.
pub fn read_json(path: &Path, text: &str) -> Result<Value, StorageError> {
    let mut value: Value = serde_json::from_str(text).map_err(|error| corrupted(path, &error))?;
    let found = value
        .get("formatVersion")
        .and_then(Value::as_u64)
        .unwrap_or(LIBRARY_FORMAT_VERSION);
    if found > LIBRARY_FORMAT_VERSION {
        return Err(StorageError::SchemaTooNew {
            path: path.to_path_buf(),
            found: u32::try_from(found).unwrap_or(u32::MAX),
            supported: 1,
        });
    }
    if let Some(object) = value.as_object_mut() {
        object.remove("formatVersion");
    }
    Ok(value)
}

/// A file that cannot be understood.
pub fn corrupted(path: &Path, reason: &impl ToString) -> StorageError {
    StorageError::Corrupted {
        path: path.to_path_buf(),
        reason: reason.to_string(),
    }
}

/// The sub-folders of `root` (none when it does not exist yet).
pub fn entry_folders(root: &Path) -> Result<Vec<PathBuf>, StorageError> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(io_error("list", root, &error)),
    };
    let mut folders = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| io_error("list", root, &error))?;
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            folders.push(entry.path());
        }
    }
    folders.sort();
    Ok(folders)
}

/// Moves `folder` to the system trash when it exists.
pub fn trash_folder(folder: &Path) -> Result<(), StorageError> {
    if !folder.exists() {
        return Ok(());
    }
    platform::move_to_trash(folder).map_err(|error| StorageError::Io {
        action: "move to the trash",
        path: folder.to_path_buf(),
        reason: error.to_string(),
    })
}
