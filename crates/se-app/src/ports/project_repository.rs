//! [`ProjectRepository`] and [`ProjectStore`]: where project folders live (ADR-0006).
//!
//! The repository handles folders as wholes (create, locate, duplicate, trash). Opening a folder
//! takes its lock and yields a [`ProjectStore`], the only way to read and write that project's
//! snapshot and journal; the lock lasts until [`ProjectStore::close`] (or until the process dies).

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use se_domain::{Changeset, ChangesetEntry, ChangesetId, Project, TaskId, Transaction};

use super::{AnalysisStore, StorageError, TaskStore};

/// Project folders on disk (implemented by se-store).
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// The canonical absolute path of the project folder at `path`.
    ///
    /// # Errors
    /// [`StorageError::NotFound`] when `path` is not a folder holding a `project.json`.
    async fn locate(&self, path: &Path) -> Result<PathBuf, StorageError>;

    /// Creates the folder `<parent_dir>/<project name>.seproj` (and `parent_dir` if missing)
    /// holding `project` as its first snapshot, and returns its canonical path.
    ///
    /// # Errors
    /// [`StorageError::AlreadyExists`] when the folder exists; IO errors.
    async fn create(&self, parent_dir: &Path, project: &Project) -> Result<PathBuf, StorageError>;

    /// Creates a project folder next to `source` holding `project` (a copy of the source's
    /// document with a new id and name) and the source's other data (transcripts, analyses), but
    /// no journal. The folder is named after `project`, with a number added when the name is taken.
    /// Returns its canonical path.
    ///
    /// # Errors
    /// [`StorageError::NotFound`] for a missing source; IO errors.
    async fn duplicate(&self, source: &Path, project: &Project) -> Result<PathBuf, StorageError>;

    /// Takes the project's lock and opens it for reading and writing.
    ///
    /// # Errors
    /// [`StorageError::Locked`] when another process (or another open handle) holds the lock;
    /// [`StorageError::NotFound`]; [`StorageError::SchemaTooNew`] for a database written by a
    /// newer version.
    async fn open(&self, path: &Path) -> Result<Box<dyn ProjectStore>, StorageError>;

    /// Moves the project folder to the operating system's trash. Never deletes anything.
    ///
    /// # Errors
    /// [`StorageError::NotFound`]; IO errors (e.g. no trash on that drive).
    async fn trash(&self, path: &Path) -> Result<(), StorageError>;
}

/// One open, locked project folder. Owned by the project's actor.
#[async_trait]
pub trait ProjectStore: Send + Sync {
    /// Canonical absolute path of the project folder.
    fn path(&self) -> &Path;

    /// Reads `project.json`, migrated to the current document format.
    ///
    /// # Errors
    /// [`StorageError::SchemaTooNew`], [`StorageError::Corrupted`], IO errors.
    async fn load_snapshot(&self) -> Result<Project, StorageError>;

    /// Journal entries whose revision is greater than `revision`, in revision order.
    ///
    /// # Errors
    /// [`StorageError::Corrupted`] for an unreadable entry; IO errors.
    async fn read_journal_since(&self, revision: u64) -> Result<Vec<JournalEntry>, StorageError>;

    /// Durably appends one committed transaction to the journal. Returns only once the entry
    /// would survive a crash of the process.
    ///
    /// # Errors
    /// IO errors: the transaction must then be considered not committed.
    async fn append_journal(&self, entry: &JournalEntry) -> Result<(), StorageError>;

    /// Durably appends one committed transaction to the journal **and** its change of an AI
    /// task's changeset, in one atomic write: after a crash both are there, or neither.
    ///
    /// # Errors
    /// IO errors: the transaction must then be considered not committed.
    async fn append_journal_with_changeset(
        &self,
        entry: &JournalEntry,
        change: &ChangesetWrite,
    ) -> Result<(), StorageError>;

    /// The open changesets of AI tasks (those not reverted), each with its entries in order.
    ///
    /// # Errors
    /// [`StorageError::SchemaTooNew`] for an entry of a newer format, [`StorageError::Corrupted`]
    /// for an unreadable one; IO errors.
    async fn load_changesets(&self) -> Result<Vec<Changeset>, StorageError>;

    /// Atomically replaces `project.json` with `project` (keeping the previous versions as
    /// backups), then drops the journal entries it includes (revision ≤ `project.revision`).
    ///
    /// # Errors
    /// IO errors; the previous snapshot and the journal are then left intact.
    async fn save_snapshot(&self, project: &Project) -> Result<(), StorageError>;

    /// The analyses of this project's assets (same database, shareable outside the actor).
    fn analyses(&self) -> Arc<dyn AnalysisStore>;

    /// The AI tasks of this project (same database, shareable outside the actor).
    fn tasks(&self) -> Arc<dyn TaskStore>;

    /// Releases the lock. The store must not be used afterwards.
    ///
    /// # Errors
    /// IO errors (the lock is released anyway when the process ends).
    async fn close(&self) -> Result<(), StorageError>;
}

/// A committed transaction as stored in the operation journal (`op_journal`).
#[derive(Debug, Clone, PartialEq)]
pub struct JournalEntry {
    /// Revision of the project right after this transaction.
    pub revision: u64,
    /// When it was committed, in milliseconds since the Unix epoch.
    pub committed_at: i64,
    /// The transaction, replayed with `se_domain::commit` after a crash.
    pub transaction: Transaction,
}

/// How a commit changes an AI task's changeset (`changesets` and `changeset_entries` tables).
#[derive(Debug, Clone, PartialEq)]
pub enum ChangesetWrite {
    /// The commit is an edit of the task: `entry` is appended to its changeset, created (at
    /// `created_at`, milliseconds since the Unix epoch) if it does not exist yet.
    Record {
        /// The task's open changeset.
        changeset_id: ChangesetId,
        /// The task.
        task_id: TaskId,
        /// When the changeset was created (used only when it is new).
        created_at: i64,
        /// The committed transaction, its inverse and the revisions it left.
        entry: Box<ChangesetEntry>,
    },
    /// The commit reverts the changeset, which is closed (kept, marked reverted at `at`).
    Revert {
        /// The changeset reverted.
        changeset_id: ChangesetId,
        /// When (milliseconds since the Unix epoch).
        at: i64,
    },
}
