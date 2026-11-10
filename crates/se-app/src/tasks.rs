//! [`TaskService`]: AI tasks run in the background by an AI CLI (`docs/architecture.md` §9.3,
//! ADR-0027) — the `TaskScheduler` of the architecture.
//!
//! A task is a user request; each execution of its agent is a run (the first one, retries,
//! follow-ups resuming the agent's session). Runs wait in one FIFO queue shared by every open
//! project and at most `maxConcurrentTasks` (setting, 1–8, default 3) run at once. Each run gets
//! an empty working folder ([`AgentWorkspaces`]), is started through an [`AgentProvider`], and its
//! events are appended to the task's log in `project.db` ([`crate::ports::TaskStore`]) and
//! published (`taskEventAppended`); every change of a task is saved and published
//! (`taskUpdated`). Tasks a stopped core left queued or running fail as « interrupted » when
//! their project opens again. The service also keeps what is known about the provider
//! (`aiStatusChanged`) and the AI settings.
//!
//! Each run gets its own MCP bearer token (`crate::agent_access`), minted when it starts and
//! revoked when it ends: an agent can reach the project only while its run is going.
//!
//! Task state lives behind one async mutex, held while a change is saved so saves of a task
//! never overtake each other; events are appended outside it.

mod ai_status;
mod instructions;
mod lifecycle;
mod queue;
mod revert;
mod runner;
pub use revert::REVERT_TASK_LABEL;
pub mod settings;
mod submission;
mod task_state;

use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex as StdMutex, MutexGuard, PoisonError,
        atomic::{AtomicBool, Ordering},
    },
};

use se_domain::{ProjectId, TaskId};
use se_protocol::{AiSettings, AiStatus, ProviderId, Task, TaskEventRecord};
use tokio::sync::Mutex;

use self::queue::RunQueue;
use crate::{
    AppError,
    agent_access::McpAccess,
    ports::{AgentProvider, AgentWorkspaces, AppLibraryRepository, Clock, EventSink, IdGenerator},
    project_actor::ProjectActorHandle,
};

/// The ports the task service works with.
#[derive(Clone)]
pub struct TaskServiceDeps {
    /// The AI providers available, the first being the default.
    pub providers: Vec<Arc<dyn AgentProvider>>,
    /// Working folders of runs.
    pub workspaces: Arc<dyn AgentWorkspaces>,
    /// AI settings (`library.db`).
    pub library: Arc<dyn AppLibraryRepository>,
    /// Events to clients.
    pub events: Arc<dyn EventSink>,
    /// Timestamps of tasks and events.
    pub clock: Arc<dyn Clock>,
    /// Task ids and new agent session ids.
    pub ids: Arc<dyn IdGenerator>,
    /// The MCP endpoint runs act through, and the registry of their tokens.
    pub mcp: McpAccess,
}

/// The AI task use cases. Cheap to clone (shared state).
#[derive(Clone)]
pub struct TaskService {
    shared: Arc<Shared>,
}

/// State shared by the service and its running runs.
struct Shared {
    deps: TaskServiceDeps,
    state: Mutex<State>,
    /// What is known about each provider, once checked.
    statuses: StdMutex<HashMap<ProviderId, AiStatus>>,
    settings: StdMutex<AiSettings>,
    /// Set when the core stops: runs stopped from now on end as interrupted, not cancelled.
    shutting_down: AtomicBool,
}

/// Open projects and the run queue.
struct State {
    projects: HashMap<ProjectId, OpenProject>,
    queue: RunQueue,
}

/// The tasks of an open project, oldest first.
struct OpenProject {
    handle: ProjectActorHandle,
    tasks: Vec<Task>,
}

impl OpenProject {
    fn task_mut(&mut self, task_id: TaskId) -> Result<&mut Task, AppError> {
        let project_id = self.handle.project_id();
        self.tasks
            .iter_mut()
            .find(|task| task.id == task_id)
            .ok_or(AppError::TaskNotFound {
                project_id,
                task_id,
            })
    }
}

impl Shared {
    /// Whether the core is stopping ([`TaskService::begin_shutdown`]).
    fn shutting_down(&self) -> bool {
        self.shutting_down.load(Ordering::SeqCst)
    }
}

impl State {
    fn project_mut(&mut self, project_id: ProjectId) -> Result<&mut OpenProject, AppError> {
        self.projects
            .get_mut(&project_id)
            .ok_or(AppError::ProjectNotOpen { project_id })
    }
}

