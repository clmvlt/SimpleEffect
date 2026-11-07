//! [`JobQueue`]: background jobs with priorities, a concurrency limit per kind, cancellation and
//! one automatic retry (`docs/architecture.md` §7, ADR-0018).
//!
//! The queue knows nothing about media: a job is a [`JobWork`] (what to run) plus its kind,
//! priority, project and asset. A job starts when its kind has a free slot and fewer than
//! [`JobQueueConfig::max_running`] jobs run in all, so priorities also order different kinds
//! (thumbnails before proxies before analyses). Each running job is a tokio task; cancelling
//! aborts the task,
//! which drops the work's future (adapters stop their processes on drop). Every change is
//! published as an event (`jobQueued`, `jobProgress`, `jobCompleted`, `jobFailed`,
//! `jobCancelled`). The jobs of a project stop when it closes ([`JobQueue::cancel_project`]).

mod reporter;
mod state;

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    time::Duration,
};

use async_trait::async_trait;
use se_domain::{AssetId, JobId, ProjectId};
use se_protocol::{CoreEvent, JobKind, JobProgress, JobSnapshot, JobState};
use tokio::{sync::oneshot, task::JoinError};

use self::{
    reporter::Reporter,
    state::{Entry, Phase, QueueState, cancel_entry},
};
use crate::{
    AppError,
    new_ids::NewIds,
    ports::{EventSink, IdGenerator, ProgressSink},
};

/// How long [`JobQueue::cancel_project`] waits for cancelled tasks to stop.
const CANCEL_GRACE: Duration = Duration::from_secs(5);

/// Order in which queued jobs start (higher first; equal priorities start oldest first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum JobPriority {
    /// Work done when nothing else waits (mezzanines of media no clip uses yet).
    Idle,
    /// Work nobody waits for (analyses).
    Low,
    /// The default.
    Normal,
    /// Work the user is looking at (thumbnails of just-imported media).
    High,
}

/// Which attempt of a job is running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JobAttempt {
    /// From 1.
    pub number: u8,
    /// Whether a failure of this attempt fails the job (no retry left).
    pub last: bool,
}

/// What a job does. Dropping the future returned by `run` must stop the work.
#[async_trait]
pub trait JobWork: Send + Sync {
    /// Runs the work, reporting progress; an error message makes the job retry once, then fail.
    /// On the `last` attempt, the work records its own failure (if it must) before returning it.
    async fn run(&self, progress: &dyn ProgressSink, attempt: JobAttempt) -> Result<(), String>;
}

/// A job to queue.
pub struct JobRequest {
    /// The project it works for (its jobs stop when it closes).
    pub project_id: ProjectId,
    /// The asset it works on, if any.
    pub asset_id: Option<AssetId>,
    /// What it produces (decides its concurrency limit).
    pub kind: JobKind,
    /// When it starts relative to others.
    pub priority: JobPriority,
    /// The work itself.
    pub work: Arc<dyn JobWork>,
}

/// Limits of the queue.
#[derive(Debug, Clone)]
pub struct JobQueueConfig {
    /// Jobs of a kind running at once (1 for kinds not listed).
    pub concurrency: BTreeMap<JobKind, usize>,
    /// Jobs running at once, all kinds together.
    pub max_running: usize,
    /// Attempts per job, the first included (2: one automatic retry).
    pub max_attempts: u8,
    /// Finished jobs remembered for `listJobs`.
    pub finished_kept: usize,
}

impl Default for JobQueueConfig {
    fn default() -> Self {
        Self {
            concurrency: BTreeMap::from([
                (JobKind::Thumbnails, 2),
                (JobKind::WaveformPeaks, 2),
                (JobKind::Mezzanine, 1),
                (JobKind::Proxy, 1),
                (JobKind::AudioAnalysis, 2),
                (JobKind::SceneDetection, 1),
            ]),
            max_running: 4,
            max_attempts: 2,
            finished_kept: 500,
        }
    }
}

/// The background job queue. Cheap to clone (shared state).
#[derive(Clone)]
pub struct JobQueue {
    shared: Arc<Shared>,
}

struct Shared {
    state: Mutex<QueueState>,
    events: Arc<dyn EventSink>,
    ids: Arc<dyn IdGenerator>,
    config: JobQueueConfig,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, QueueState> {
        // A panic while holding the lock cannot leave the bookkeeping half-updated in a way that
        // matters more than losing every job, so poisoning is ignored.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn limit(&self, kind: JobKind) -> usize {
        self.config
            .concurrency
            .get(&kind)
            .copied()
            .unwrap_or(1)
            .max(1)
    }
}

impl JobQueue {
    /// An empty queue publishing to `events`.
    #[must_use]
    pub fn new(
        events: Arc<dyn EventSink>,
        ids: Arc<dyn IdGenerator>,
        config: JobQueueConfig,
    ) -> Self {
        Self {
            shared: Arc::new(Shared {
                state: Mutex::new(QueueState::default()),
                events,
                ids,
                config,
            }),
        }
    }

    /// Queues a job (it starts at once when its kind has a free slot) and returns its id.
    ///
    /// # Errors
    /// [`AppError::Internal`] if the id generator broke its contract.
    pub fn enqueue(&self, request: JobRequest) -> Result<JobId, AppError> {
        let id = NewIds(self.shared.ids.as_ref()).job()?;
        let snapshot = JobSnapshot {
            id,
            project_id: request.project_id,
            asset_id: request.asset_id,
            kind: request.kind,
            state: JobState::Queued,
            attempt: 1,
        };
        let mut state = self.shared.lock();
        state.entries.push(Entry {
            snapshot: snapshot.clone(),
            priority: request.priority,
            work: request.work,
            phase: Phase::Queued,
        });
        self.shared
            .events
            .publish(CoreEvent::JobQueued { job: snapshot });
        self.dispatch(&mut state);
        Ok(id)
    }

