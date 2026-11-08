//! The `changesets` and `changeset_entries` tables (schema v4, prompt 19): the edits of each AI
//! task (`se_domain::Changeset`), written in the same SQLite transaction as their journal entry
//! so a crash never separates a commit from its changeset.

use rusqlite::{Connection, params};
use se_app::ports::{ChangesetWrite, JournalEntry, StorageError};
use se_domain::{Changeset, ChangesetEntry, ChangesetId, TaskId};

use super::{JOURNAL_FORMAT_VERSION, ProjectDatabase};
use crate::io_errors::sqlite_error;

impl ProjectDatabase {
    /// Appends `entry` to the journal and applies `change` to the changesets, atomically.
    ///
    /// # Errors
    /// IO errors (nothing is written then), an entry whose revision is already journaled, a
    /// revert of an unknown changeset.
    pub fn append_with_changeset(
        &self,
        entry: &JournalEntry,
        change: &ChangesetWrite,
    ) -> Result<(), StorageError> {
        let error =
            |error: rusqlite::Error| sqlite_error("append to the journal of", &self.path, &error);
        // Unchecked: the connection is shared behind a mutex, never used by two writers at once.
        let transaction = self.connection.unchecked_transaction().map_err(error)?;
        self.insert_journal_entry(&transaction, entry)?;
        match change {
            ChangesetWrite::Record {
                changeset_id,
                task_id,
                created_at,
                entry: changeset_entry,
            } => self.record_entry(
                &transaction,
                *changeset_id,
                *task_id,
                *created_at,
                changeset_entry,
            )?,
            ChangesetWrite::Revert { changeset_id, at } => {
                let updated = transaction
                    .execute(
                        "UPDATE changesets SET reverted_at = ?2 WHERE id = ?1",
                        params![changeset_id.to_string(), at],
                    )
                    .map_err(error)?;
                if updated == 0 {
                    return Err(StorageError::Corrupted {
                        path: self.path.clone(),
                        reason: format!("no changeset {changeset_id} to mark reverted"),
                    });
                }
            }
        }
        transaction.commit().map_err(error)
    }

    /// The changesets not reverted, oldest first, each with its entries in order.
    ///
    /// # Errors
    /// [`StorageError::SchemaTooNew`] for an entry of a newer format, [`StorageError::Corrupted`]
    /// for an unreadable one; IO errors.
    pub fn open_changesets(&self) -> Result<Vec<Changeset>, StorageError> {
        let error =
            |error: rusqlite::Error| sqlite_error("read the AI changesets of", &self.path, &error);
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, task_id FROM changesets WHERE reverted_at IS NULL
                 ORDER BY created_at, id",
            )
            .map_err(error)?;
        let heads = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(error)?;
        heads
            .into_iter()
            .map(|(id, task_id)| {
                let changeset_id = self.parse_id(&id, ChangesetId::parse)?;
                let task_id = self.parse_id(&task_id, TaskId::parse)?;
                let mut changeset = Changeset::new(changeset_id, task_id);
                changeset.entries = self.changeset_entries(&id)?;
                Ok(changeset)
            })
            .collect()
    }

    fn record_entry(
        &self,
        connection: &Connection,
        changeset_id: ChangesetId,
        task_id: TaskId,
        created_at: i64,
        entry: &ChangesetEntry,
    ) -> Result<(), StorageError> {
        let error =
            |error: rusqlite::Error| sqlite_error("record an AI edit in", &self.path, &error);
        let payload = self.json_of("serialise a changeset entry for", entry)?;
        let id = changeset_id.to_string();
        connection
            .execute(
                "INSERT INTO changesets (id, task_id, created_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT (id) DO NOTHING",
                params![id, task_id.to_string(), created_at],
            )
            .map_err(error)?;
        connection
            .execute(
                "INSERT INTO changeset_entries
                 (changeset_id, sequence, transaction_id, format_version, payload)
                 VALUES (?1, (SELECT COALESCE(MAX(sequence) + 1, 0) FROM changeset_entries
                 WHERE changeset_id = ?1), ?2, ?3, ?4)",
                params![
                    id,
                    entry.transaction.id.to_string(),
                    JOURNAL_FORMAT_VERSION,
                    payload
                ],
            )
            .map(drop)
            .map_err(error)
    }

    fn changeset_entries(&self, changeset_id: &str) -> Result<Vec<ChangesetEntry>, StorageError> {
        let error =
            |error: rusqlite::Error| sqlite_error("read the AI changesets of", &self.path, &error);
        let mut statement = self
            .connection
            .prepare(
                "SELECT sequence, format_version, payload FROM changeset_entries
                 WHERE changeset_id = ?1 ORDER BY sequence",
            )
            .map_err(error)?;
        let rows = statement
            .query_map([changeset_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, u32>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(error)?;
        let mut entries = Vec::new();
        for row in rows {
            let (sequence, format_version, payload) = row.map_err(error)?;
            if format_version > JOURNAL_FORMAT_VERSION {
                return Err(StorageError::SchemaTooNew {
                    path: self.path.clone(),
                    found: format_version,
                    supported: JOURNAL_FORMAT_VERSION,
                });
            }
            entries.push(self.parse_json(&payload, || {
                format!("entry {sequence} of changeset {changeset_id}")
            })?);
        }
        Ok(entries)
    }

    fn parse_id<T, E: std::fmt::Display>(
        &self,
        text: &str,
        parse: impl FnOnce(&str) -> Result<T, E>,
    ) -> Result<T, StorageError> {
        parse(text).map_err(|error| StorageError::Corrupted {
            path: self.path.clone(),
            reason: format!("changeset row holds an invalid id {text}: {error}"),
        })
    }
}
