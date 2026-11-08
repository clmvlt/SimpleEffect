//! One run of a task, from the queue to its final status: working folder, agent start, events
//! appended and published as they arrive, cancellation, and the status the run ends with.

use std::{sync::Arc, time::Duration};

use se_protocol::{AgentEvent, AgentFailureKind, CoreEvent, ProviderId, TaskStatus};
use tokio::{
    sync::{oneshot, watch},
    time::Instant,
};

use super::{
    Shared, instructions,
    queue::RunKey,
    task_state::{self, SessionPlan},
};
use crate::ports::{AgentProvider, AgentRun, AgentRunSpec, AgentSession, TaskStore};

/// How long a run asked to stop may still report before it is given up on (its process is
/// killed when the run is dropped).
const CANCEL_GRACE: Duration = Duration::from_secs(5);

/// How long the CLI may take to exit after its terminal event.
const EXIT_GRACE: Duration = Duration::from_secs(10);

/// What a run needs once its task is marked running.
struct RunStart {
    provider: Option<Arc<dyn AgentProvider>>,
    recorder: Recorder,
    spec: AgentRunSpec,
}

/// Appends a run's events to its task's log and publishes them.
struct Recorder {
    shared: Arc<Shared>,
    key: RunKey,
    store: Arc<dyn TaskStore>,
    run: u32,
    provider: ProviderId,
}

/// How the agent's event stream ended.
struct StreamEnd {
    terminal: Option<AgentEvent>,
    stop_requested: bool,
}

/// Runs the latest run of `key`'s task, then frees its slot. `cancelled` turns `true` when the
/// run must stop; `_gone` is dropped once everything is saved (closing projects wait for it).
pub(super) async fn execute(
    shared: Arc<Shared>,
    key: RunKey,
    mut cancelled: watch::Receiver<bool>,
    _gone: oneshot::Sender<()>,
) {
    let status = match shared.begin_run(key).await {
        Some(start) => Some(drive(&shared, start, &mut cancelled).await),
        None => None,
    };
    shared.end_run(key, status).await;
}

async fn drive(
    shared: &Arc<Shared>,
    start: RunStart,
    cancelled: &mut watch::Receiver<bool>,
) -> TaskStatus {
    let RunStart {
        provider,
        recorder,
        mut spec,
    } = start;
    let task_id = recorder.key.task_id;
    let Some(provider) = provider else {
        let event = AgentEvent::Failed {
            failure: AgentFailureKind::NotInstalled,
            message: format!(
                "this version of SimpleEffect has no AI provider {}",
                recorder.provider
            ),
        };
        recorder.record(&event).await;
        return final_status(
            &recorder,
            StreamEnd {
                terminal: Some(event),
                stop_requested: false,
            },
        )
        .await;
    };
    if *cancelled.borrow_and_update() {
        return final_status(
            &recorder,
            StreamEnd {
                terminal: None,
                stop_requested: true,
            },
        )
        .await;
    }
    let workspaces = &shared.deps.workspaces;
    spec.working_dir = match workspaces.prepare(task_id).await {
        Ok(dir) => dir,
        Err(error) => {
            let event = error.to_event();
            recorder.record(&event).await;
            let end = StreamEnd {
                terminal: Some(event),
                stop_requested: false,
            };
            return final_status(&recorder, end).await;
        }
    };
    let end = match provider.start(spec).await {
        Ok(run) => follow(&recorder, run, cancelled).await,
        Err(error) => {
            let event = error.to_event();
            recorder.record(&event).await;
            StreamEnd {
                terminal: Some(event),
                stop_requested: *cancelled.borrow(),
            }
        }
    };
    workspaces.release(task_id).await;
    final_status(&recorder, end).await
}

/// Records the run's events until its stream closes, stopping it when asked.
async fn follow(
    recorder: &Recorder,
    run: AgentRun,
    cancelled: &mut watch::Receiver<bool>,
) -> StreamEnd {
    let AgentRun {
        mut events,
        mut cancel,
    } = run;
    let mut end = StreamEnd {
        terminal: None,
        stop_requested: *cancelled.borrow_and_update(),
    };
    let mut deadline = None;
    if end.stop_requested {
        cancel.cancel();
        deadline = Some(Instant::now() + CANCEL_GRACE);
    }
    loop {
        let give_up = async {
            match deadline {
                Some(deadline) => tokio::time::sleep_until(deadline).await,
                None => std::future::pending().await,
            }
        };
        tokio::select! {
            changed = cancelled.changed(), if !end.stop_requested => {
                // An error means the queue forgot the run, which only happens once it ended:
                // stop anyway.
                if changed.is_err() || *cancelled.borrow() {
                    end.stop_requested = true;
                    cancel.cancel();
                    deadline = Some(earliest(deadline, CANCEL_GRACE));
                }
            }
            event = events.recv() => {
                let Some(event) = event else { break };
                if end.terminal.is_some() {
                    tracing::debug!(task = %recorder.key.task_id, ?event, "event after the end of a run ignored");
                    continue;
                }
                recorder.record(&event).await;
                if event.is_terminal() {
                    end.terminal = Some(event);
                    deadline = Some(earliest(deadline, EXIT_GRACE));
                }
            }
            () = give_up => {
                tracing::warn!(task = %recorder.key.task_id, "the agent did not stop in time; dropping it");
                break;
            }
        }
    }
    end
}

/// The earlier of `deadline` and `delay` from now.
fn earliest(deadline: Option<Instant>, delay: Duration) -> Instant {
    let candidate = Instant::now() + delay;
    deadline.map_or(candidate, |deadline| deadline.min(candidate))
}

