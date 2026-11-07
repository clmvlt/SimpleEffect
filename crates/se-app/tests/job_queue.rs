//! The job queue with scripted work: priorities, concurrency per kind, cancellation, retry.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{
    collections::{BTreeMap, VecDeque},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use async_trait::async_trait;
use se_app::{
    JobPriority, JobQueue, JobQueueConfig, JobRequest, JobWork,
    ports::ProgressSink,
    test_support::{RecordingEventSink, SequentialIds},
};
use se_domain::{AssetId, JobId, ProjectId};
use se_protocol::{CoreEvent, JobKind, JobState};
use tokio::sync::Notify;

/// Work that logs its start, reports progress, may wait for a gate, then returns the next
/// scripted outcome (success when the script is empty). Dropping it mid-run is recorded.
struct ScriptedWork {
    name: &'static str,
    log: Arc<Mutex<Vec<&'static str>>>,
    gate: Option<Arc<Notify>>,
    outcomes: Mutex<VecDeque<Result<(), String>>>,
    dropped_while_running: Arc<AtomicBool>,
}

struct RunningGuard(Arc<AtomicBool>);

impl Drop for RunningGuard {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

#[async_trait]
impl JobWork for ScriptedWork {
    async fn run(&self, progress: &dyn ProgressSink) -> Result<(), String> {
        self.log.lock().unwrap().push(self.name);
        let guard = RunningGuard(Arc::clone(&self.dropped_while_running));
        progress.report(0.001);
        progress.report(0.002);
        progress.report(0.5);
        if let Some(gate) = &self.gate {
            gate.notified().await;
        }
        std::mem::forget(guard);
        self.outcomes.lock().unwrap().pop_front().unwrap_or(Ok(()))
    }
}

struct Harness {
    queue: JobQueue,
    events: Arc<RecordingEventSink>,
    log: Arc<Mutex<Vec<&'static str>>>,
}

fn harness(concurrency: usize) -> Harness {
    let events = Arc::new(RecordingEventSink::default());
    let config = JobQueueConfig {
        concurrency: BTreeMap::from([
            (JobKind::Thumbnails, concurrency),
            (JobKind::WaveformPeaks, concurrency),
        ]),
        ..JobQueueConfig::default()
    };
    Harness {
        queue: JobQueue::new(events.clone(), Arc::new(SequentialIds::default()), config),
        events,
        log: Arc::default(),
    }
}

const PROJECT: ProjectId = ProjectId::from_test_index(1);

impl Harness {
    fn work(&self, name: &'static str, gate: Option<&Arc<Notify>>) -> ScriptedWork {
        ScriptedWork {
            name,
            log: Arc::clone(&self.log),
            gate: gate.cloned(),
            outcomes: Mutex::default(),
            dropped_while_running: Arc::default(),
        }
    }

    fn enqueue(&self, kind: JobKind, priority: JobPriority, work: ScriptedWork) -> JobId {
        self.queue
            .enqueue(JobRequest {
                project_id: PROJECT,
                asset_id: Some(AssetId::from_test_index(1)),
                kind,
                priority,
                work: Arc::new(work),
            })
            .unwrap()
    }

    fn started(&self) -> Vec<&'static str> {
        self.log.lock().unwrap().clone()
    }

    fn state(&self, id: JobId) -> JobState {
        self.queue
            .list(PROJECT)
            .into_iter()
            .find(|job| job.id == id)
            .map(|job| job.state)
            .unwrap()
    }
}

/// Lets spawned tasks run until nothing is left to do.
async fn settle() {
    for _ in 0..50 {
        tokio::task::yield_now().await;
    }
}

#[tokio::test]
async fn higher_priorities_start_first_and_kinds_have_their_own_slots() {
    let harness = harness(1);
    let gate = Arc::new(Notify::new());
    harness.enqueue(
        JobKind::Thumbnails,
        JobPriority::Normal,
        harness.work("first", Some(&gate)),
    );
    harness.enqueue(
        JobKind::Thumbnails,
        JobPriority::Low,
        harness.work("low", None),
    );
    harness.enqueue(
        JobKind::Thumbnails,
        JobPriority::High,
        harness.work("high", None),
    );
    harness.enqueue(
        JobKind::WaveformPeaks,
        JobPriority::Low,
        harness.work("peaks", None),
    );
    settle().await;
    // One thumbnails slot is busy; the peaks slot is free.
    assert_eq!(harness.started(), ["first", "peaks"]);
    gate.notify_one();
    settle().await;
    assert_eq!(harness.started(), ["first", "peaks", "high", "low"]);
}

