//! Wiring of the use cases to their adapters: projects over se-store, media over se-media, AI
//! tasks over se-agents (acting through `/mcp` with per-run tokens), frames for agents through
//! the renderer, all publishing to the event bus.

use std::{path::PathBuf, sync::Arc};

use se_agents::{FsAgentWorkspaces, providers::claude_code::ClaudeCodeProvider};
use se_app::{
    AgentEditService, DEFAULT_SAVE_DELAY, FrameRenderService, JobQueue, JobQueueConfig, McpAccess,
    MediaService, MediaServiceDeps, ProjectService, ProjectServiceConfig, ProjectServiceDeps,
    TaskService, TaskServiceDeps, TaskTokenRegistry,
    media_service::settings::keys::TOOLS_DIR,
    ports::{AppLibraryRepository, FontCatalog, IdGenerator},
};
use se_fonts::LocalFontCatalog;
use se_media::{FfmpegJobRunner, FfmpegTools, FfprobeProber, FsMediaFiles, ToolLocator};
use se_store::{FsProjectRepository, SqliteAppLibrary};

use crate::{
    app_paths::AppPaths,
    error::CoreError,
    rpc::EventBus,
    system_ports::{RandomTokens, SystemClock, UuidV7Ids},
};

/// The use cases served by the core.
pub struct Services {
    /// Projects (create, open, edit…), telling the media service about openings and closings.
    pub projects: Arc<ProjectService>,
    /// Media (import, relink, jobs, `/media` lookups).
    pub media: Arc<MediaService>,
    /// AI tasks, the AI provider's status and the AI settings.
    pub tasks: Arc<TaskService>,
    /// Edits of AI agents (MCP write tools), committed as AI transactions.
    pub edits: Arc<AgentEditService>,
    /// Bundled and installed fonts (`listFonts`, `/media/fonts`).
    pub fonts: Arc<dyn FontCatalog>,
    /// Frames of sequences drawn by the renderer for agents (`get_frames`, `/media/renders`).
    pub renders: Arc<FrameRenderService>,
    /// MCP tokens of running agents (minted by `tasks`, checked by `/mcp`).
    pub tokens: Arc<TaskTokenRegistry>,
}

/// Builds the services over the real adapters, for a core listening on `port`.
///
/// # Errors
/// [`CoreError::Library`] when `library.db` cannot be opened.
pub async fn build(paths: &AppPaths, events: &EventBus, port: u16) -> Result<Services, CoreError> {
    let library =
        Arc::new(SqliteAppLibrary::open(&paths.library_db()).map_err(CoreError::Library)?);
    let tools_dir = match library.setting(TOOLS_DIR).await {
        Ok(value) => value.filter(|dir| !dir.is_empty()).map(PathBuf::from),
        Err(error) => {
            tracing::warn!(%error, "reading the media tools folder failed");
            None
        }
    };
    let media = Arc::new(media_service(tools_dir, events, library.clone()));
    let tokens = Arc::new(TaskTokenRegistry::new(Arc::new(RandomTokens)));
    let mcp = McpAccess {
        url: format!("http://127.0.0.1:{port}/mcp"),
        tokens: Arc::clone(&tokens),
    };
    let tasks = Arc::new(task_service(paths, events, library.clone(), mcp).await);
    let renders = Arc::new(FrameRenderService::new(
        Arc::new(events.clone()),
        Arc::new(UuidV7Ids),
    ));
    let deps = ProjectServiceDeps {
        repository: Arc::new(FsProjectRepository),
        library,
        events: Arc::new(events.clone()),
        clock: Arc::new(SystemClock),
        ids: Arc::new(UuidV7Ids),
    };
    let config = ProjectServiceConfig {
        default_parent_dir: paths.default_projects_dir(),
        save_delay: DEFAULT_SAVE_DELAY,
    };
    let projects = Arc::new(
        ProjectService::new(deps, config)
            .with_observer(media.clone())
            .with_observer(tasks.clone()),
    );
    let edits = Arc::new(AgentEditService::new(
        Arc::clone(&projects),
        tasks.as_ref().clone(),
        Arc::new(UuidV7Ids),
    ));
    Ok(Services {
        projects,
        media,
        tasks,
        edits,
        fonts: font_catalog(),
        renders,
        tokens,
    })
}

/// The AI task service over Claude Code, with the stored AI settings applied and the provider
/// checked in the background right away (so the status bar shows it without waiting).
async fn task_service(
    paths: &AppPaths,
    events: &EventBus,
    library: Arc<dyn AppLibraryRepository>,
    mcp: McpAccess,
) -> TaskService {
    let workspaces = FsAgentWorkspaces::new(paths.agent_workdirs());
    workspaces.remove_leftovers().await;
    let tasks = TaskService::new(TaskServiceDeps {
        providers: vec![Arc::new(ClaudeCodeProvider::new())],
        workspaces: Arc::new(workspaces),
        library,
        events: Arc::new(events.clone()),
        clock: Arc::new(SystemClock),
        ids: Arc::new(UuidV7Ids),
        mcp,
    });
    if let Err(error) = tasks.start().await {
        tracing::warn!(%error, "reading the AI settings failed; defaults apply");
    }
    let warm_up = tasks.clone();
    tokio::spawn(async move {
        let status = warm_up.refresh_ai_status().await;
        tracing::info!(installation = ?status.installation, auth = ?status.auth, "AI provider checked");
    });
    tasks
}

/// The font catalogue, reading the installed fonts in the background right away so the first
/// `listFonts` does not wait for it.
fn font_catalog() -> Arc<dyn FontCatalog> {
    let catalog: Arc<dyn FontCatalog> = Arc::new(LocalFontCatalog::new());
    let warm_up = Arc::clone(&catalog);
    tokio::spawn(async move {
        let count = warm_up.families().await.len();
        tracing::info!(count, "font catalogue ready");
    });
    catalog
}

/// The media use cases over ffprobe/ffmpeg found by a [`ToolLocator`]. Hardware encoders are
/// detected in the background right away, so the first mezzanine does not wait for it.
fn media_service(
    tools_dir: Option<PathBuf>,
    events: &EventBus,
    library: Arc<dyn AppLibraryRepository>,
) -> MediaService {
    let tools = FfmpegTools::new(ToolLocator::for_current_exe(tools_dir));
    let runner = Arc::new(FfmpegJobRunner::new(tools.clone()));
    let ids: Arc<dyn IdGenerator> = Arc::new(UuidV7Ids);
    let warm_up = tools.clone();
    tokio::spawn(async move { warm_up.warm_up().await });
    MediaService::new(MediaServiceDeps {
        prober: Arc::new(FfprobeProber::new(tools.locator().clone())),
        files: Arc::new(FsMediaFiles),
        runner: runner.clone(),
        frames: runner,
        tools: Arc::new(tools),
        jobs: JobQueue::new(
            Arc::new(events.clone()),
            ids.clone(),
            JobQueueConfig::default(),
        ),
        library,
        ids,
        clock: Arc::new(SystemClock),
    })
}
