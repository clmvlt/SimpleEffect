//! [`MediaService`]: the media use cases behind the protocol (import, relink, remove, decodability
//! reports, jobs) and the lifecycle hooks that check missing files when a project opens and stop
//! its jobs when it closes (ADR-0018).
//!
//! Like [`crate::ProjectService`], it never mutates a document itself: every change is a
//! transaction sent to the project's actor. User actions (import, relink, remove) are user
//! transactions (undoable); facts the app observed (missing files, decodability) are
//! [`se_domain::Actor::System`] transactions, journaled but never undone.

mod import;
pub mod media_paths;
mod opening;
mod relink;
mod scheduling;

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

use async_trait::async_trait;
use se_domain::{Asset, AssetId, AssetKind, JobId, ProjectId};
use se_protocol::JobSnapshot;

use crate::{
    AppError,
    jobs::JobQueue,
    ports::{IdGenerator, MediaFiles, MediaJobRunner, MediaProber},
    project_actor::ProjectActorHandle,
    project_service::ProjectObserver,
};

/// Transaction labels of user actions (undo menu). Stable identifiers: the renderer translates
/// them (`apps/desktop/src/app/editor/history-label.ts`).
pub mod labels {
    /// Importing media files.
    pub const IMPORT_MEDIA: &str = "Import media";
    /// Pointing an asset at another file.
    pub const RELINK_MEDIA: &str = "Relink media";
    /// Removing an asset.
    pub const REMOVE_MEDIA: &str = "Remove media";
    /// Flags of media found missing (or found again) when a project opens (system, not undoable).
    pub const DETECT_MISSING_MEDIA: &str = "Detect missing media";
    /// Decodability reported by the renderer (system, not undoable).
    pub const RECORD_DECODABILITY: &str = "Record decodability";
}

/// The ports and services the media use cases work with.
#[derive(Clone)]
pub struct MediaServiceDeps {
    /// Reads streams of media files.
    pub prober: Arc<dyn MediaProber>,
    /// Existence and fingerprints of files.
    pub files: Arc<dyn MediaFiles>,
    /// Produces thumbnails and peaks.
    pub runner: Arc<dyn MediaJobRunner>,
    /// Runs the jobs.
    pub jobs: JobQueue,
    /// New asset and transaction ids.
    pub ids: Arc<dyn IdGenerator>,
}

/// Where the files of an asset are, for the `/media` routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetFiles {
    /// The original file.
    pub original: PathBuf,
    /// Its kind.
    pub kind: AssetKind,
    /// The folder of its derived files (thumbnails, peaks), when its fingerprint is known.
    pub cache_dir: Option<PathBuf>,
}

/// The media use cases.
pub struct MediaService {
    deps: MediaServiceDeps,
    /// Open projects, as told by [`ProjectObserver`] calls.
    open: Mutex<HashMap<ProjectId, ProjectActorHandle>>,
}

impl MediaService {
    /// A service with no project open.
    #[must_use]
    pub fn new(deps: MediaServiceDeps) -> Self {
        Self {
            deps,
            open: Mutex::new(HashMap::new()),
        }
    }

    fn registry(&self) -> MutexGuard<'_, HashMap<ProjectId, ProjectActorHandle>> {
        // The map holds handles only; a panic elsewhere cannot leave it inconsistent.
        self.open.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The actor of an open project.
    fn handle(&self, project_id: ProjectId) -> Result<ProjectActorHandle, AppError> {
        self.registry()
            .get(&project_id)
            .cloned()
            .ok_or(AppError::ProjectNotOpen { project_id })
    }

    /// The jobs of an open project: queued, running and finished in this session.
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`].
    pub fn list_jobs(&self, project_id: ProjectId) -> Result<Vec<JobSnapshot>, AppError> {
        self.handle(project_id)?;
        Ok(self.deps.jobs.list(project_id))
    }

    /// Cancels a queued or running job.
    ///
    /// # Errors
    /// [`AppError::JobNotFound`].
    pub fn cancel_job(&self, job_id: JobId) -> Result<(), AppError> {
        self.deps.jobs.cancel(job_id)
    }

    /// Where the files of an asset of any open project are; `None` if no open project has it.
    pub async fn asset_files(&self, asset_id: AssetId) -> Option<AssetFiles> {
        let handles: Vec<ProjectActorHandle> = self.registry().values().cloned().collect();
        for handle in handles {
            let Ok(view) = handle.view().await else {
                continue;
            };
            if let Some(asset) = view
                .project
                .assets
                .iter()
                .find(|asset| asset.id == asset_id)
            {
                return Some(AssetFiles {
                    original: PathBuf::from(&asset.original_path),
                    kind: asset.kind,
                    cache_dir: asset.fingerprint.as_ref().map(|fingerprint| {
                        media_paths::media_cache_dir(handle.path(), fingerprint)
                    }),
                });
            }
        }
        None
    }
}

/// The asset `asset_id` of `project`.
fn find_asset(project: &se_domain::Project, asset_id: AssetId) -> Result<&Asset, AppError> {
    project
        .assets
        .iter()
        .find(|asset| asset.id == asset_id)
        .ok_or(AppError::AssetNotFound {
            project_id: project.id,
            asset_id,
        })
}

#[async_trait]
impl ProjectObserver for MediaService {
    async fn project_opened(&self, handle: &ProjectActorHandle) {
        self.registry().insert(handle.project_id(), handle.clone());
        self.check_media_files(handle).await;
    }

    async fn project_closing(&self, project_id: ProjectId) {
        self.registry().remove(&project_id);
        self.deps.jobs.cancel_project(project_id).await;
    }
}