/// Locks a plain mutex, ignoring poisoning (the data are replaced whole, never half-updated).
fn lock<T>(mutex: &StdMutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl TaskService {
    /// A service with no project open, the default settings and nothing known about providers
    /// yet: call [`TaskService::start`] once at startup.
    #[must_use]
    pub fn new(deps: TaskServiceDeps) -> Self {
        let provider = deps.providers.first().map_or_else(
            || ProviderId("none".to_owned()),
            |provider| provider.info().id,
        );
        let settings = AiSettings {
            provider,
            executable_path: None,
            default_model: None,
            default_effort: None,
            max_concurrent_tasks: AiSettings::DEFAULT_MAX_CONCURRENT_TASKS,
        };
        let limit = usize::from(settings.max_concurrent_tasks);
        Self {
            shared: Arc::new(Shared {
                deps,
                state: Mutex::new(State {
                    projects: HashMap::new(),
                    queue: RunQueue::new(limit),
                }),
                statuses: StdMutex::new(HashMap::new()),
                settings: StdMutex::new(settings),
                shutting_down: AtomicBool::new(false),
            }),
        }
    }

    /// Applies the stored settings (executable path, concurrency).
    ///
    /// # Errors
    /// [`AppError::Storage`] when `library.db` cannot be read.
    pub async fn start(&self) -> Result<(), AppError> {
        let default = lock(&self.shared.settings).provider.clone();
        let mut stored = settings::load(self.shared.deps.library.as_ref(), &default).await?;
        if self.shared.provider(&stored.provider).is_none() {
            stored.provider = default;
        }
        self.shared.apply_settings(&stored).await;
        Ok(())
    }

    /// The AI settings.
    #[must_use]
    pub fn ai_settings(&self) -> AiSettings {
        lock(&self.shared.settings).clone()
    }

    /// Validates and saves `settings`, then applies them: the executable path (the provider is
    /// checked again when it changed) and the number of tasks running at once.
    ///
    /// # Errors
    /// [`AppError::InvalidSetting`]; [`AppError::Storage`].
    pub async fn update_ai_settings(&self, settings: AiSettings) -> Result<AiSettings, AppError> {
        let provider =
            self.shared
                .provider(&settings.provider)
                .ok_or_else(|| AppError::InvalidSetting {
                    name: "provider",
                    reason: format!("no AI provider is called {}", settings.provider),
                })?;
        let settings = settings::validated(settings, &provider.capabilities())?;
        settings::save(self.shared.deps.library.as_ref(), &settings).await?;
        let previous = self.ai_settings();
        self.shared.apply_settings(&settings).await;
        if previous.executable_path != settings.executable_path
            || previous.provider != settings.provider
        {
            self.refresh_ai_status().await;
        }
        Ok(settings)
    }

    /// What is known about the provider in use, checking it the first time.
    pub async fn ai_status(&self) -> AiStatus {
        let provider_id = lock(&self.shared.settings).provider.clone();
        let known = lock(&self.shared.statuses).get(&provider_id).cloned();
        match known {
            Some(status) => status,
            None => self.refresh_ai_status().await,
        }
    }

    /// Checks the provider in use again (installation, login) and publishes `aiStatusChanged`.
    pub async fn refresh_ai_status(&self) -> AiStatus {
        let provider_id = lock(&self.shared.settings).provider.clone();
        self.shared.refresh_status(&provider_id).await
    }

    /// The tasks of an open project, oldest first.
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`].
    pub async fn list(&self, project_id: ProjectId) -> Result<Vec<Task>, AppError> {
        let mut state = self.shared.state.lock().await;
        Ok(state.project_mut(project_id)?.tasks.clone())
    }

    /// The event log of a task.
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`], [`AppError::TaskNotFound`], [`AppError::Storage`].
    pub async fn events(
        &self,
        project_id: ProjectId,
        task_id: TaskId,
    ) -> Result<Vec<TaskEventRecord>, AppError> {
        let store = {
            let mut state = self.shared.state.lock().await;
            let project = state.project_mut(project_id)?;
            project.task_mut(task_id)?;
            Arc::clone(project.handle.tasks())
        };
        Ok(store.load_task_events(task_id).await?)
    }
}

impl Shared {
    /// The registered provider with this id.
    fn provider(&self, id: &ProviderId) -> Option<Arc<dyn AgentProvider>> {
        self.deps
            .providers
            .iter()
            .find(|provider| &provider.info().id == id)
            .cloned()
    }

    /// Makes `settings` current: the provider's executable and the queue's limit.
    async fn apply_settings(self: &Arc<Self>, settings: &AiSettings) {
        if let Some(provider) = self.provider(&settings.provider) {
            provider.set_executable(settings.executable_path.as_ref().map(Into::into));
        }
        *lock(&self.settings) = settings.clone();
        let mut state = self.state.lock().await;
        state
            .queue
            .set_limit(usize::from(settings.max_concurrent_tasks));
        self.dispatch(&mut state);
    }

    fn now(&self) -> i64 {
        self.deps.clock.now_unix_millis()
    }
}
