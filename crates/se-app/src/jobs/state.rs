//! The bookkeeping of the job queue: entries in creation order and the choice of the next job to
//! start. Plain data, no tokio, so the scheduling rules are unit-tested directly.

use std::{collections::BTreeMap, sync::Arc};

use se_domain::{AssetId, JobId, ProjectId};
use se_protocol::{JobKind, JobSnapshot, JobState};
use tokio::{sync::oneshot, task::AbortHandle};

use super::{JobPriority, JobWork};

/// Where an entry is in its life.
pub(super) enum Phase {
    /// Waiting for a slot.
    Queued,
    /// Running: `abort` stops its task; `terminated` resolves once the task is gone.
    Running {
        abort: AbortHandle,
        terminated: Option<oneshot::Receiver<()>>,
    },
    /// Completed, failed or cancelled; kept for `listJobs`.
    Finished,
}

/// One job.
pub(super) struct Entry {
    pub snapshot: JobSnapshot,
    pub priority: JobPriority,
    pub work: Arc<dyn JobWork>,
    pub phase: Phase,
}

impl Entry {
    /// Queued or running.
    pub fn is_active(&self) -> bool {
        !matches!(self.phase, Phase::Finished)
    }
}

/// Every job of the session, oldest first.
#[derive(Default)]
pub(super) struct QueueState {
    pub entries: Vec<Entry>,
}

impl QueueState {
    /// The queued entry to start next: the highest priority, then the oldest, among kinds that
    /// have fewer running jobs than `limit(kind)`.
    pub fn next_runnable(&self, limit: impl Fn(JobKind) -> usize) -> Option<usize> {
        let mut running: BTreeMap<JobKind, usize> = BTreeMap::new();
        for entry in &self.entries {
            if matches!(entry.phase, Phase::Running { .. }) {
                *running.entry(entry.snapshot.kind).or_default() += 1;
            }
        }
        let mut best: Option<usize> = None;
        for (index, entry) in self.entries.iter().enumerate() {
            let kind = entry.snapshot.kind;
            let has_slot = running.get(&kind).copied().unwrap_or(0) < limit(kind);
            if !matches!(entry.phase, Phase::Queued) || !has_slot {
                continue;
            }
            // Entries are in creation order, so a strictly higher priority is needed to win.
            if best.is_none_or(|best| entry.priority > self.entries[best].priority) {
                best = Some(index);
            }
        }
        best
    }

    /// The active entry with this id.
    pub fn active_mut(&mut self, job_id: JobId) -> Option<&mut Entry> {
        self.entries
            .iter_mut()
            .find(|entry| entry.snapshot.id == job_id && entry.is_active())
    }

    /// The running entry with this id.
    pub fn running_mut(&mut self, job_id: JobId) -> Option<&mut Entry> {
        self.entries.iter_mut().find(|entry| {
            entry.snapshot.id == job_id && matches!(entry.phase, Phase::Running { .. })
        })
    }

    /// Whether a job of `kind` for this asset is queued or running.
    pub fn has_active(&self, project_id: ProjectId, asset_id: AssetId, kind: JobKind) -> bool {
        self.entries.iter().any(|entry| {
            entry.is_active()
                && entry.snapshot.project_id == project_id
                && entry.snapshot.asset_id == Some(asset_id)
                && entry.snapshot.kind == kind
        })
    }

    /// Forgets the oldest finished entries beyond `kept`.
    pub fn prune_finished(&mut self, kept: usize) {
        let finished = self
            .entries
            .iter()
            .filter(|entry| !entry.is_active())
            .count();
        let mut excess = finished.saturating_sub(kept);
        self.entries.retain(|entry| {
            if excess > 0 && !entry.is_active() {
                excess -= 1;
                return false;
            }
            true
        });
    }
}

/// Marks `entry` cancelled and stops its task; returns the receiver resolving once the task is
/// gone (for running jobs).
pub(super) fn cancel_entry(entry: &mut Entry) -> Option<oneshot::Receiver<()>> {
    let phase = std::mem::replace(&mut entry.phase, Phase::Finished);
    entry.snapshot.state = JobState::Cancelled;
    match phase {
        Phase::Running { abort, terminated } => {
            abort.abort();
            terminated
        }
        Phase::Queued | Phase::Finished => None,
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;
    use crate::ports::ProgressSink;

    struct Idle;

    #[async_trait]
    impl JobWork for Idle {
        async fn run(&self, _progress: &dyn ProgressSink) -> Result<(), String> {
            Ok(())
        }
    }

    fn entry(index: u64, kind: JobKind, priority: JobPriority, phase: Phase) -> Entry {
        Entry {
            snapshot: JobSnapshot {
                id: JobId::from_test_index(index),
                project_id: ProjectId::from_test_index(1),
                asset_id: Some(AssetId::from_test_index(index)),
                kind,
                state: JobState::Queued,
                attempt: 1,
            },
            priority,
            work: Arc::new(Idle),
            phase,
        }
    }

    #[test]
    fn the_highest_priority_then_the_oldest_starts_first() {
        let state = QueueState {
            entries: vec![
                entry(
                    1,
                    JobKind::WaveformPeaks,
                    JobPriority::Normal,
                    Phase::Queued,
                ),
                entry(2, JobKind::Thumbnails, JobPriority::High, Phase::Queued),
                entry(3, JobKind::Thumbnails, JobPriority::High, Phase::Queued),
            ],
        };
        assert_eq!(state.next_runnable(|_| 1), Some(1));
    }

    #[test]
    fn finished_entries_are_pruned_oldest_first() {
        let mut state = QueueState {
            entries: vec![
                entry(1, JobKind::Thumbnails, JobPriority::High, Phase::Finished),
                entry(2, JobKind::Thumbnails, JobPriority::High, Phase::Queued),
                entry(3, JobKind::Thumbnails, JobPriority::High, Phase::Finished),
            ],
        };
        state.prune_finished(1);
        let ids: Vec<_> = state
            .entries
            .iter()
            .map(|entry| entry.snapshot.id)
            .collect();
        assert_eq!(ids, [JobId::from_test_index(2), JobId::from_test_index(3)]);
    }

    #[test]
    fn queued_jobs_cancel_without_a_task() {
        let mut queued = entry(1, JobKind::Thumbnails, JobPriority::Low, Phase::Queued);
        assert!(cancel_entry(&mut queued).is_none());
        assert_eq!(queued.snapshot.state, JobState::Cancelled);
        assert!(!queued.is_active());
    }
}
