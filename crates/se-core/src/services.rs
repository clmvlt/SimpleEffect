//! Wiring of the use cases to their adapters: projects over se-store, media over se-media, AI
//! tasks over se-agents (acting through `/mcp` with per-run tokens), frames for agents through
//! the renderer, transcription over se-transcribe (whisper.cpp) sharing the media job queue, smart
//! cuts over projects, media and transcription, automatic reframing over se-media's frame
//! sampling and se-vision's face detector (sharing the job queue too), exports drawn by the
//! renderer and written by se-media (prompt 27), all publishing to the event bus.

use std::{path::PathBuf, sync::Arc};

use se_agents::{FsAgentWorkspaces, providers::claude_code::ClaudeCodeProvider};
use se_app::{
    AgentEditService, BrandKitService, BrandKitServiceDeps, CaptionService, CaptionServiceDeps,
    DEFAULT_SAVE_DELAY, DiagnosticsService, ExportRegistry, ExportService, ExportServiceDeps,
    FrameRenderService, JobQueue, JobQueueConfig, McpAccess, MediaService, MediaServiceDeps,
    ProjectService, ProjectServiceConfig, ProjectServiceDeps, ReframeService, ReframeServiceDeps,
    SmartCutService, SmartCutServiceDeps, TaskService, TaskServiceDeps, TaskTokenRegistry,
    TemplateCheckService, TemplateLibraryDeps, TemplateLibraryService, TranscriptionService,
    TranscriptionServiceDeps,
    media_service::settings::keys::TOOLS_DIR,
    ports::{AppLibraryRepository, BrandKitLibrary, FontCatalog, IdGenerator, MediaProber},
};
use se_fonts::LocalFontCatalog;
use se_media::{
    FfmpegJobRunner, FfmpegTools, FfprobeProber, FsExportOutputs, FsMediaFiles, ToolLocator,
};
use se_store::{
    FsBrandKitLibrary, FsDiagnosticsFiles, FsProjectRepository, FsTemplateLibrary, SqliteAppLibrary,
};
use se_transcribe::{FsModelFiles, HttpDownloader, WhisperLocator, WhisperTranscriber};
use se_vision::{RuntimeLocator, YuNetDetector};

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
    /// Motion template modules checked by the renderer (prompt 24, `reportTemplateCheck`).
    pub template_checks: Arc<TemplateCheckService>,
    /// MCP tokens of running agents (minted by `tasks`, checked by `/mcp`).
    pub tokens: Arc<TaskTokenRegistry>,
    /// Transcription (models, downloads, jobs, transcripts), observing projects and imports.
    pub transcription: TranscriptionService,
    /// Smart cuts: text-based editing, filler words, silences (prompt 22).
    pub smart_cuts: SmartCutService,
    /// Captions over SRT files on disk (prompt 23).
    pub captions: CaptionService,
    /// The template library in `<data>/templates` (prompt 25).
    pub library_templates: TemplateLibraryService,
    /// Brand kits in `<data>/brand-kits` (prompt 25).
    pub brand_kits: BrandKitService,
    /// The brand kits on disk (logo files served at `/media/brand-kits`).
    pub brand_kit_files: Arc<dyn BrandKitLibrary>,
    /// Automatic reframing (prompt 26).
    pub reframing: ReframeService,
    /// Video exports drawn by the renderer, written by se-media (prompt 27).
    pub exports: ExportService,
    /// Diagnostics archives of the logs and the machine (prompt 30).
    pub diagnostics: DiagnosticsService,
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
    let ids: Arc<dyn IdGenerator> = Arc::new(UuidV7Ids);
    let jobs = JobQueue::new(Arc::new(events.clone()), ids, JobQueueConfig::default());
    let export_registry = Arc::new(ExportRegistry::new(jobs.clone(), Arc::new(events.clone())));
    let tools = FfmpegTools::new(ToolLocator::for_current_exe(tools_dir));
    let locator = tools.locator().clone();
    let runner = Arc::new(FfmpegJobRunner::new(tools.clone()));
    let prober: Arc<dyn MediaProber> = Arc::new(FfprobeProber::new(tools.locator().clone()));
    let media = Arc::new(media_service(
        tools,
        runner.clone(),
        jobs.clone(),
        library.clone(),
    ));
    let reframing_jobs = jobs.clone();
    let sampler = runner.clone();
    let transcription = transcription_service(paths, events, library.clone(), jobs, runner)?;
    media.add_import_listener(Arc::new(transcription.clone()));
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
    let template_checks = Arc::new(TemplateCheckService::new(
        Arc::new(events.clone()),
        Arc::new(UuidV7Ids),
    ));
    let projects = Arc::new(
        project_service(paths, events, library.clone())
            .with_observer(media.clone())
            .with_observer(tasks.clone())
            .with_observer(Arc::new(transcription.clone()))
            .with_observer(export_registry.clone()),
    );
    let exports = export_service(
        paths,
        events,
        &projects,
        export_registry,
        &locator,
        library.clone(),
    );
    let edits = Arc::new(AgentEditService::new(
        Arc::clone(&projects),
        tasks.as_ref().clone(),
        Arc::new(UuidV7Ids),
    ));
    let smart_cuts = SmartCutService::new(SmartCutServiceDeps {
        projects: Arc::clone(&projects),
        media: Arc::clone(&media),
        transcription: transcription.clone(),
        library,
    });
    let captions = CaptionService::new(CaptionServiceDeps {
        projects: Arc::clone(&projects),
        transcription: transcription.clone(),
        subtitles: Arc::new(se_export::FsSubtitleFiles),
    });
    let (library_templates, brand_kits, brand_kit_files) =
        library_services(paths, events, &projects, &media, &template_checks, prober);
    let reframing = reframing_service(events, &projects, &media, reframing_jobs, sampler);
    Ok(Services {
        diagnostics: diagnostics_service(paths),
        exports,
        reframing,
        library_templates,
        brand_kits,
        brand_kit_files,
        captions,
        smart_cuts,
        projects,
        media,
        tasks,
        edits,
        fonts: font_catalog(),
        renders,
        template_checks,
        tokens,
        transcription,
    })
}

