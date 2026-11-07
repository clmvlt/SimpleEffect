//! Fakes of the agent ports: [`ReplayProvider`] replays scripted runs with simulated delays (the
//! provider of every task service test, and later of UX tests), [`FakeAgentWorkspaces`] hands
//! out folder paths without touching a disk.

use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use async_trait::async_trait;
use se_domain::TaskId;
use se_protocol::{
    AgentEvent, AgentFailureKind, AuthStatus, EffortLevel, ModelOption, ProviderCapabilities,
    ProviderId, ProviderInfo,
};
use tokio::sync::{Semaphore, mpsc};

use super::lock;
use crate::ports::{
    AgentError, AgentProvider, AgentRun, AgentRunSpec, AgentSession, AgentWorkspaces, CancelHandle,
    CancelSignal, ProviderInstallation,
};

/// Id of the [`ReplayProvider`].
pub const REPLAY_PROVIDER_ID: &str = "replay";

/// One step of a replayed run.
#[derive(Debug, Clone, PartialEq)]
pub enum ReplayStep {
    /// Waits (tests pause tokio's clock, so this costs nothing).
    Wait(Duration),
    /// Emits an event.
    Emit(AgentEvent),
    /// Emits [`AgentEvent::SessionStarted`] with the spec's session id (new or resumed), as a
    /// CLI that accepts chosen session ids does.
    StartSession,
    /// Waits until [`ReplayProvider::release`] lets one held run go on.
    Hold,
}

/// The steps of one run.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplayScript(pub Vec<ReplayStep>);

impl ReplayScript {
    /// A session, some text, usage, then completion with `summary`, 10 ms apart.
    #[must_use]
    pub fn success(summary: &str) -> Self {
        Self(vec![
            ReplayStep::StartSession,
            ReplayStep::Wait(Duration::from_millis(10)),
            ReplayStep::Emit(AgentEvent::AssistantText {
                text: summary.to_owned(),
            }),
            ReplayStep::Emit(AgentEvent::Usage {
                input_tokens: 120,
                output_tokens: 8,
                cost_usd: Some(0.001),
                duration_ms: Some(10),
            }),
            ReplayStep::Emit(AgentEvent::Completed {
                summary: summary.to_owned(),
            }),
        ])
    }

    /// A session, then nothing until released, then completion.
    #[must_use]
    pub fn held() -> Self {
        Self(vec![
            ReplayStep::StartSession,
            ReplayStep::Hold,
            ReplayStep::Emit(AgentEvent::Completed {
                summary: "done".to_owned(),
            }),
        ])
    }

    /// A session, then a failure.
    #[must_use]
    pub fn failing(failure: AgentFailureKind, message: &str) -> Self {
        Self(vec![
            ReplayStep::StartSession,
            ReplayStep::Emit(AgentEvent::Failed {
                failure,
                message: message.to_owned(),
            }),
        ])
    }

    /// `events`, `delay` apart (a recorded run replayed).
    #[must_use]
    pub fn from_events(events: Vec<AgentEvent>, delay: Duration) -> Self {
        let mut steps = Vec::new();
        for event in events {
            steps.push(ReplayStep::Wait(delay));
            steps.push(ReplayStep::Emit(event));
        }
        Self(steps)
    }
}

/// An [`AgentProvider`] replaying scripts: the next pushed script for each run, else the default
/// one ([`ReplayScript::success`]). Clones share state.
#[derive(Clone)]
pub struct ReplayProvider {
    inner: Arc<Inner>,
}

struct Inner {
    scripts: Mutex<VecDeque<ReplayScript>>,
    default_script: Mutex<ReplayScript>,
    installation: Mutex<Result<ProviderInstallation, AgentError>>,
    auth: Mutex<Result<AuthStatus, AgentError>>,
    start_errors: Mutex<VecDeque<AgentError>>,
    specs: Mutex<Vec<AgentRunSpec>>,
    executable: Mutex<Option<PathBuf>>,
    detections: AtomicUsize,
    alive: AtomicUsize,
    killed: AtomicUsize,
    released: Semaphore,
}

