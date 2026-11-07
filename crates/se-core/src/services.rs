//! Wiring of the use cases to their adapters: projects over se-store, media over se-media, both
//! publishing to the event bus.

use std::{path::PathBuf, sync::Arc};

use se_app::{
    DEFAULT_SAVE_DELAY, JobQueue, JobQueueConfig, MediaService, MediaServiceDeps, ProjectService,
    ProjectServiceConfig, ProjectServiceDeps, ports::AppLibraryRepository,
};
use se_media::{FfmpegJobRunner, FfprobeProber, FsMediaFiles, ToolLocator};
use se_store::{FsProjectRepository, SqliteAppLibrary};

use crate::{
    app_paths::AppPaths,
    error::CoreError,
    rpc::EventBus,
    system_ports::{SystemClock, UuidV7Ids},
};

/// Library setting holding the folder of ffmpeg and ffprobe chosen by the user (searched before
/// the bundled binaries; the settings screen of prompt 09 edits it).
pub const MEDIA_TOOLS_DIR_SETTING: &str = "media.toolsDir";

/// The use cases served by the core.
pub struct Services {
    /// Projects (create, open, edit…), telling the media service about openings and closings.
    pub projects: Arc<ProjectService>,
    /// Media (import, relink, jobs, `/media` lookups).
    pub media: Arc<MediaService>,
}

/// Builds the services over the real adapters.
///
/// # Errors
/// [`CoreError::Library`] when `library.db` cannot be opened.
pub async fn build(paths: &AppPaths, events: &EventBus) -> Result<Services, CoreError> {
    let library =
        Arc::new(SqliteAppLibrary::open(&paths.library_db()).map_err(CoreError::Library)?);
    let tools_dir = match library.setting(MEDIA_TOOLS_DIR_SETTING).await {
        Ok(value) => value.map(PathBuf::from),
        Err(error) => {
            tracing::warn!(%error, "reading the media tools folder failed");
            None
        }
    };
    let media = Arc::new(media_service(tools_dir, events));
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
    })
}

/// The media use cases over ffprobe/ffmpeg found by a [`ToolLocator`].
fn media_service(tools_dir: Option<PathBuf>, events: &EventBus) -> MediaService {
    let tools = ToolLocator::for_current_exe(tools_dir);
    let ids = Arc::new(UuidV7Ids);
    MediaService::new(MediaServiceDeps {
        prober: Arc::new(FfprobeProber::new(tools.clone())),
        files: Arc::new(FsMediaFiles),
        runner: Arc::new(FfmpegJobRunner::new(tools)),
        jobs: JobQueue::new(
            Arc::new(events.clone()),
            ids.clone(),
            JobQueueConfig::default(),
        ),
        ids,
    })
}
