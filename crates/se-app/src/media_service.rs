//! [`MediaService`]: the media use cases behind the protocol (import, relink, remove, decodability
//! reports, jobs, mezzanines and proxies, analyses, settings, the project cache, frames) and the
//! lifecycle hooks that check missing files when a project opens and stop its jobs when it closes
//! (ADR-0018, ADR-0019).
//!
//! Like [`crate::ProjectService`], it never mutates a document itself: every change is a
//! transaction sent to the project's actor. User actions (import, relink, remove) are user
//! transactions (undoable); facts the app observed (missing files, decodability) are
//! [`se_domain::Actor::System`] transactions, journaled but never undone.

mod cache;
mod derived;
mod frames;
mod import;
mod job_work;
pub mod media_paths;
mod opening;
mod relink;
mod scheduling;
pub mod settings;
mod silences;

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

use async_trait::async_trait;
use se_domain::{
    Actor, Asset, AssetId, AssetKind, DerivedFile, DerivedStatus, JobId, Operation, ProjectId,
    Transaction,
};
use se_protocol::JobSnapshot;

pub use self::frames::{MAX_FRAME_WIDTH, MIN_FRAME_WIDTH};
use crate::{
    AppError,
    jobs::JobQueue,
    new_ids::NewIds,
    ports::{
        AppLibraryRepository, Clock, FrameExtractor, IdGenerator, MediaFiles, MediaJobRunner,
        MediaProber, MediaTools,
    },
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
    /// Mezzanines or proxies recorded as pending before their jobs start (system).
    pub const QUEUE_DERIVED_MEDIA: &str = "Queue derived media";
    /// A mezzanine or proxy job finished or failed (system).
    pub const RECORD_DERIVED_MEDIA: &str = "Record derived media";
    /// The summary of a finished analysis (system).
    pub const RECORD_ANALYSIS: &str = "Record media analysis";
}

/// The ports and services the media use cases work with.
#[derive(Clone)]
pub struct MediaServiceDeps {
    /// Reads streams of media files.
    pub prober: Arc<dyn MediaProber>,
    /// Existence and fingerprints of files.
    pub files: Arc<dyn MediaFiles>,
    /// Produces thumbnails, peaks, mezzanines, proxies and analyses.
    pub runner: Arc<dyn MediaJobRunner>,
    /// Extracts single frames.
    pub frames: Arc<dyn FrameExtractor>,
    /// Where ffmpeg is looked for, and what it can do.
    pub tools: Arc<dyn MediaTools>,
    /// Runs the jobs.
    pub jobs: JobQueue,
    /// Media settings and decoder capabilities (`library.db`).
    pub library: Arc<dyn AppLibraryRepository>,
    /// New asset and transaction ids.
    pub ids: Arc<dyn IdGenerator>,
    /// Timestamps of stored analyses.
    pub clock: Arc<dyn Clock>,
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
    /// Its mezzanine, once ready.
    pub mezzanine: Option<PathBuf>,
    /// Its proxy, once ready.
    pub proxy: Option<PathBuf>,
}

/// Told about every import, after its assets were committed and their jobs queued (automatic
/// transcription, prompt 21). Must not call back into the media service.
#[async_trait]
pub trait ImportListener: Send + Sync {
    /// `assets` were just imported into the project of `handle`.
    async fn assets_imported(&self, handle: &ProjectActorHandle, assets: &[AssetId]);
}

/// The media use cases.
pub struct MediaService {
    deps: MediaServiceDeps,
    /// Open projects, as told by [`ProjectObserver`] calls.
    open: Mutex<HashMap<ProjectId, ProjectActorHandle>>,
    /// Told about imports.
    import_listeners: Mutex<Vec<Arc<dyn ImportListener>>>,
}

impl MediaService {
    /// A service with no project open.
    #[must_use]
    pub fn new(deps: MediaServiceDeps) -> Self {
        Self {
            deps,
            open: Mutex::new(HashMap::new()),
            import_listeners: Mutex::new(Vec::new()),
        }
    }

    /// Tells `listener` about every later import.
    pub fn add_import_listener(&self, listener: Arc<dyn ImportListener>) {
        self.import_listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(listener);
    }

    /// The import listeners, copied out of their lock.
    fn import_listeners(&self) -> Vec<Arc<dyn ImportListener>> {
        self.import_listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
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
                let ready = |file: &Option<DerivedFile>| {
                    file.as_ref()
                        .filter(|file| file.status == DerivedStatus::Ready)
                        .and_then(|file| {
                            media_paths::resolve_relative(handle.path(), &file.relative_path)
                        })
                };
                return Some(AssetFiles {
                    original: PathBuf::from(&asset.original_path),
                    kind: asset.kind,
                    cache_dir: asset.fingerprint.as_ref().map(|fingerprint| {
                        media_paths::media_cache_dir(handle.path(), fingerprint)
                    }),
                    mezzanine: ready(&asset.derived.mezzanine),
                    proxy: ready(&asset.derived.proxy),
                });
            }
        }
        None
    }
}

/// Commits `ops` as one transaction of `actor` through `handle`.
async fn commit_transaction(
    handle: &ProjectActorHandle,
    ids: &dyn IdGenerator,
    actor: Actor,
    label: &str,
    ops: Vec<Operation>,
) -> Result<(), AppError> {
    let transaction = Transaction {
        id: NewIds(ids).transaction()?,
        actor,
        label: label.to_owned(),
        ops,
    };
    handle.apply(transaction).await.map(drop)
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
