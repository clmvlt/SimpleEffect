//! [`Reporter`]: the progress sink handed to running jobs.

use std::sync::{Arc, Mutex, PoisonError};

use se_domain::{AssetId, JobId, ProjectId};
use se_protocol::{CoreEvent, JobProgress, JobSnapshot, JobState};

use super::Shared;
use crate::ports::ProgressSink;

/// Smallest change of progress worth an event (1 %): jobs may report far more often.
const MIN_STEP: f64 = 0.01;

/// Updates a running job's state and publishes `jobProgress`, at most once per percent.
pub(super) struct Reporter {
    shared: Arc<Shared>,
    job_id: JobId,
    project_id: ProjectId,
    asset_id: Option<AssetId>,
    last_published: Mutex<Option<f64>>,
}

impl Reporter {
    pub fn new(shared: Arc<Shared>, job: &JobSnapshot) -> Self {
        Self {
            shared,
            job_id: job.id,
            project_id: job.project_id,
            asset_id: job.asset_id,
            last_published: Mutex::new(None),
        }
    }
}

impl ProgressSink for Reporter {
    fn report(&self, fraction: f64) {
        if !fraction.is_finite() {
            return;
        }
        let fraction = fraction.clamp(0.0, 1.0);
        {
            let mut last = self
                .last_published
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            let too_small = last.is_some_and(|last| (fraction - last).abs() < MIN_STEP);
            if too_small && fraction < 1.0 {
                return;
            }
            *last = Some(fraction);
        }
        let mut state = self.shared.lock();
        // A cancelled job may still report before its task stops: stay silent then.
        let Some(entry) = state.running_mut(self.job_id) else {
            return;
        };
        entry.snapshot.state = JobState::Running {
            progress: Some(fraction),
        };
        self.shared
            .events
            .publish(CoreEvent::JobProgress(JobProgress {
                job_id: self.job_id,
                project_id: self.project_id,
                asset_id: self.asset_id,
                progress: Some(fraction),
            }));
    }
}
