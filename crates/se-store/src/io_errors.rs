//! Mapping IO failures to [`StorageError`] and running blocking work off the async runtime.

use std::{io, path::Path};

use se_app::ports::StorageError;

/// An IO failure while doing `action` on `path`. A full disk says what to do (prompt 29).
pub fn io_error(action: &'static str, path: &Path, error: &io::Error) -> StorageError {
    let reason = if error.kind() == io::ErrorKind::StorageFull {
        "the disk is full: free some space so SimpleEffect can save again".to_owned()
    } else {
        error.to_string()
    };
    StorageError::Io {
        action,
        path: path.to_path_buf(),
        reason,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_disk_says_what_to_do() {
        let error = io_error(
            "write",
            Path::new("project.json"),
            &io::Error::from(io::ErrorKind::StorageFull),
        );
        assert!(error.to_string().contains("free some space"), "{error}");
    }
}