impl Default for ReplayProvider {
    /// Installed (`/bin/replay` 1.0.0), logged in, every run succeeding.
    fn default() -> Self {
        Self {
            inner: Arc::new(Inner {
                scripts: Mutex::new(VecDeque::new()),
                default_script: Mutex::new(ReplayScript::success("done")),
                installation: Mutex::new(Ok(ProviderInstallation {
                    path: PathBuf::from("/bin/replay"),
                    version: "1.0.0".to_owned(),
                })),
                auth: Mutex::new(Ok(AuthStatus::LoggedIn {
                    account_kind: Some("test".to_owned()),
                })),
                start_errors: Mutex::new(VecDeque::new()),
                specs: Mutex::new(Vec::new()),
                executable: Mutex::new(None),
                detections: AtomicUsize::new(0),
                alive: AtomicUsize::new(0),
                killed: AtomicUsize::new(0),
                released: Semaphore::new(0),
            }),
        }
    }
}

impl ReplayProvider {
    /// Uses `script` for the next run not yet scripted.
    pub fn push_script(&self, script: ReplayScript) {
        lock(&self.inner.scripts).push_back(script);
    }

    /// Uses `script` for runs with no pushed script.
    pub fn set_default_script(&self, script: ReplayScript) {
        *lock(&self.inner.default_script) = script;
    }

    /// What `detect` answers from now on.
    pub fn set_installation(&self, installation: Result<ProviderInstallation, AgentError>) {
        *lock(&self.inner.installation) = installation;
    }

    /// What `auth_status` answers from now on.
    pub fn set_auth(&self, auth: Result<AuthStatus, AgentError>) {
        *lock(&self.inner.auth) = auth;
    }

    /// Makes the next `start` fail with `error`.
    pub fn fail_next_start(&self, error: AgentError) {
        lock(&self.inner.start_errors).push_back(error);
    }

    /// Lets `count` held runs go on (runs reaching a hold later use up remaining releases).
    pub fn release(&self, count: usize) {
        self.inner.released.add_permits(count);
    }

    /// The specs of every started run, in order.
    #[must_use]
    pub fn specs(&self) -> Vec<AgentRunSpec> {
        lock(&self.inner.specs).clone()
    }

    /// Runs whose fake process is still alive.
    #[must_use]
    pub fn alive(&self) -> usize {
        self.inner.alive.load(Ordering::SeqCst)
    }

    /// Runs stopped by a cancellation.
    #[must_use]
    pub fn killed(&self) -> usize {
        self.inner.killed.load(Ordering::SeqCst)
    }

    /// How many times `detect` was called.
    #[must_use]
    pub fn detections(&self) -> usize {
        self.inner.detections.load(Ordering::SeqCst)
    }

    /// The executable set through `set_executable`.
    #[must_use]
    pub fn executable(&self) -> Option<PathBuf> {
        lock(&self.inner.executable).clone()
    }

    fn next_script(&self) -> ReplayScript {
        lock(&self.inner.scripts)
            .pop_front()
            .unwrap_or_else(|| lock(&self.inner.default_script).clone())
    }
}

