//! [`StorageError`]: why a persistence port failed.

use std::path::PathBuf;

/// A failure of [`super::ProjectRepository`], [`super::ProjectStore`] or
/// [`super::AppLibraryRepository`]. Messages name the file and say what to do: they reach users
/// and AI agents unchanged.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StorageError {
    /// No project folder at the path.
    #[error(
        "no SimpleEffect project at {}: choose a `.seproj` folder that contains a project.json",
        path.display()
    )]
    NotFound {
        /// The path that was tried.
        path: PathBuf,
    },
    /// Another process (or another open copy) holds the project's lock.
    #[error(
        "the project at {} is already open in another SimpleEffect window; close it there first",
        path.display()
    )]
    Locked {
        /// The project folder.
        path: PathBuf,
    },
    /// The folder a new project would use already exists.
    #[error(
        "a folder already exists at {}: choose another project name or location",
        path.display()
    )]
    AlreadyExists {
        /// The folder that is in the way.
        path: PathBuf,
    },
    /// A file was written by a newer version of SimpleEffect.
    #[error(
        "{} was saved by a newer SimpleEffect (format version {found}; this version reads up to \
         {supported}): update SimpleEffect to open it",
        path.display()
    )]
    SchemaTooNew {
        /// The file.
        path: PathBuf,
        /// Version found in the file.
        found: u32,
        /// Newest version this build reads.
        supported: u32,
    },
    /// A file exists but cannot be understood.
    #[error("cannot read {}: {reason}", path.display())]
    Corrupted {
        /// The file.
        path: PathBuf,
        /// What is wrong with it, and where backups are when there are some.
        reason: String,
    },
    /// The operating system refused a read or a write.
    #[error("cannot {action} {}: {reason}", path.display())]
    Io {
        /// What was being done (`write`, `create the folder`…).
        action: &'static str,
        /// The file or folder.
        path: PathBuf,
        /// The system's explanation.
        reason: String,
    },
}
