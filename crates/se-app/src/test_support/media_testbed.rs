//! [`MediaTestbed`]: a `ProjectService`, a `MediaService`, a `TaskService`, an
//! `AgentEditService`, a `FrameRenderService`, a `TranscriptionService`, a `ReframeService` and
//! an `ExportService` wired to the in-memory fakes, for tests of the media, task, agent,
//! transcription, reframing and export use cases (se-app) and of what serves them (se-mcp,
//! se-core).

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use se_domain::{Asset, Format, FormatPreset, FrameRate, ProjectId};

use super::{
    FakeAgentWorkspaces, FakeDownloader, FakeExportOutputs, FakeFrameExtractor, FakeFrameSampler,
    FakeJobRunner, FakeMedia, FakeMediaTools, FakeModelFiles, FakeRendererLink,
    FakeSpeechExtractor, FakeSubjectDetector, FakeSubtitleFiles, FakeTranscriber, FixedClock,
    InMemoryAppLibrary, InMemoryBrandKitLibrary, InMemoryProjectRepository,
    InMemoryTemplateLibrary, RecordingEventSink, ReplayProvider, SequentialIds, SequentialTokens,
    testbed_services::{export_service, library_services, reframing_service, speech_services},
};
use crate::{
    AgentEditService, AppError, BrandKitService, CaptionService, ExportRegistry, ExportService,
    FrameRenderService, JobQueue, JobQueueConfig, McpAccess, MediaService, MediaServiceDeps,
    ProjectService, ProjectServiceConfig, ProjectServiceDeps, ReframeService, SmartCutService,
    TaskService, TaskServiceDeps, TaskTokenRegistry, TemplateCheckService, TemplateLibraryService,
    TranscriptionService, TranscriptionServiceDeps,
};

/// The MCP URL runs of the testbed's task service are given.
pub const TESTBED_MCP_URL: &str = "http://127.0.0.1:4100/mcp";