#[async_trait]
impl AgentProvider for ReplayProvider {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            id: ProviderId(REPLAY_PROVIDER_ID.to_owned()),
            name: "Replay".to_owned(),
            command: "replay".to_owned(),
            login_command: "/login".to_owned(),
            help_url: "https://example.com/replay".to_owned(),
        }
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            resume_sessions: true,
            image_input: false,
            models: vec![ModelOption {
                id: "small".to_owned(),
                label: "Small".to_owned(),
            }],
            effort_levels: vec![EffortLevel::Low, EffortLevel::High],
        }
    }

    fn set_executable(&self, path: Option<PathBuf>) {
        *lock(&self.inner.executable) = path;
    }

    async fn detect(&self) -> Result<ProviderInstallation, AgentError> {
        self.inner.detections.fetch_add(1, Ordering::SeqCst);
        lock(&self.inner.installation).clone()
    }

    async fn auth_status(&self) -> Result<AuthStatus, AgentError> {
        lock(&self.inner.auth).clone()
    }

    async fn start(&self, spec: AgentRunSpec) -> Result<AgentRun, AgentError> {
        if let Some(error) = lock(&self.inner.start_errors).pop_front() {
            return Err(error);
        }
        lock(&self.inner.specs).push(spec.clone());
        let script = self.next_script();
        let (sender, events) = mpsc::channel(64);
        let (cancel, signal) = CancelHandle::new();
        self.inner.alive.fetch_add(1, Ordering::SeqCst);
        tokio::spawn(replay(
            Arc::clone(&self.inner),
            script,
            spec,
            sender,
            signal,
        ));
        Ok(AgentRun { events, cancel })
    }
}

/// Plays `script` until its end or a cancellation (then reports the run cancelled).
async fn replay(
    inner: Arc<Inner>,
    script: ReplayScript,
    spec: AgentRunSpec,
    sender: mpsc::Sender<AgentEvent>,
    mut signal: CancelSignal,
) {
    let session_id = match &spec.session {
        AgentSession::Start(id) | AgentSession::Resume(id) => id.clone(),
    };
    let mut killed = false;
    for step in script.0 {
        let play = async {
            match step {
                ReplayStep::Wait(delay) => {
                    tokio::time::sleep(delay).await;
                    true
                }
                ReplayStep::Emit(event) => sender.send(event).await.is_ok(),
                ReplayStep::StartSession => sender
                    .send(AgentEvent::SessionStarted {
                        session_id: session_id.clone(),
                        model: spec.model.clone(),
                    })
                    .await
                    .is_ok(),
                ReplayStep::Hold => match inner.released.acquire().await {
                    Ok(permit) => {
                        permit.forget();
                        true
                    }
                    Err(_) => false,
                },
            }
        };
        tokio::select! {
            biased;
            () = signal.cancelled() => {
                killed = true;
                break;
            }
            going_on = play => if !going_on { killed = true; break; },
        }
    }
    if killed {
        inner.killed.fetch_add(1, Ordering::SeqCst);
        let _ = sender
            .send(AgentEvent::Failed {
                failure: AgentFailureKind::Cancelled,
                message: "the run was stopped".to_owned(),
            })
            .await;
    }
    // The "process" is gone before the channel closes, as with a real CLI.
    inner.alive.fetch_sub(1, Ordering::SeqCst);
}

/// Folder paths under `/agent-workdirs`, with a record of what was prepared and released.
#[derive(Debug, Default)]
pub struct FakeAgentWorkspaces {
    prepared: Mutex<Vec<TaskId>>,
    released: Mutex<Vec<TaskId>>,
    fail_next: Mutex<Option<String>>,
}

impl FakeAgentWorkspaces {
    /// Tasks whose folder was prepared, in order.
    #[must_use]
    pub fn prepared(&self) -> Vec<TaskId> {
        lock(&self.prepared).clone()
    }

    /// Tasks whose folder was released, in order.
    #[must_use]
    pub fn released(&self) -> Vec<TaskId> {
        lock(&self.released).clone()
    }

    /// Makes the next `prepare` fail with `message`.
    pub fn fail_next(&self, message: &str) {
        *lock(&self.fail_next) = Some(message.to_owned());
    }
}

#[async_trait]
impl AgentWorkspaces for FakeAgentWorkspaces {
    async fn prepare(&self, task_id: TaskId) -> Result<PathBuf, AgentError> {
        if let Some(message) = lock(&self.fail_next).take() {
            return Err(AgentError::Failed { message });
        }
        lock(&self.prepared).push(task_id);
        Ok(PathBuf::from("/agent-workdirs").join(task_id.to_string()))
    }

    async fn release(&self, task_id: TaskId) {
        lock(&self.released).push(task_id);
    }
}
