//! [`ExportRegistry`]: the exports of this session, in memory. It publishes every change
//! (`exportUpdated`), routes the renderer's uploads to the running export's job, and cancels the
//! exports of a project that closes (a [`ProjectObserver`], built before the project service).

use std::{
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

use async_trait::async_trait;
use se_domain::{AssetId, ExportId, JobId, ProjectId};
use se_protocol::{CoreEvent, ExportInfo, ExportState};
use tokio::sync::mpsc;

use super::{job::ExportMessage, scene::ExportMediaFiles};
use crate::{
    AppError, JobQueue, ports::EventSink, project_actor::ProjectActorHandle,
    project_service::ProjectObserver,
};

/// Largest export picture kept for the list, in bytes.
pub const MAX_THUMBNAIL_BYTES: usize = 2 * 1024 * 1024;

/// One export of the session.
struct ExportEntry {
    info: ExportInfo,
    media: Vec<(AssetId, ExportMediaFiles)>,
    inbox: Option<mpsc::Sender<ExportMessage>>,
    thumbnail: Option<Arc<Vec<u8>>>,
}

/// Which file of an asset the renderer reads for an export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportMediaVariant {
    /// The original file.
    Original,
    /// Its mezzanine (when it was ready at the start).
    Mezzanine,
}

/// The exports of this session.
pub struct ExportRegistry {
    entries: Mutex<Vec<ExportEntry>>,
    jobs: JobQueue,
    events: Arc<dyn EventSink>,
}

