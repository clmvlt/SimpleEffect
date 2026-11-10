//! [`ExportService`]: final video exports (prompt 27, ADR-0038).
//!
//! The renderer draws exports with `@se/engine` (golden rule 7: the export is the preview); the
//! core orchestrates them and writes their files. `start` snapshots the sequence
//! ([`scene::export_scene`]) and queues an [`job::ExportJob`] (`JobKind::Export`, one at a time,
//! never retried). When it runs, the job opens the output (se-media: partial files, ffmpeg),
//! publishes `exportRenderRequested` with the snapshot, writes what the renderer uploads
//! (`/media/exports/{id}/…`) and completes the file, checked by ffprobe. Progress, cancellation
//! and failures are published as `exportUpdated`; a cancelled or failed export leaves no
//! partial file. Exports live in memory ([`ExportRegistry`]); the dialog's preferences live in
//! `library.db` ([`preferences`]).

pub mod check;
mod job;
mod paths;
pub mod preferences;
mod registry;
pub mod scene;
pub mod settings;
mod uploads;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use se_domain::{AssetId, ExportId, Flicks};
use se_protocol::{
    ExportInfo, ExportPreferences, ExportRenderRequest, ExportState, JobKind, StartExportRequest,
};

use self::{job::ExportJob, scene::ExportScene};
pub use self::{
    job::{ExportMessage, ExportTiming},
    registry::{ExportMediaVariant, ExportRegistry, MAX_THUMBNAIL_BYTES, is_finished},
};
use crate::{
    AppError, JobPriority, JobRequest, ProjectService,
    new_ids::NewIds,
    ports::{
        AppLibraryRepository, Clock, EventSink, ExportOutputPlan, ExportOutputs, IdGenerator,
        MediaProber, RendererLink,
    },
};

/// What the export service works with.
pub struct ExportServiceDeps {
    /// The exports of the session (also a project observer, built before `projects`).
    pub registry: Arc<ExportRegistry>,
    /// Open projects, snapshotted at each start.
    pub projects: Arc<ProjectService>,
    /// Where files are written (se-media).
    pub outputs: Arc<dyn ExportOutputs>,
    /// Checks finished files (ffprobe).
    pub prober: Arc<dyn MediaProber>,
    /// Whether an editor window can draw exports.
    pub renderer: Arc<dyn RendererLink>,
    /// The dialog's preferences.
    pub library: Arc<dyn AppLibraryRepository>,
    /// Render requests and cancellations for the renderer.
    pub events: Arc<dyn EventSink>,
    /// Start dates.
    pub clock: Arc<dyn Clock>,
    /// Export ids.
    pub ids: Arc<dyn IdGenerator>,
    /// The folder proposed before any export (the user's videos folder).
    pub default_directory: PathBuf,
}

/// Starts, follows and cancels exports. Cheap to clone.
#[derive(Clone)]
pub struct ExportService {
    deps: Arc<ExportServiceDeps>,
    timing: ExportTiming,
}

impl ExportService {
    /// A service with the default waits for the renderer.
    #[must_use]
    pub fn new(deps: ExportServiceDeps) -> Self {
        Self {
            deps: Arc::new(deps),
            timing: ExportTiming::default(),
        }
    }

    /// The same service with other waits (tests).
    #[must_use]
    pub const fn with_timing(mut self, timing: ExportTiming) -> Self {
        self.timing = timing;
        self
    }

    /// Snapshots the sequence and queues its export.
    ///
    /// # Errors
    /// [`AppError::InvalidExport`] for bad settings, path or span;
    /// [`AppError::ProjectNotOpen`]; [`AppError::RendererUnavailable`] without an editor window.
    pub async fn start(&self, request: StartExportRequest) -> Result<ExportInfo, AppError> {
        let invalid = |reason: String| AppError::InvalidExport { reason };
        settings::check_settings(&request.settings).map_err(invalid)?;
        let path = paths::checked_path(&request.path, &request.settings).map_err(invalid)?;
        self.refuse_busy_path(&path)?;
        let project = self.deps.projects.project(request.project_id).await?;
        let scene =
            scene::export_scene(&project, request.sequence_id, request.range).map_err(invalid)?;
        if !self.deps.renderer.is_connected() {
            return Err(AppError::RendererUnavailable {
                reason: "no SimpleEffect window is open to draw the export".to_owned(),
            });
        }
        let folder = self
            .deps
            .projects
            .project_folder(request.project_id)
            .await?;
        let media = scene::media_files_of(&project.assets, &folder);
        let export_id = NewIds(self.deps.ids.as_ref()).export()?;
        let info = self.new_info(export_id, &request, &path, &scene);
        self.deps.registry.insert(info.clone(), media);
        let job = self.job(export_id, &request, path, scene);
        let queued = self.deps.registry.jobs().enqueue(JobRequest {
            project_id: request.project_id,
            asset_id: None,
            kind: JobKind::Export,
            priority: JobPriority::High,
            work: Arc::new(job),
        });
        match queued {
            Ok(job_id) => Ok(self
                .deps
                .registry
                .set_job(export_id, job_id)
                .unwrap_or(info)),
            Err(error) => {
                self.deps.registry.remove(export_id);
                Err(error)
            }
        }
    }