    /// Cancels a queued or running job.
    ///
    /// # Errors
    /// [`AppError::JobNotFound`] when no active job has this id.
    pub fn cancel(&self, job_id: JobId) -> Result<(), AppError> {
        let mut state = self.shared.lock();
        let entry = state
            .active_mut(job_id)
            .ok_or(AppError::JobNotFound { job_id })?;
        // The task stops on its own; nobody waits for it here.
        drop(cancel_entry(entry));
        let snapshot = entry.snapshot.clone();
        self.shared
            .events
            .publish(CoreEvent::JobCancelled { job: snapshot });
        self.dispatch(&mut state);
        Ok(())
    }

    /// Cancels every job of a project, waits (a few seconds at most) for running ones to stop,
    /// and forgets them all.
    pub async fn cancel_project(&self, project_id: ProjectId) {
        let mut stopping = Vec::new();
        {
            let mut state = self.shared.lock();
            for entry in &mut state.entries {
                if entry.snapshot.project_id == project_id && entry.is_active() {
                    stopping.extend(cancel_entry(entry));
                    let snapshot = entry.snapshot.clone();
                    self.shared
                        .events
                        .publish(CoreEvent::JobCancelled { job: snapshot });
                }
            }
            state
                .entries
                .retain(|entry| entry.snapshot.project_id != project_id);
            self.dispatch(&mut state);
        }
        let all_stopped = async {
            for terminated in stopping {
                // An error only means the task is gone, which is what we wait for.
                let _ = terminated.await;
            }
        };
        if tokio::time::timeout(CANCEL_GRACE, all_stopped)
            .await
            .is_err()
        {
            tracing::warn!(%project_id, "cancelled jobs did not stop within {CANCEL_GRACE:?}");
        }
    }

    /// The jobs of a project, oldest first: queued, running and those finished in this session.
    #[must_use]
    pub fn list(&self, project_id: ProjectId) -> Vec<JobSnapshot> {
        self.shared
            .lock()
            .entries
            .iter()
            .filter(|entry| entry.snapshot.project_id == project_id)
            .map(|entry| entry.snapshot.clone())
            .collect()
    }

    /// Whether a job of `kind` is queued or running for this asset.
    #[must_use]
    pub fn has_active(&self, project_id: ProjectId, asset_id: AssetId, kind: JobKind) -> bool {
        self.shared.lock().has_active(project_id, asset_id, kind)
    }

    /// Starts queued jobs while their kinds have free slots.
    fn dispatch(&self, state: &mut QueueState) {
        let max_running = self.shared.config.max_running.max(1);
        while let Some(index) = state.next_runnable(|kind| self.shared.limit(kind), max_running) {
            let entry = &mut state.entries[index];
            entry.snapshot.state = JobState::Running { progress: None };
            let snapshot = entry.snapshot.clone();
            let work = Arc::clone(&entry.work);
            let reporter = Reporter::new(Arc::clone(&self.shared), &snapshot);
            let attempt = JobAttempt {
                number: snapshot.attempt,
                last: snapshot.attempt >= self.shared.config.max_attempts,
            };
            let worker = tokio::spawn(async move { work.run(&reporter, attempt).await });
            let (terminated_sender, terminated) = oneshot::channel::<()>();
            entry.phase = Phase::Running {
                abort: worker.abort_handle(),
                terminated: Some(terminated),
            };
            let queue = self.clone();
            let job_id = snapshot.id;
            tokio::spawn(async move {
                let outcome = worker.await;
                drop(terminated_sender);
                queue.finish(job_id, outcome);
            });
            self.shared
                .events
                .publish(CoreEvent::JobProgress(JobProgress {
                    job_id,
                    project_id: snapshot.project_id,
                    asset_id: snapshot.asset_id,
                    progress: None,
                }));
        }
    }

    /// Records how a running job ended: completed, retried or failed. A cancelled job is no
    /// longer running and is left alone.
    fn finish(&self, job_id: JobId, outcome: Result<Result<(), String>, JoinError>) {
        let mut state = self.shared.lock();
        let max_attempts = self.shared.config.max_attempts;
        let Some(entry) = state.running_mut(job_id) else {
            return;
        };
        let failure = match outcome {
            Ok(Ok(())) => None,
            Ok(Err(message)) => Some(message),
            Err(error) => Some(format!("the job stopped unexpectedly: {error}")),
        };
        let event = match failure {
            None => {
                entry.snapshot.state = JobState::Completed;
                entry.phase = Phase::Finished;
                CoreEvent::JobCompleted {
                    job: entry.snapshot.clone(),
                }
            }
            Some(message) if entry.snapshot.attempt < max_attempts => {
                tracing::warn!(%job_id, %message, "job failed; retrying once");
                entry.snapshot.attempt += 1;
                entry.snapshot.state = JobState::Queued;
                entry.phase = Phase::Queued;
                CoreEvent::JobQueued {
                    job: entry.snapshot.clone(),
                }
            }
            Some(message) => {
                tracing::warn!(%job_id, %message, "job failed");
                entry.snapshot.state = JobState::Failed { message };
                entry.phase = Phase::Finished;
                CoreEvent::JobFailed {
                    job: entry.snapshot.clone(),
                }
            }
        };
        self.shared.events.publish(event);
        state.prune_finished(self.shared.config.finished_kept);
        self.dispatch(&mut state);
    }
}
