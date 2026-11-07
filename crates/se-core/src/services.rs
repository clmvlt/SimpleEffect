//! Wiring of the use cases to their adapters: projects over se-store, media over se-media, both
//! publishing to the event bus.

use std::{path::PathBuf, sync::Arc};

use se_app::{
    DEFAULT_SAVE_DELAY, JobQueue, JobQueueConfig, MediaService, MediaServiceDeps, ProjectService,
    ProjectServiceConfig, ProjectServiceDeps,
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
    system_ports::{SystemClock, UuidV7Ids},
};

/// The use cases served by the core.
pub struct Services {
    /// Projects (create, open, edit…), telling the media service about openings and closings.
    pub projects: Arc<ProjectService>,
    /// Media (import, relink, jobs, `/media` lookups).
    pub media: Arc<MediaService>,
    /// Bundled and installed fonts (`listFonts`, `/media/fonts`).
    pub fonts: Arc<dyn FontCatalog>,
}

/// Builds the services over the real adapters.
///
/// # Errors
/// [`CoreError::Library`] when `library.db` cannot be opened.
pub async fn build(paths: &AppPaths, events: &EventBus) -> Result<Services, CoreError> {
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
    let projects = ProjectService::new(deps, config).with_observer(media.clone());
    Ok(Services {
        projects: Arc::new(projects),
        media,
        fonts: font_catalog(),
    })
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