    fn refuse_busy_path(&self, path: &Path) -> Result<(), AppError> {
        let busy = self
            .deps
            .registry
            .list()
            .into_iter()
            .any(|export| !is_finished(&export.state) && Path::new(&export.path) == path);
        if busy {
            return Err(AppError::InvalidExport {
                reason: format!(
                    "another export is writing {}: wait for it or choose another name",
                    path.display()
                ),
            });
        }
        Ok(())
    }

    fn new_info(
        &self,
        export_id: ExportId,
        request: &StartExportRequest,
        path: &Path,
        scene: &ExportScene,
    ) -> ExportInfo {
        ExportInfo {
            id: export_id,
            job_id: None,
            project_id: request.project_id,
            sequence_id: request.sequence_id,
            sequence_name: scene.sequence_name.clone(),
            preset_name: request.preset_name.clone(),
            path: path.to_string_lossy().into_owned(),
            settings: request.settings,
            duration: scene.span.duration(),
            created_at: u64::try_from(self.deps.clock.now_unix_millis()).unwrap_or(0),
            has_thumbnail: false,
            state: ExportState::Queued,
        }
    }

    fn job(
        &self,
        export_id: ExportId,
        request: &StartExportRequest,
        path: PathBuf,
        scene: ExportScene,
    ) -> ExportJob {
        let pipeline = settings::pipeline_of(&request.settings);
        ExportJob {
            export_id,
            plan: ExportOutputPlan {
                path,
                container: request.settings.container,
                pipeline,
                video: request.settings.video,
                audio: request.settings.audio,
                frame_rate: scene.frame_rate,
            },
            request: ExportRenderRequest {
                export_id,
                project_id: request.project_id,
                sequence_id: request.sequence_id,
                snapshot: scene.snapshot,
                settings: request.settings,
                start: scene.span.start(),
                end: scene.span.end(),
                pipeline,
            },
            registry: Arc::clone(&self.deps.registry),
            outputs: Arc::clone(&self.deps.outputs),
            prober: Arc::clone(&self.deps.prober),
            renderer: Arc::clone(&self.deps.renderer),
            events: Arc::clone(&self.deps.events),
            timing: self.timing,
        }
    }

    /// Cancels a queued or rendering export (a finalizing or ended one is left alone).
    ///
    /// # Errors
    /// [`AppError::ExportNotFound`].
    pub fn cancel(&self, export_id: ExportId) -> Result<(), AppError> {
        self.deps.registry.cancel(export_id)
    }

    /// Every export of the session, oldest first.
    #[must_use]
    pub fn list(&self) -> Vec<ExportInfo> {
        self.deps.registry.list()
    }

    /// Forgets the exports that ended.
    pub fn clear_finished(&self) {
        self.deps.registry.clear_finished();
    }

    /// Keeps a picture of an export for the list (see [`ExportRegistry::set_thumbnail`]).
    ///
    /// # Errors
    /// [`AppError::ExportNotFound`], [`AppError::InvalidExport`].
    pub fn set_thumbnail(&self, export_id: ExportId, jpeg: Vec<u8>) -> Result<(), AppError> {
        self.deps.registry.set_thumbnail(export_id, jpeg)
    }

    /// The picture of an export, if any.
    #[must_use]
    pub fn thumbnail(&self, export_id: ExportId) -> Option<Arc<Vec<u8>>> {
        self.deps.registry.thumbnail(export_id)
    }

    /// The file an unfinished export reads for an asset (`/media/exports/{id}/assets/…`).
    #[must_use]
    pub fn media_file(
        &self,
        export_id: ExportId,
        asset_id: AssetId,
        variant: ExportMediaVariant,
    ) -> Option<PathBuf> {
        self.deps.registry.media_file(export_id, asset_id, variant)
    }

    /// The dialog's preferences.
    ///
    /// # Errors
    /// [`AppError::Storage`].
    pub async fn preferences(&self) -> Result<ExportPreferences, AppError> {
        preferences::load(self.deps.library.as_ref(), &self.deps.default_directory).await
    }

    /// Stores the dialog's preferences.
    ///
    /// # Errors
    /// [`AppError::InvalidExport`] naming the faulty value; [`AppError::Storage`].
    pub async fn update_preferences(
        &self,
        preferences: &ExportPreferences,
    ) -> Result<(), AppError> {
        preferences::save(self.deps.library.as_ref(), preferences).await
    }
}

/// Duration of `[start, end)`, zero when reversed.
#[must_use]
pub fn span_duration(start: Flicks, end: Flicks) -> Flicks {
    end.checked_sub(start)
        .filter(|duration| !duration.is_negative())
        .unwrap_or(Flicks::ZERO)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reversed_spans_last_nothing() {
        let one = Flicks::one_second();
        assert_eq!(span_duration(Flicks::ZERO, one), one);
        assert_eq!(span_duration(one, Flicks::ZERO), Flicks::ZERO);
    }
}
