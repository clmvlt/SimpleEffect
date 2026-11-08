//! `project.db`: the operation journal, the analyses of assets (`analyses.rs`), the AI tasks and
//! their event logs (`tasks.rs`), and tables later prompts fill (transcripts).

mod analyses;
mod tasks;

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, params};
use se_app::ports::{JournalEntry, StorageError};
use se_domain::Transaction;

use crate::{
    io_errors::{sqlite_error, utf8_path},
    sqlite::open_database,
};

/// Schema migrations of `project.db` (see [`open_database`]). Append, never edit.
const MIGRATIONS: &[&str] = &[SCHEMA_V1, SCHEMA_V2, SCHEMA_V3];

/// Version 1. `op_journal` holds the transactions committed since the last snapshot. The other
/// tables are created now so the schema has one history; their columns are refined by the
/// prompts that fill them (tasks: 17, transcripts: 21, analyses: 09) through new migrations.
const SCHEMA_V1: &str = "
CREATE TABLE op_journal (
    revision INTEGER PRIMARY KEY,
    transaction_id TEXT NOT NULL,
    committed_at INTEGER NOT NULL,
    format_version INTEGER NOT NULL,
    payload TEXT NOT NULL
);
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    created_at INTEGER NOT NULL,
    status TEXT NOT NULL,
    payload TEXT NOT NULL
);
CREATE TABLE task_events (
    task_id TEXT NOT NULL REFERENCES tasks (id) ON DELETE CASCADE,
    sequence INTEGER NOT NULL,
    at INTEGER NOT NULL,
    payload TEXT NOT NULL,
    PRIMARY KEY (task_id, sequence)
);
CREATE TABLE transcripts (
    asset_id TEXT PRIMARY KEY,
    language TEXT,
    created_at INTEGER NOT NULL,
    payload TEXT NOT NULL
);
CREATE VIRTUAL TABLE transcript_search USING fts5 (asset_id UNINDEXED, text);
CREATE TABLE analyses (
    asset_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    payload TEXT NOT NULL,
    PRIMARY KEY (asset_id, kind)
);
";

/// Version 2 (prompt 09): an analysis records the content it describes (`content_key`, the
/// fingerprint cache key: a relinked asset with other content is analysed again) and the version
/// of its JSON.
const SCHEMA_V2: &str = "
ALTER TABLE analyses ADD COLUMN content_key TEXT NOT NULL DEFAULT '';
ALTER TABLE analyses ADD COLUMN format_version INTEGER NOT NULL DEFAULT 1;
";

/// Version 3 (prompt 17): task rows and event rows record the version of their JSON, and events
/// the run that produced them (`tasks.payload` is a `se_protocol::Task`, `task_events.payload` a
/// `se_protocol::AgentEvent`).
const SCHEMA_V3: &str = "
ALTER TABLE tasks ADD COLUMN format_version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE task_events ADD COLUMN run INTEGER NOT NULL DEFAULT 0;
ALTER TABLE task_events ADD COLUMN format_version INTEGER NOT NULL DEFAULT 1;
";

/// Version of the JSON stored in `op_journal.payload` (the `Transaction` format, fixed by the
/// insta snapshots of se-domain). Bump it, and migrate old entries, if that format changes.
const JOURNAL_FORMAT_VERSION: u32 = 1;

/// An open `project.db`.
pub struct ProjectDatabase {
    connection: Connection,
    path: PathBuf,
}

