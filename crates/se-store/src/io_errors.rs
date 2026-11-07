//! Mapping IO failures to [`StorageError`] and running blocking work off the async runtime.

use std::{io, path::Path};

use se_app::ports::StorageError;

/// An IO failure while doing `action` on `path`.
pub fn io_error(action: &'static str, path: &Path, error: &io::Error) -> StorageError {
    StorageError::Io {
        action,
        path: path.to_path_buf(),
        reason: error.to_string(),
    }
}

/// A SQLite failure while doing `action` on the database at `path`.
pub fn sqlite_error(action: &'static str, path: &Path, error: &rusqlite::Error) -> StorageError {
    StorageError::Io {
        action,
        path: path.to_path_buf(),
        reason: error.to_string(),
    }
}

/// Runs blocking filesystem or SQLite work on tokio's blocking pool.
pub async fn blocking<T: Send + 'static>(
    path: &Path,
    work: impl FnOnce() -> Result<T, StorageError> + Send + 'static,
) -> Result<T, StorageError> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(work)
        .await
        .unwrap_or_else(|error| {
            Err(StorageError::Io {
                action: "finish the storage work on",
                path,
                reason: format!("the background task failed: {error}"),
            })
        })
}

/// `path` as UTF-8 text, for SQLite columns and statements.
pub fn utf8_path<'a>(action: &'static str, path: &'a Path) -> Result<&'a str, StorageError> {
    path.to_str().ok_or_else(|| StorageError::Io {
        action,
        path: path.to_path_buf(),
        reason: "the path is not valid Unicode; move the project to a folder with a simpler name"
            .to_owned(),
    })
}