#[tokio::test]
async fn a_failed_job_is_retried_once_then_fails() {
    let harness = harness(2);
    let flaky = harness.work("flaky", None);
    flaky
        .outcomes
        .lock()
        .unwrap()
        .push_back(Err("boom".to_owned()));
    let flaky = harness.enqueue(JobKind::Thumbnails, JobPriority::Normal, flaky);
    let broken = harness.work("broken", None);
    broken
        .outcomes
        .lock()
        .unwrap()
        .extend([Err("first".to_owned()), Err("second".to_owned())]);
    let broken = harness.enqueue(JobKind::Thumbnails, JobPriority::Normal, broken);
    settle().await;
    assert_eq!(harness.state(flaky), JobState::Completed);
    assert_eq!(
        harness.state(broken),
        JobState::Failed {
            message: "second".to_owned()
        }
    );
    let events = harness.events.take();
    let retried = events
        .iter()
        .filter(|event| matches!(event, CoreEvent::JobQueued { job } if job.attempt == 2))
        .count();
    assert_eq!(retried, 2);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, CoreEvent::JobFailed { job } if job.id == broken))
    );
}

#[tokio::test]
async fn cancelling_stops_running_work_and_skips_queued_work() {
    let harness = harness(1);
    let gate = Arc::new(Notify::new());
    let running = harness.work("running", Some(&gate));
    let dropped = Arc::clone(&running.dropped_while_running);
    let running = harness.enqueue(JobKind::Thumbnails, JobPriority::Normal, running);
    let queued = harness.enqueue(
        JobKind::Thumbnails,
        JobPriority::Normal,
        harness.work("queued", None),
    );
    harness.enqueue(
        JobKind::Thumbnails,
        JobPriority::Normal,
        harness.work("next", None),
    );
    settle().await;
    harness.queue.cancel(queued).unwrap();
    harness.queue.cancel(running).unwrap();
    settle().await;
    assert!(
        dropped.load(Ordering::SeqCst),
        "the running work must be dropped"
    );
    assert_eq!(harness.started(), ["running", "next"]);
    assert_eq!(harness.state(running), JobState::Cancelled);
    assert_eq!(harness.state(queued), JobState::Cancelled);
    let error = harness.queue.cancel(queued).unwrap_err();
    assert_eq!(error, se_app::AppError::JobNotFound { job_id: queued });
}

#[tokio::test]
async fn closing_a_project_cancels_and_forgets_its_jobs() {
    let harness = harness(1);
    let gate = Arc::new(Notify::new());
    let running = harness.work("running", Some(&gate));
    let dropped = Arc::clone(&running.dropped_while_running);
    harness.enqueue(JobKind::Thumbnails, JobPriority::Normal, running);
    harness.enqueue(
        JobKind::Thumbnails,
        JobPriority::Normal,
        harness.work("queued", None),
    );
    settle().await;
    harness.queue.cancel_project(PROJECT).await;
    assert!(dropped.load(Ordering::SeqCst));
    assert_eq!(harness.queue.list(PROJECT), []);
    let cancelled = harness
        .events
        .take()
        .into_iter()
        .filter(|event| matches!(event, CoreEvent::JobCancelled { .. }))
        .count();
    assert_eq!(cancelled, 2);
    settle().await;
    assert_eq!(harness.started(), ["running"]);
}

#[tokio::test]
async fn progress_is_published_once_per_percent() {
    let harness = harness(1);
    let gate = Arc::new(Notify::new());
    let id = harness.enqueue(
        JobKind::Thumbnails,
        JobPriority::Normal,
        harness.work("job", Some(&gate)),
    );
    settle().await;
    assert_eq!(
        harness.state(id),
        JobState::Running {
            progress: Some(0.5)
        }
    );
    let progress: Vec<Option<f64>> = harness
        .events
        .take()
        .into_iter()
        .filter_map(|event| match event {
            CoreEvent::JobProgress(progress) => Some(progress.progress),
            _ => None,
        })
        .collect();
    // Started, then 0.001 (first report), then 0.5; 0.002 is too close to 0.001.
    assert_eq!(progress, [None, Some(0.001), Some(0.5)]);
    gate.notify_one();
    settle().await;
    assert_eq!(harness.state(id), JobState::Completed);
}