/// The models folder of the testbed's transcription service.
pub const TESTBED_MODELS_DIR: &str = "/app-data/models/whisper";

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
    /// The renderer connection of `renders` and `template_checks`.
    pub renderer: Arc<FakeRendererLink>,
    /// Motion template checks over `renderer` (prompt 24).
    pub template_checks: Arc<TemplateCheckService>,
    /// Transcription use cases (observing projects, told about imports); not started.
    pub transcription: TranscriptionService,
    /// Smart cuts over `projects`, `media` and `transcription` (prompt 22).
    pub smart_cuts: SmartCutService,
    /// Captions over `projects`, `transcription` and `subtitles` (prompt 23).
    pub captions: CaptionService,
    /// SRT files of `captions`.
    pub subtitles: FakeSubtitleFiles,
    /// The template library of `library_templates` (prompt 25).
    pub template_library: InMemoryTemplateLibrary,
    /// Template library use cases over `projects` and `template_checks`.
    pub library_templates: TemplateLibraryService,
    /// The brand kits of `brand_kits` (logo copies appear in `files`).
    pub brand_kit_library: InMemoryBrandKitLibrary,
    /// Brand kit use cases over `projects` and `media`.
    pub brand_kits: BrandKitService,
    /// whisper.cpp of `transcription`.
    pub transcriber: FakeTranscriber,
    /// Speech audio extractions.
    pub extractor: FakeSpeechExtractor,
    /// The models folder (and files to import).
    pub model_files: FakeModelFiles,
    /// Model downloads (writing into `model_files`).
    pub downloader: FakeDownloader,
    /// Automatic reframing over `projects`, `media`, `sampler` and `detector` (prompt 26).
    pub reframing: ReframeService,
    /// Pictures sampled for face detection.
    pub sampler: FakeFrameSampler,
    /// The face detector.
    pub detector: FakeSubjectDetector,
    /// Exports over `export_outputs`, drawn by `renderer`, cancelled when projects close.
    pub exports: ExportService,
    /// Where exports write.
    pub export_outputs: FakeExportOutputs,
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
        let jobs = JobQueue::new(events.clone(), ids.clone(), JobQueueConfig::default());
        let fake_media = FakeMediaPorts::new();
        let media = fake_media.service(&jobs, &library, &ids, &clock);
        let FakeMediaPorts {
            files,
            runner,
            frames,
            tools,
        } = fake_media;
        let fakes = TranscriptionFakes::default();
        let transcription = TranscriptionService::new(TranscriptionServiceDeps {
            jobs: jobs.clone(),
            library: library.clone(),
            events: events.clone(),
            ids: ids.clone(),
            clock: clock.clone(),
            ..fakes.deps()
        });
        media.add_import_listener(Arc::new(transcription.clone()));
        let runtime = Runtime {
            events: events.clone(),
            clock: clock.clone(),
            ids: ids.clone(),
        };
        let (tasks, agent, workspaces, tokens) = task_service(&library, &runtime);
        let renderer = Arc::new(FakeRendererLink::default());
        let renders = Arc::new(FrameRenderService::new(renderer.clone(), ids.clone()));
        let template_checks = Arc::new(TemplateCheckService::new(renderer.clone(), ids.clone()));
        let export_registry = Arc::new(ExportRegistry::new(jobs.clone(), events.clone()));
        let projects = Arc::new(
            project_service(parent_dir, &repository, &library, &runtime)
                .with_observer(media.clone())
                .with_observer(Arc::new(tasks.clone()))
                .with_observer(Arc::new(transcription.clone()))
                .with_observer(export_registry.clone()),
        );
        let edits = Arc::new(AgentEditService::new(
            projects.clone(),
            tasks.clone(),
            ids.clone(),
        ));
        let (smart_cuts, captions, subtitles) =
            speech_services(&projects, &media, &transcription, &library);
        let (template_library, library_templates, brand_kit_library, brand_kits) =
            library_services(&projects, &media, &files, &template_checks, &runtime);
        let (reframing, sampler, detector) = reframing_service(&projects, &media, jobs, &runtime);
        let (exports, export_outputs) = export_service(
            &projects,
            export_registry,
            &renderer,
            &files,
            &library,
            &runtime,
        );
        Self {
            exports,
            export_outputs,
            reframing,
            sampler,
            detector,
            template_library,
            library_templates,
            brand_kit_library,
            brand_kits,
            captions,
            subtitles,
            smart_cuts,
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
            template_checks,
            transcription,
            transcriber: fakes.transcriber,
            extractor: fakes.extractor,
            model_files: fakes.model_files,
            downloader: fakes.downloader,
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

/// The transcription fakes of a testbed (clones share state with the service).
struct TranscriptionFakes {
    transcriber: FakeTranscriber,
    extractor: FakeSpeechExtractor,
    model_files: FakeModelFiles,
    downloader: FakeDownloader,
}

impl Default for TranscriptionFakes {
    fn default() -> Self {
        let model_files = FakeModelFiles::default();
        Self {
            transcriber: FakeTranscriber::default(),
            extractor: FakeSpeechExtractor::default(),
            downloader: FakeDownloader::new(model_files.clone()),
            model_files,
        }
    }
}

impl TranscriptionFakes {
    /// Service dependencies over these fakes; jobs, library, events, ids and clock are the
    /// testbed's own (replaced by the caller).
    fn deps(&self) -> TranscriptionServiceDeps {
        let events = Arc::new(RecordingEventSink::default());
        let ids = Arc::new(SequentialIds::default());
        TranscriptionServiceDeps {
            transcriber: Arc::new(self.transcriber.clone()),
            extractor: Arc::new(self.extractor.clone()),
            downloader: Arc::new(self.downloader.clone()),
            model_files: Arc::new(self.model_files.clone()),
            jobs: JobQueue::new(events.clone(), ids.clone(), JobQueueConfig::default()),
            library: Arc::new(InMemoryAppLibrary::default()),
            events,
            ids,
            clock: Arc::new(FixedClock::default()),
            models_dir: PathBuf::from(TESTBED_MODELS_DIR),
            cpu_threads: 16,
        }
    }
}

/// Lets spawned tasks (queued jobs) run.
pub async fn settle() {
    for _ in 0..100 {
        tokio::task::yield_now().await;
    }
}

/// The project service over the in-memory folders, saving 2 s after edits.
fn project_service(
    parent_dir: &Path,
    repository: &InMemoryProjectRepository,
    library: &Arc<InMemoryAppLibrary>,
    runtime: &Runtime,
) -> ProjectService {
    let deps = ProjectServiceDeps {
        repository: Arc::new(repository.clone()),
        library: library.clone(),
        events: runtime.events.clone(),
        clock: runtime.clock.clone(),
        ids: runtime.ids.clone(),
    };
    let config = ProjectServiceConfig {
        default_parent_dir: parent_dir.to_path_buf(),
        save_delay: Duration::from_secs(2),
    };
    ProjectService::new(deps, config)
}

/// The runtime fakes shared by every service.
pub(super) struct Runtime {
    pub(super) events: Arc<RecordingEventSink>,
    pub(super) clock: Arc<FixedClock>,
    pub(super) ids: Arc<SequentialIds>,
}

/// The task service over a replayed AI provider, with its workspaces and MCP tokens.
fn task_service(
    library: &Arc<InMemoryAppLibrary>,
    runtime: &Runtime,
) -> (
    TaskService,
    ReplayProvider,
    Arc<FakeAgentWorkspaces>,
    Arc<TaskTokenRegistry>,
) {
    let agent = ReplayProvider::default();
    let workspaces = Arc::new(FakeAgentWorkspaces::default());
    let tokens = Arc::new(TaskTokenRegistry::new(
        Arc::new(SequentialTokens::default()),
    ));
    let tasks = TaskService::new(TaskServiceDeps {
        providers: vec![Arc::new(agent.clone())],
        workspaces: workspaces.clone(),
        library: library.clone(),
        events: runtime.events.clone(),
        clock: runtime.clock.clone(),
        ids: runtime.ids.clone(),
        mcp: McpAccess {
            url: TESTBED_MCP_URL.to_owned(),
            tokens: tokens.clone(),
        },
    });
    (tasks, agent, workspaces, tokens)
}

/// The fakes behind the media service.
struct FakeMediaPorts {
    files: Arc<FakeMedia>,
    runner: Arc<FakeJobRunner>,
    frames: Arc<FakeFrameExtractor>,
    tools: Arc<FakeMediaTools>,
}

impl FakeMediaPorts {
    /// Media files, a runner writing into them, frames and tools.
    fn new() -> Self {
        let files = Arc::new(FakeMedia::default());
        let runner = Arc::new(FakeJobRunner::default());
        runner.write_files_to(files.clone());
        let frames = Arc::new(FakeFrameExtractor::new(files.clone()));
        Self {
            files,
            runner,
            frames,
            tools: Arc::new(FakeMediaTools::default()),
        }
    }

    /// The media service over these fakes.
    fn service(
        &self,
        jobs: &JobQueue,
        library: &Arc<InMemoryAppLibrary>,
        ids: &Arc<SequentialIds>,
        clock: &Arc<FixedClock>,
    ) -> Arc<MediaService> {
        Arc::new(MediaService::new(MediaServiceDeps {
            prober: self.files.clone(),
            files: self.files.clone(),
            runner: self.runner.clone(),
            frames: self.frames.clone(),
            tools: self.tools.clone(),
            jobs: jobs.clone(),
            library: library.clone(),
            ids: ids.clone(),
            clock: clock.clone(),
        }))
    }
}