/// Diagnostics archives of the core's logs, masking the user's home folder (se-store files).
fn diagnostics_service(paths: &AppPaths) -> DiagnosticsService {
    DiagnosticsService::new(
        Arc::new(FsDiagnosticsFiles),
        Arc::new(SystemClock),
        paths.logs_dir.clone(),
        directories::BaseDirs::new()
            .map(|dirs| dirs.home_dir().to_path_buf())
            .as_deref(),
    )
}

/// Projects in folders on disk (se-store), new ones proposed in the documents folder.
fn project_service(
    paths: &AppPaths,
    events: &EventBus,
    library: Arc<SqliteAppLibrary>,
) -> ProjectService {
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
    ProjectService::new(deps, config)
}

/// Exports (prompt 27): files written by se-media with the ffmpeg of `locator`, checked by its
/// ffprobe, drawn by the renderer behind the event bus; first proposed in the videos folder.
fn export_service(
    paths: &AppPaths,
    events: &EventBus,
    projects: &Arc<ProjectService>,
    registry: Arc<ExportRegistry>,
    locator: &ToolLocator,
    library: Arc<SqliteAppLibrary>,
) -> ExportService {
    ExportService::new(ExportServiceDeps {
        registry,
        projects: Arc::clone(projects),
        outputs: Arc::new(FsExportOutputs::new(locator.clone())),
        prober: Arc::new(FfprobeProber::new(locator.clone())),
        renderer: Arc::new(events.clone()),
        library,
        events: Arc::new(events.clone()),
        clock: Arc::new(SystemClock),
        ids: Arc::new(UuidV7Ids),
        default_directory: paths.default_exports_dir(),
    })
}

/// Automatic reframing (prompt 26): frames sampled by ffmpeg, faces found by YuNet over the ONNX
/// Runtime library next to the binary (loaded on first use, on half the processor's threads).
fn reframing_service(
    events: &EventBus,
    projects: &Arc<ProjectService>,
    media: &Arc<MediaService>,
    jobs: JobQueue,
    sampler: Arc<FfmpegJobRunner>,
) -> ReframeService {
    let threads = std::thread::available_parallelism().map_or(4, std::num::NonZero::get);
    let detector = YuNetDetector::new(RuntimeLocator::for_current_exe(), (threads / 2).max(1));
    ReframeService::new(ReframeServiceDeps {
        projects: Arc::clone(projects),
        media: Arc::clone(media),
        jobs,
        sampler,
        detector: Arc::new(detector),
        events: Arc::new(events.clone()),
        clock: Arc::new(SystemClock),
    })
}