impl ProjectDatabase {
    /// Opens (or creates) the database at `path`, upgrading its schema.
    ///
    /// # Errors
    /// [`StorageError::SchemaTooNew`]; IO errors.
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        Ok(Self {
            connection: open_database(path, MIGRATIONS)?,
            path: path.to_path_buf(),
        })
    }

    /// Appends one entry; durable when this returns (`synchronous = FULL`).
    ///
    /// # Errors
    /// IO errors, or an entry whose revision is already journaled.
    pub fn append(&self, entry: &JournalEntry) -> Result<(), StorageError> {
        let payload =
            serde_json::to_string(&entry.transaction).map_err(|error| StorageError::Io {
                action: "serialise a transaction for",
                path: self.path.clone(),
                reason: error.to_string(),
            })?;
        self.connection
            .execute(
                "INSERT INTO op_journal (revision, transaction_id, committed_at, format_version, payload)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    sql_revision(entry.revision),
                    entry.transaction.id.to_string(),
                    entry.committed_at,
                    JOURNAL_FORMAT_VERSION,
                    payload
                ],
            )
            .map(drop)
            .map_err(|error| sqlite_error("append to the journal of", &self.path, &error))
    }

    /// Entries with a revision greater than `revision`, in order.
    ///
    /// # Errors
    /// [`StorageError::SchemaTooNew`] for an entry of a newer format, [`StorageError::Corrupted`]
    /// for an unreadable one; IO errors.
    pub fn entries_since(&self, revision: u64) -> Result<Vec<JournalEntry>, StorageError> {
        let error =
            |error: rusqlite::Error| sqlite_error("read the journal of", &self.path, &error);
        let mut statement = self
            .connection
            .prepare(
                "SELECT revision, committed_at, format_version, payload FROM op_journal
                 WHERE revision > ?1 ORDER BY revision",
            )
            .map_err(error)?;
        let rows = statement
            .query_map([sql_revision(revision)], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, u32>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(error)?;
        let mut entries = Vec::new();
        for row in rows {
            let (revision, committed_at, format_version, payload) = row.map_err(error)?;
            entries.push(self.decode(revision, committed_at, format_version, &payload)?);
        }
        Ok(entries)
    }

    fn decode(
        &self,
        revision: i64,
        committed_at: i64,
        format_version: u32,
        payload: &str,
    ) -> Result<JournalEntry, StorageError> {
        if format_version > JOURNAL_FORMAT_VERSION {
            return Err(StorageError::SchemaTooNew {
                path: self.path.clone(),
                found: format_version,
                supported: JOURNAL_FORMAT_VERSION,
            });
        }
        let corrupted = |reason: String| StorageError::Corrupted {
            path: self.path.clone(),
            reason,
        };
        let transaction: Transaction = serde_json::from_str(payload).map_err(|error| {
            corrupted(format!("journal entry {revision} is unreadable: {error}"))
        })?;
        let revision = u64::try_from(revision)
            .map_err(|_| corrupted(format!("journal entry {revision} has a negative revision")))?;
        Ok(JournalEntry {
            revision,
            committed_at,
            transaction,
        })
    }

    /// Drops the entries included in a snapshot of `revision`.
    ///
    /// # Errors
    /// IO errors.
    pub fn truncate_through(&self, revision: u64) -> Result<(), StorageError> {
        self.connection
            .execute(
                "DELETE FROM op_journal WHERE revision <= ?1",
                [sql_revision(revision)],
            )
            .map(drop)
            .map_err(|error| sqlite_error("truncate the journal of", &self.path, &error))
    }

    /// Writes a consistent copy of this database to `target` (`VACUUM INTO`), which must not
    /// exist. Safe while another connection writes.
    ///
    /// # Errors
    /// IO errors.
    pub fn copy_to(&self, target: &Path) -> Result<(), StorageError> {
        let target_text = utf8_path("copy the database to", target)?;
        self.connection
            .execute("VACUUM INTO ?1", [target_text])
            .map(drop)
            .map_err(|error| sqlite_error("copy", &self.path, &error))
    }

    /// Number of journal entries (tests and diagnostics).
    ///
    /// # Errors
    /// IO errors.
    pub fn journal_len(&self) -> Result<u64, StorageError> {
        let count: Option<i64> = self
            .connection
            .query_row("SELECT COUNT(*) FROM op_journal", [], |row| row.get(0))
            .optional()
            .map_err(|error| sqlite_error("count the journal of", &self.path, &error))?;
        Ok(count
            .and_then(|count| u64::try_from(count).ok())
            .unwrap_or(0))
    }
}

/// Revisions as SQLite integers. Revisions grow by one per commit: they never reach `i64::MAX`.
fn sql_revision(revision: u64) -> i64 {
    i64::try_from(revision).unwrap_or(i64::MAX)
}