/// The status a run ends with; records the terminal event the stream lacked, if any.
async fn final_status(recorder: &Recorder, end: StreamEnd) -> TaskStatus {
    match end.terminal {
        Some(AgentEvent::Completed { .. }) => TaskStatus::Completed,
        Some(AgentEvent::Failed { failure, .. })
            if end.stop_requested || failure == AgentFailureKind::Cancelled =>
        {
            TaskStatus::Cancelled
        }
        Some(AgentEvent::Failed { failure, message }) => TaskStatus::Failed { failure, message },
        _ if end.stop_requested => {
            let event = AgentEvent::Failed {
                failure: AgentFailureKind::Cancelled,
                message: "the task was cancelled".to_owned(),
            };
            recorder.record(&event).await;
            TaskStatus::Cancelled
        }
        _ => {
            let message = "the agent stopped without reporting a result: retry the task".to_owned();
            recorder
                .record(&AgentEvent::Failed {
                    failure: AgentFailureKind::Crashed,
                    message: message.clone(),
                })
                .await;
            TaskStatus::Failed {
                failure: AgentFailureKind::Crashed,
                message,
            }
        }
    }
}

impl Recorder {
    /// Appends `event` to the log, publishes it and applies what it tells about the task or the
    /// provider. A storage failure is logged: the run goes on.
    async fn record(&self, event: &AgentEvent) {
        let now = self.shared.now();
        let task_id = self.key.task_id;
        match self
            .store
            .append_task_event(task_id, self.run, now, event)
            .await
        {
            Ok(record) => self
                .shared
                .deps
                .events
                .publish(CoreEvent::TaskEventAppended {
                    project_id: self.key.project_id,
                    task_id,
                    event: record,
                }),
            Err(error) => tracing::warn!(task = %task_id, %error, "saving a task event failed"),
        }
        match event {
            AgentEvent::SessionStarted { session_id, .. } => {
                self.shared.set_session(self.key, session_id).await;
            }
            AgentEvent::Failed { failure, message } => {
                self.shared.note_failure(&self.provider, *failure, message);
            }
            _ => {}
        }
    }
}

impl Shared {
    /// Marks the task's latest run running and prepares it; `None` when the task is gone or no
    /// longer waits (cancelled meanwhile).
    async fn begin_run(self: &Arc<Self>, key: RunKey) -> Option<RunStart> {
        let now = self.now();
        let session_id = self.deps.ids.new_uuid_v7().to_string();
        let mut state = self.state.lock().await;
        let project = state.projects.get_mut(&key.project_id)?;
        let store = Arc::clone(project.handle.tasks());
        let task = project
            .tasks
            .iter_mut()
            .find(|task| task.id == key.task_id)?;
        if task.status != TaskStatus::Queued {
            return None;
        }
        task_state::start_run(task, now);
        let task = task.clone();
        if let Err(error) = store.save_task(&task).await {
            tracing::warn!(task = %task.id, %error, "saving a started task failed");
        }
        self.deps
            .events
            .publish(CoreEvent::TaskUpdated { task: task.clone() });
        let session = match task_state::session_plan(&task) {
            SessionPlan::Fresh => AgentSession::Start(session_id),
            SessionPlan::Resume(session) => AgentSession::Resume(session),
        };
        let spec = AgentRunSpec {
            prompt: task
                .runs
                .last()
                .map_or_else(|| task.prompt.clone(), |run| run.prompt.clone()),
            system_instructions: instructions::compose(&task),
            mcp: None,
            model: task.model.clone(),
            effort: task.effort,
            session,
            working_dir: std::path::PathBuf::new(),
        };
        Some(RunStart {
            provider: self.provider(&task.provider),
            recorder: Recorder {
                shared: Arc::clone(self),
                key,
                store,
                run: task_state::current_run(&task),
                provider: task.provider.clone(),
            },
            spec,
        })
    }

    /// Records the session the agent announced.
    async fn set_session(&self, key: RunKey, session_id: &str) {
        let now = self.now();
        let mut state = self.state.lock().await;
        let Some(project) = state.projects.get_mut(&key.project_id) else {
            return;
        };
        let store = Arc::clone(project.handle.tasks());
        let Some(task) = project.tasks.iter_mut().find(|task| task.id == key.task_id) else {
            return;
        };
        if task.agent_session_id.as_deref() == Some(session_id) {
            return;
        }
        task.agent_session_id = Some(session_id.to_owned());
        task.updated_at = now;
        let task = task.clone();
        if let Err(error) = store.save_task(&task).await {
            tracing::warn!(task = %task.id, %error, "saving a task's session failed");
        }
        self.deps.events.publish(CoreEvent::TaskUpdated { task });
    }

    /// Frees the run's slot, records its final status (when it ran) and starts waiting runs.
    async fn end_run(self: &Arc<Self>, key: RunKey, status: Option<TaskStatus>) {
        let now = self.now();
        let mut state = self.state.lock().await;
        state.queue.finish(key.task_id);
        if let Some(status) = status
            && let Some(project) = state.projects.get_mut(&key.project_id)
        {
            let store = Arc::clone(project.handle.tasks());
            if let Some(task) = project.tasks.iter_mut().find(|task| task.id == key.task_id) {
                task_state::finish_run(task, status, now);
                let task = task.clone();
                if let Err(error) = store.save_task(&task).await {
                    tracing::warn!(task = %task.id, %error, "saving a finished task failed");
                }
                self.deps.events.publish(CoreEvent::TaskUpdated { task });
            }
        }
        self.dispatch(&mut state);
    }
}
