//! Downloads of models, the voice model and the GPU pack: started only by an explicit command,
//! one task per target, progress published as `transcriptionStatusChanged` (at most once per
//! percent), cancelled by aborting the task (the `.part` file stays for a later resume).

use std::sync::atomic::{AtomicU64, Ordering};

use se_protocol::{DownloadProgress, DownloadTarget};

use super::{
    TranscriptionService,
    catalogue::{self, CatalogueFile, VOICE_MODEL},
    state::Download,
};
use crate::{
    AppError,
    ports::{DownloadError, DownloadSink, DownloadSpec},
};

impl TranscriptionService {
    /// Starts downloading `target` (and the voice model with a first transcription model). The
    /// UI asked the user first: this is the only way the app reaches the network itself.
    ///
    /// # Errors
    /// [`AppError::TranscriptionTargetNotFound`] for an unknown model or a platform without GPU
    /// pack. A download already running is left alone.
    pub fn start_download(&self, target: &DownloadTarget) -> Result<(), AppError> {
        match target {
            DownloadTarget::Model { model_id } => {
                let model = catalogue::model(model_id).ok_or_else(|| {
                    AppError::TranscriptionTargetNotFound {
                        what: format!("the model '{model_id}'"),
                    }
                })?;
                self.spawn_file_download(target, model.file);
                if !self.state().installed.has_voice_model() {
                    self.spawn_file_download(&DownloadTarget::VoiceModel, VOICE_MODEL);
                }
            }
            DownloadTarget::VoiceModel => self.spawn_file_download(target, VOICE_MODEL),
            DownloadTarget::GpuRuntime => {
                let offered = self
                    .state()
                    .environment
                    .as_ref()
                    .is_some_and(|environment| environment.gpu_runtime.is_some());
                if !offered {
                    return Err(AppError::TranscriptionTargetNotFound {
                        what: "a GPU pack for this platform".to_owned(),
                    });
                }
                self.spawn_gpu_runtime_install();
            }
        }
        self.publish_status();
        Ok(())
    }

    /// Stops the download of `target` (nothing is installed).
    ///
    /// # Errors
    /// [`AppError::TranscriptionTargetNotFound`] when none runs.
    pub fn cancel_download(&self, target: &DownloadTarget) -> Result<(), AppError> {
        let removed = self.state().downloads.remove(target);
        let Some(download) = removed else {
            return Err(AppError::TranscriptionTargetNotFound {
                what: "this download".to_owned(),
            });
        };
        if let Some(task) = download.task {
            task.abort();
        }
        self.publish_status();
        Ok(())
    }

    /// Marks `target` as starting; `false` when it already runs.
    fn begin(&self, target: &DownloadTarget, total: u64) -> bool {
        let mut state = self.state();
        let running = state
            .downloads
            .get(target)
            .is_some_and(|download| !matches!(download.progress, DownloadProgress::Failed { .. }));
        if running {
            return false;
        }
        state.downloads.insert(
            target.clone(),
            Download {
                progress: DownloadProgress::Downloading { received: 0, total },
                task: None,
            },
        );
        true
    }

    fn spawn_file_download(&self, target: &DownloadTarget, file: CatalogueFile) {
        if !self.begin(target, file.size) {
            return;
        }
        let service = self.clone();
        let spec = DownloadSpec {
            url: file.url.to_owned(),
            sha256: file.sha256.to_owned(),
            size: file.size,
            destination: self.deps().models_dir.join(file.file_name),
        };
        let task_target = target.clone();
        let task = tokio::spawn(async move {
            let sink = Progress::new(service.clone(), task_target.clone());
            let outcome = service.deps().downloader.download(&spec, &sink).await;
            service.finish(&task_target, outcome).await;
        });
        self.attach(target, task.abort_handle());
    }

    fn spawn_gpu_runtime_install(&self) {
        let target = DownloadTarget::GpuRuntime;
        let size = self
            .state()
            .environment
            .as_ref()
            .and_then(|environment| environment.gpu_runtime.as_ref())
            .map_or(0, |offer| offer.download_size);
        if !self.begin(&target, size) {
            return;
        }
        let service = self.clone();
        let task = tokio::spawn(async move {
            let sink = Progress::new(service.clone(), DownloadTarget::GpuRuntime);
            let outcome = service.deps().transcriber.install_gpu_runtime(&sink).await;
            if outcome.is_ok()
                && let Err(error) = service.refresh().await
            {
                tracing::warn!(%error, "refreshing the transcription status failed");
            }
            service.finish(&DownloadTarget::GpuRuntime, outcome).await;
        });
        self.attach(&target, task.abort_handle());
    }

    /// Keeps the task's abort handle while its download runs (not once it failed, succeeded or
    /// was cancelled: the task may end before this runs).
    fn attach(&self, target: &DownloadTarget, task: tokio::task::AbortHandle) {
        if let Some(download) = self.state().downloads.get_mut(target)
            && !matches!(download.progress, DownloadProgress::Failed { .. })
        {
            download.task = Some(task);
        }
    }

    /// Records the end of a download: gone when it succeeded, failed with its message otherwise.
    async fn finish(&self, target: &DownloadTarget, outcome: Result<(), DownloadError>) {
        match outcome {
            Ok(()) => {
                self.state().downloads.remove(target);
                self.refresh_models().await;
            }
            Err(error) => {
                tracing::warn!(%error, ?target, "a transcription download failed");
                let mut state = self.state();
                if let Some(download) = state.downloads.get_mut(target) {
                    download.progress = DownloadProgress::Failed {
                        message: error.to_string(),
                    };
                    download.task = None;
                }
            }
        }
        self.publish_status();
    }

    /// Records progress of a running download.
    fn record_progress(&self, target: &DownloadTarget, progress: DownloadProgress) {
        if let Some(download) = self.state().downloads.get_mut(target) {
            download.progress = progress;
        }
    }
}

/// Progress of one download, published at most once per percent.
struct Progress {
    service: TranscriptionService,
    target: DownloadTarget,
    /// Last percent published (`u64::MAX` before the first).
    published: AtomicU64,
}

impl Progress {
    fn new(service: TranscriptionService, target: DownloadTarget) -> Self {
        Self {
            service,
            target,
            published: AtomicU64::new(u64::MAX),
        }
    }
}

impl DownloadSink for Progress {
    fn received(&self, received: u64, total: u64) {
        self.service.record_progress(
            &self.target,
            DownloadProgress::Downloading { received, total },
        );
        let percent = received.saturating_mul(100) / total.max(1);
        if self.published.swap(percent, Ordering::Relaxed) != percent {
            self.service.publish_status();
        }
    }

    fn verifying(&self) {
        self.service
            .record_progress(&self.target, DownloadProgress::Verifying);
        self.service.publish_status();
    }
}
