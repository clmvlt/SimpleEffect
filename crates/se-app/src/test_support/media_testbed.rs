//! [`MediaTestbed`]: a `ProjectService` and a `MediaService` wired to the in-memory fakes, for
//! tests of the media use cases (se-app) and of what serves them (se-core).

use std::{path::Path, sync::Arc, time::Duration};

use se_domain::{Asset, Format, FormatPreset, FrameRate, ProjectId};

use super::{
    FakeFrameExtractor, FakeJobRunner, FakeMedia, FakeMediaTools, FixedClock, InMemoryAppLibrary,
    InMemoryProjectRepository, RecordingEventSink, SequentialIds,
};
use crate::{
    AppError, JobQueue, JobQueueConfig, MediaService, MediaServiceDeps, ProjectService,
    ProjectServiceConfig, ProjectServiceDeps,
};

/// The services and every fake behind them (clones share state with the services).
pub struct MediaTestbed {
    /// Project use cases, observed by `media`.
    pub projects: ProjectService,
    /// Media use cases.
    pub media: Arc<MediaService>,
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
        let deps = ProjectServiceDeps {
            repository: Arc::new(repository.clone()),
            library: library.clone(),
            events: events.clone(),
            clock,
            ids,
        };
        let config = ProjectServiceConfig {
            default_parent_dir: parent_dir.to_path_buf(),
            save_delay: Duration::from_secs(2),
        };
        Self {
            projects: ProjectService::new(deps, config).with_observer(media.clone()),
            media,
            files,
            runner,
            frames,
            tools,
            library,
            repository,
            events,
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
