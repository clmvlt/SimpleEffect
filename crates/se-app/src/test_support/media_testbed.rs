//! [`MediaTestbed`]: a `ProjectService`, a `MediaService`, a `TaskService`, an
//! `AgentEditService` and a `FrameRenderService` wired to the in-memory fakes, for tests of the
//! media, task and agent use cases (se-app) and of what serves them (se-mcp, se-core).

use std::{path::Path, sync::Arc, time::Duration};

use se_domain::{Asset, Format, FormatPreset, FrameRate, ProjectId};

use super::{
    FakeAgentWorkspaces, FakeFrameExtractor, FakeJobRunner, FakeMedia, FakeMediaTools,
    FakeRendererLink, FixedClock, InMemoryAppLibrary, InMemoryProjectRepository,
    RecordingEventSink, ReplayProvider, SequentialIds, SequentialTokens,
};
use crate::{
    AgentEditService, AppError, FrameRenderService, JobQueue, JobQueueConfig, McpAccess,
    MediaService, MediaServiceDeps, ProjectService, ProjectServiceConfig, ProjectServiceDeps,
    TaskService, TaskServiceDeps, TaskTokenRegistry,
};

/// The MCP URL runs of the testbed's task service are given.
pub const TESTBED_MCP_URL: &str = "http://127.0.0.1:4100/mcp";

/// The services and every fake behind them (clones share state with the services).
pub struct MediaTestbed {
    /// Project use cases, observed by `media` and `tasks`.
    pub projects: Arc<ProjectService>,
    /// Media use cases.
    pub media: Arc<MediaService>,
    /// AI task use cases (also observing projects).
    pub tasks: TaskService,
    /// Edits of AI agents (over `projects`, recording changesets on `tasks`).
    pub edits: Arc<AgentEditService>,
    /// The AI provider of `tasks`.
    pub agent: ReplayProvider,
    /// Working folders of agent runs.
    pub workspaces: Arc<FakeAgentWorkspaces>,
    /// The clock of every service.
    pub clock: Arc<FixedClock>,
    /// Media files, probes and files written by jobs.
    pub files: Arc<FakeMedia>,
    /// The job runner (writes transcodes into `files`).
    pub runner: Arc<FakeJobRunner>,
    /// The frame extractor (writes into `files`).
    pub frames: Arc<FakeFrameExtractor>,
    /// The tools.
    pub tools: Arc<FakeMediaTools>,
    /// Settings and recent projects.
    pub library: Arc<InMemoryAppLibrary>,
    /// Project folders.
    pub repository: InMemoryProjectRepository,
    /// Every event published.
    pub events: Arc<RecordingEventSink>,
    /// MCP tokens of running agents (shared with `tasks`).
    pub tokens: Arc<TaskTokenRegistry>,
    /// Composited frames for agents, over `renderer`.
    pub renders: Arc<FrameRenderService>,
    /// The renderer connection of `renders`.
    pub renderer: Arc<FakeRendererLink>,
}

impl MediaTestbed {
    /// Services whose new projects go to `parent_dir`.
    #[must_use]
    pub fn new(parent_dir: &Path) -> Self {
        let repository = InMemoryProjectRepository::default();
        let events = Arc::new(RecordingEventSink::default());
        let ids = Arc::new(SequentialIds::default());
        let clock = Arc::new(FixedClock::default());
        let library = Arc::new(InMemoryAppLibrary::default());
        let files = Arc::new(FakeMedia::default());
        let runner = Arc::new(FakeJobRunner::default());
        runner.write_files_to(files.clone());
        let frames = Arc::new(FakeFrameExtractor::new(files.clone()));
        let tools = Arc::new(FakeMediaTools::default());
        let media = Arc::new(MediaService::new(MediaServiceDeps {
            prober: files.clone(),
            files: files.clone(),
            runner: runner.clone(),
            frames: frames.clone(),
            tools: tools.clone(),
            jobs: JobQueue::new(events.clone(), ids.clone(), JobQueueConfig::default()),
            library: library.clone(),
            ids: ids.clone(),
            clock: clock.clone(),
        }));
        let agent = ReplayProvider::default();
        let workspaces = Arc::new(FakeAgentWorkspaces::default());
        let tokens = Arc::new(TaskTokenRegistry::new(
            Arc::new(SequentialTokens::default()),
        ));
        let tasks = TaskService::new(TaskServiceDeps {
            providers: vec![Arc::new(agent.clone())],
            workspaces: workspaces.clone(),
            library: library.clone(),
            events: events.clone(),
            clock: clock.clone(),
            ids: ids.clone(),
            mcp: McpAccess {
                url: TESTBED_MCP_URL.to_owned(),
                tokens: tokens.clone(),
            },
        });
        let renderer = Arc::new(FakeRendererLink::default());
        let renders = Arc::new(FrameRenderService::new(renderer.clone(), ids.clone()));
        let deps = ProjectServiceDeps {
            repository: Arc::new(repository.clone()),
            library: library.clone(),
            events: events.clone(),
            clock: clock.clone(),
            ids: ids.clone(),
        };
        let config = ProjectServiceConfig {
            default_parent_dir: parent_dir.to_path_buf(),
            save_delay: Duration::from_secs(2),
        };
        let projects = Arc::new(
            ProjectService::new(deps, config)
                .with_observer(media.clone())
                .with_observer(Arc::new(tasks.clone())),
        );
        let edits = Arc::new(AgentEditService::new(projects.clone(), tasks.clone(), ids));
        Self {
            projects,
            media,
            tasks,
            edits,
            agent,
            workspaces,
            clock,
            files,
            runner,
            frames,
            tools,
            library,
            repository,
            events,
            tokens,
            renders,
            renderer,
        }
    }

    /// Creates and opens the project `name` (1080p, 25 fps) in `parent_dir`.
    ///
    /// # Errors
    /// The creation error.
    pub async fn create_project(
        &self,
        parent_dir: &Path,
        name: &str,
    ) -> Result<ProjectId, AppError> {
        let format = Format::from_preset(FormatPreset::Hd16x9, FrameRate::FPS_25)
            .ok_or_else(|| AppError::Internal("the 1080p preset is invalid".to_owned()))?;
        let session = self
            .projects
            .create_project(name, parent_dir, format)
            .await?;
        Ok(session.snapshot.project.id)
    }

    /// The assets of an open project.
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`].
    pub async fn assets(&self, project_id: ProjectId) -> Result<Vec<Asset>, AppError> {
        Ok(self
            .projects
            .project_session(project_id)
            .await?
            .snapshot
            .assets)
    }
}

/// Lets spawned tasks (queued jobs) run.
pub async fn settle() {
    for _ in 0..100 {
        tokio::task::yield_now().await;
    }
}
