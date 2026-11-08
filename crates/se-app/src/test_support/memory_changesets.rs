//! The AI changesets of a fake project folder (the `changesets` and `changeset_entries` tables of
//! `project.db` in se-store).

use se_domain::Changeset;

use crate::ports::{ChangesetWrite, StorageError};

/// A changeset as a fake folder keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredChangeset {
    /// The changeset and its entries, oldest first.
    pub changeset: Changeset,
    /// When it was created (milliseconds since the Unix epoch).
    pub created_at: i64,
    /// When it was reverted; `None` while open.
    pub reverted_at: Option<i64>,
}

/// Applies `change` to a folder's changesets, as se-store's single SQLite transaction does.
///
/// # Errors
/// [`StorageError::Corrupted`] when reverting a changeset that is not there (a bug).
pub(super) fn apply_write(
    changesets: &mut Vec<StoredChangeset>,
    change: &ChangesetWrite,
) -> Result<(), StorageError> {
    match change {
        ChangesetWrite::Record {
            changeset_id,
            task_id,
            created_at,
            entry,
        } => {
            if let Some(stored) = changesets
                .iter_mut()
                .find(|stored| stored.changeset.id == *changeset_id)
            {
                stored.changeset.entries.push(entry.as_ref().clone());
            } else {
                let mut changeset = Changeset::new(*changeset_id, *task_id);
                changeset.entries.push(entry.as_ref().clone());
                changesets.push(StoredChangeset {
                    changeset,
                    created_at: *created_at,
                    reverted_at: None,
                });
            }
            Ok(())
        }
        ChangesetWrite::Revert { changeset_id, at } => {
            let stored = changesets
                .iter_mut()
                .find(|stored| stored.changeset.id == *changeset_id)
                .ok_or_else(|| StorageError::Corrupted {
                    path: std::path::PathBuf::from("project.db"),
                    reason: format!("no changeset {changeset_id} to revert"),
                })?;
            stored.reverted_at = Some(*at);
            Ok(())
        }
    }
}

/// The open changesets, in creation order.
pub(super) fn open_changesets(changesets: &[StoredChangeset]) -> Vec<Changeset> {
    changesets
        .iter()
        .filter(|stored| stored.reverted_at.is_none())
        .map(|stored| stored.changeset.clone())
        .collect()
}