/// The template library and brand kits in the app data folder (prompt 25), and the kits'
/// files (served at `/media/brand-kits`).
fn library_services(
    paths: &AppPaths,
    events: &EventBus,
    projects: &Arc<ProjectService>,
    media: &Arc<MediaService>,
    template_checks: &Arc<TemplateCheckService>,
    prober: Arc<dyn MediaProber>,
) -> (
    TemplateLibraryService,
    BrandKitService,
    Arc<dyn BrandKitLibrary>,
) {
    let library_templates = TemplateLibraryService::new(TemplateLibraryDeps {
        library: Arc::new(FsTemplateLibrary::new(paths.templates())),
        projects: Arc::clone(projects),
        checks: Arc::clone(template_checks),
        events: Arc::new(events.clone()),
        clock: Arc::new(SystemClock),
        ids: Arc::new(UuidV7Ids),
    });
    let brand_kit_files: Arc<dyn BrandKitLibrary> =
        Arc::new(FsBrandKitLibrary::new(paths.brand_kits()));
    let brand_kits = BrandKitService::new(BrandKitServiceDeps {
        library: Arc::clone(&brand_kit_files),
        projects: Arc::clone(projects),
        media: Arc::clone(media),
        prober,
        events: Arc::new(events.clone()),
        clock: Arc::new(SystemClock),
        ids: Arc::new(UuidV7Ids),
    });
    (library_templates, brand_kits, brand_kit_files)
}

/// The transcription service over whisper.cpp (bundled builds, the GPU pack under
/// `<data>/runtimes`), models in `<data>/models/whisper`, sharing the media job queue; the
/// machine is looked at in the background right away.
fn transcription_service(
    paths: &AppPaths,
    events: &EventBus,
    library: Arc<dyn AppLibraryRepository>,
    jobs: JobQueue,
    runner: Arc<FfmpegJobRunner>,
) -> Result<TranscriptionService, CoreError> {
    let downloader = Arc::new(HttpDownloader::new().map_err(CoreError::Transcription)?);
    let threads = std::thread::available_parallelism().map_or(4, std::num::NonZero::get);
    let runtimes = paths.runtimes();
    let locator = WhisperLocator::for_current_exe(WhisperTranscriber::gpu_pack_dir(&runtimes));
    let transcriber = WhisperTranscriber::new(locator, runtimes, downloader.clone(), threads);
    let service = TranscriptionService::new(TranscriptionServiceDeps {
        transcriber: Arc::new(transcriber),
        extractor: runner,
        downloader,
        model_files: Arc::new(FsModelFiles),
        jobs,
        library,
        events: Arc::new(events.clone()),
        ids: Arc::new(UuidV7Ids),
        clock: Arc::new(SystemClock),
        models_dir: paths.whisper_models(),
        cpu_threads: threads,
    });
    let warm_up = service.clone();
    tokio::spawn(async move {
        if let Err(error) = warm_up.start().await {
            tracing::warn!(%error, "reading the transcription preferences failed; defaults apply");
        }
    });
    Ok(service)
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

/// The media use cases over ffprobe/ffmpeg found by `tools`, running `jobs`. Hardware encoders
/// are detected in the background right away, so the first mezzanine does not wait for it.
fn media_service(
    tools: FfmpegTools,
    runner: Arc<FfmpegJobRunner>,
    jobs: JobQueue,
    library: Arc<dyn AppLibraryRepository>,
) -> MediaService {
    let ids: Arc<dyn IdGenerator> = Arc::new(UuidV7Ids);
    let warm_up = tools.clone();
    tokio::spawn(async move { warm_up.warm_up().await });
    MediaService::new(MediaServiceDeps {
        prober: Arc::new(FfprobeProber::new(tools.locator().clone())),
        files: Arc::new(FsMediaFiles),
        runner: runner.clone(),
        frames: runner,
        tools: Arc::new(tools),
        jobs,
        library,
        ids,
        clock: Arc::new(SystemClock),
    })
}