impl ExportRegistry {
    /// An empty registry cancelling jobs of `jobs` and publishing to `events`.
    #[must_use]
    pub fn new(jobs: JobQueue, events: Arc<dyn EventSink>) -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
            jobs,
            events,
        }
    }

    fn lock(&self) -> MutexGuard<'_, Vec<ExportEntry>> {
        // Entries hold no invariant a panic could break halfway: poisoning is ignored.
        self.entries.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(super) fn jobs(&self) -> &JobQueue {
        &self.jobs
    }

    /// Records a new export (its job not queued yet) and publishes it.
    pub(super) fn insert(&self, info: ExportInfo, media: Vec<(AssetId, ExportMediaFiles)>) {
        self.lock().push(ExportEntry {
            info: info.clone(),
            media,
            inbox: None,
            thumbnail: None,
        });
        self.events
            .publish(CoreEvent::ExportUpdated { export: info });
    }

    /// Forgets an export whose job could not be queued.
    pub(super) fn remove(&self, export_id: ExportId) {
        self.lock().retain(|entry| entry.info.id != export_id);
    }

    /// Sets the job of an export.
    pub(super) fn set_job(&self, export_id: ExportId, job_id: JobId) -> Option<ExportInfo> {
        self.change(export_id, |info| info.job_id = Some(job_id))
    }

    /// One export.
    ///
    /// # Errors
    /// [`AppError::ExportNotFound`].
    pub fn info(&self, export_id: ExportId) -> Result<ExportInfo, AppError> {
        self.lock()
            .iter()
            .find(|entry| entry.info.id == export_id)
            .map(|entry| entry.info.clone())
            .ok_or(AppError::ExportNotFound { export_id })
    }

    /// Every export of the session, oldest first.
    #[must_use]
    pub fn list(&self) -> Vec<ExportInfo> {
        self.lock().iter().map(|entry| entry.info.clone()).collect()
    }

    /// Moves an export to `state` unless it already ended, and publishes it.
    pub(super) fn set_state(&self, export_id: ExportId, state: ExportState) -> Option<ExportInfo> {
        self.change(export_id, |info| {
            if !is_finished(&info.state) {
                info.state = state;
            }
        })
    }

    /// Applies `change` to an export and publishes it when something changed.
    fn change(
        &self,
        export_id: ExportId,
        change: impl FnOnce(&mut ExportInfo),
    ) -> Option<ExportInfo> {
        let changed = {
            let mut entries = self.lock();
            let entry = entries
                .iter_mut()
                .find(|entry| entry.info.id == export_id)?;
            let before = entry.info.clone();
            change(&mut entry.info);
            if is_finished(&entry.info.state) {
                entry.inbox = None;
            }
            (entry.info != before).then(|| entry.info.clone())
        };
        if let Some(info) = &changed {
            self.events.publish(CoreEvent::ExportUpdated {
                export: info.clone(),
            });
        }
        changed
    }

    /// Cancels an export that has not reached its finalization (nothing happens otherwise): its
    /// job stops (partial files removed) and the renderer is told to stop drawing it.
    ///
    /// # Errors
    /// [`AppError::ExportNotFound`].
    pub fn cancel(&self, export_id: ExportId) -> Result<(), AppError> {
        let info = self.info(export_id)?;
        if !matches!(
            info.state,
            ExportState::Queued | ExportState::Rendering { .. }
        ) {
            return Ok(());
        }
        self.mark_cancelled(export_id);
        // Without a job yet, the job sees the cancellation when it starts. The job may also be
        // gone already (its project's jobs stopped first): nothing left to stop then.
        if let Some(job_id) = info.job_id
            && let Err(error) = self.jobs.cancel(job_id)
        {
            tracing::debug!(%export_id, %error, "the export's job had already stopped");
        }
        Ok(())
    }

    /// Marks an unfinished export cancelled and tells the renderer to stop drawing it.
    pub(super) fn mark_cancelled(&self, export_id: ExportId) {
        if self.set_state(export_id, ExportState::Cancelled).is_some() {
            self.events
                .publish(CoreEvent::ExportRenderCancelled { export_id });
        }
    }

    /// Forgets the exports that ended (the list's « Effacer »).
    pub fn clear_finished(&self) {
        self.lock().retain(|entry| !is_finished(&entry.info.state));
        self.events.publish(CoreEvent::ExportsCleared);
    }

    /// Lets the running job of an export receive the renderer's uploads.
    pub(super) fn open_inbox(&self, export_id: ExportId, sender: mpsc::Sender<ExportMessage>) {
        if let Some(entry) = self
            .lock()
            .iter_mut()
            .find(|entry| entry.info.id == export_id)
        {
            entry.inbox = Some(sender);
        }
    }

    /// Stops routing uploads to an export's job.
    pub(super) fn close_inbox(&self, export_id: ExportId) {
        if let Some(entry) = self
            .lock()
            .iter_mut()
            .find(|entry| entry.info.id == export_id)
        {
            entry.inbox = None;
        }
    }

    /// Where uploads of a rendering export go.
    ///
    /// # Errors
    /// [`AppError::ExportNotFound`], [`AppError::ExportNotRendering`].
    pub(super) fn inbox(
        &self,
        export_id: ExportId,
    ) -> Result<mpsc::Sender<ExportMessage>, AppError> {
        let entries = self.lock();
        let entry = entries
            .iter()
            .find(|entry| entry.info.id == export_id)
            .ok_or(AppError::ExportNotFound { export_id })?;
        entry
            .inbox
            .clone()
            .ok_or(AppError::ExportNotRendering { export_id })
    }

    /// Keeps a picture of an export for the list (JPEG, at most [`MAX_THUMBNAIL_BYTES`]).
    ///
    /// # Errors
    /// [`AppError::ExportNotFound`]; [`AppError::InvalidExport`] for anything but a JPEG of
    /// that size.
    pub fn set_thumbnail(&self, export_id: ExportId, jpeg: Vec<u8>) -> Result<(), AppError> {
        if jpeg.len() > MAX_THUMBNAIL_BYTES || !jpeg.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Err(AppError::InvalidExport {
                reason: format!(
                    "an export picture must be a JPEG of at most {MAX_THUMBNAIL_BYTES} bytes"
                ),
            });
        }
        {
            let mut entries = self.lock();
            let entry = entries
                .iter_mut()
                .find(|entry| entry.info.id == export_id)
                .ok_or(AppError::ExportNotFound { export_id })?;
            entry.thumbnail = Some(Arc::new(jpeg));
        }
        self.change(export_id, |info| info.has_thumbnail = true);
        Ok(())
    }

    /// The picture of an export, if the renderer sent one.
    #[must_use]
    pub fn thumbnail(&self, export_id: ExportId) -> Option<Arc<Vec<u8>>> {
        self.lock()
            .iter()
            .find(|entry| entry.info.id == export_id)
            .and_then(|entry| entry.thumbnail.clone())
    }

    /// The file of `asset_id` an unfinished export reads, as snapshotted at its start.
    #[must_use]
    pub fn media_file(
        &self,
        export_id: ExportId,
        asset_id: AssetId,
        variant: ExportMediaVariant,
    ) -> Option<PathBuf> {
        let entries = self.lock();
        let entry = entries
            .iter()
            .find(|entry| entry.info.id == export_id && !is_finished(&entry.info.state))?;
        let (_, files) = entry.media.iter().find(|(id, _)| *id == asset_id)?;
        match variant {
            ExportMediaVariant::Original => Some(files.original.clone()),
            ExportMediaVariant::Mezzanine => files.mezzanine.clone(),
        }
    }

    /// Whether an export ended (or no longer exists).
    pub(super) fn is_over(&self, export_id: ExportId) -> bool {
        self.lock()
            .iter()
            .find(|entry| entry.info.id == export_id)
            .is_none_or(|entry| is_finished(&entry.info.state))
    }

    /// Whether an export is queued, rendering or finalizing.
    #[must_use]
    pub fn has_active(&self) -> bool {
        self.lock()
            .iter()
            .any(|entry| !is_finished(&entry.info.state))
    }
}

#[async_trait]
impl ProjectObserver for ExportRegistry {
    async fn project_opened(&self, _handle: &ProjectActorHandle) {}

    async fn project_closing(&self, project_id: ProjectId) {
        let exports: Vec<ExportId> = self
            .lock()
            .iter()
            .filter(|entry| entry.info.project_id == project_id)
            .map(|entry| entry.info.id)
            .collect();
        for export_id in exports {
            // The export exists: it was just listed.
            let _ = self.cancel(export_id);
        }
    }
}

/// Whether an export ended (completed, failed or cancelled).
#[must_use]
pub const fn is_finished(state: &ExportState) -> bool {
    matches!(
        state,
        ExportState::Completed { .. } | ExportState::Failed { .. } | ExportState::Cancelled
    )
}
