//! [`ProjectLock`]: one open copy of a project at a time, across processes.

use std::{
    fs::{File, OpenOptions, TryLockError},
    path::Path,
};

use se_app::ports::StorageError;

use crate::io_errors::io_error;

/// An exclusive OS lock on the project's `.lock` file.
///
/// The operating system releases it when the file is closed or the process dies, so a crash
/// never leaves a project locked. The file itself stays in the folder (deleting it would let two
/// processes lock two different files of the same name); its presence means nothing.
#[derive(Debug)]
pub struct ProjectLock {
    file: File,
}

impl ProjectLock {
    /// Takes the lock of the project whose lock file is `lock_path`.
    ///
    /// # Errors
    /// [`StorageError::Locked`] when another process (or another open handle) holds it;
    /// IO errors.
    pub fn acquire(lock_path: &Path, project_root: &Path) -> Result<Self, StorageError> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(lock_path)
            .map_err(|error| io_error("create the lock file", lock_path, &error))?;
        match file.try_lock() {
            Ok(()) => Ok(Self { file }),
            Err(TryLockError::WouldBlock) => Err(StorageError::Locked {
                path: project_root.to_path_buf(),
            }),
            Err(TryLockError::Error(error)) => Err(io_error("lock", lock_path, &error)),
        }
    }

    /// Releases the lock now (dropping it does too).
    ///
    /// # Errors
    /// IO errors.
    pub fn release(self, lock_path: &Path) -> Result<(), StorageError> {
        self.file
            .unlock()
            .map_err(|error| io_error("unlock", lock_path, &error))
    }
}
