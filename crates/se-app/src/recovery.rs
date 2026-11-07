//! Crash recovery: replaying journal entries on top of the last snapshot (ADR-0006).

use se_domain::{Project, commit};

use crate::ports::JournalEntry;

/// Replays `journal` (entries newer than `snapshot`, in revision order) and returns the project
/// as it was after the last entry.
///
/// Commits are deterministic, so the replay rebuilds exactly the state that was journaled.
///
/// # Errors
/// A description of the first entry that does not follow the previous revision or no longer
/// applies.
pub fn replay(snapshot: Project, journal: &[JournalEntry]) -> Result<Project, String> {
    let mut project = snapshot;
    for entry in journal {
        let expected = project.revision.saturating_add(1);
        if entry.revision != expected {
            return Err(format!(
                "entry {} does not follow revision {} (expected {expected})",
                entry.revision, project.revision
            ));
        }
        let committed = commit(&project, &entry.transaction).map_err(|error| {
            format!(
                "entry {} (« {} ») no longer applies: {error}",
                entry.revision, entry.transaction.label
            )
        })?;
        project = committed.project;
    }
    Ok(project)
}

#[cfg(test)]
mod tests {
    use se_domain::{
        Actor, ItemId, MoveItem, Transaction, TransactionId,
        test_support::{example_project, millis},
    };

    use super::*;

    fn move_text(revision: u64, start_ms: i64) -> JournalEntry {
        JournalEntry {
            revision,
            committed_at: 0,
            transaction: Transaction {
                id: TransactionId::from_test_index(revision),
                actor: Actor::User,
                label: "Move".to_owned(),
                ops: vec![
                    MoveItem {
                        item: ItemId::from_test_index(3),
                        track: example_project().sequences[0].tracks[1].id,
                        start: millis(start_ms),
                        expected_revision: None,
                    }
                    .into(),
                ],
            },
        }
    }

    #[test]
    fn replays_entries_in_order() {
        let snapshot = example_project();
        let next = snapshot.revision + 1;
        let journal = [move_text(next, 8000), move_text(next + 1, 9000)];
        let project = replay(snapshot, &journal).unwrap();
        assert_eq!(project.revision, next + 1);
        let text = project.sequences[0].tracks[1]
            .items
            .iter()
            .find(|item| item.id == ItemId::from_test_index(3))
            .unwrap();
        assert_eq!(text.timeline_range.start(), millis(9000));
    }

    #[test]
    fn refuses_a_gap_in_revisions() {
        let snapshot = example_project();
        let error = replay(snapshot.clone(), &[move_text(snapshot.revision + 2, 8000)]);
        assert!(error.unwrap_err().contains("does not follow"));
    }

    #[test]
    fn refuses_an_entry_that_no_longer_applies() {
        let snapshot = example_project();
        let mut entry = move_text(snapshot.revision + 1, 8000);
        entry.transaction.ops = vec![
            se_domain::RemoveItem {
                item: ItemId::from_test_index(999),
                expected_revision: None,
            }
            .into(),
        ];
        let error = replay(snapshot, &[entry]).unwrap_err();
        assert!(error.contains("no longer applies"));
    }
}
