//! What the transcription settings show, kept in memory ([`State`]): this machine's environment,
//! the models folder, the preferences and the downloads. [`TranscriptionService::refresh`]
//! reloads it; [`TranscriptionService::status`] reads it.

use std::collections::HashMap;

use se_protocol::{
    CoreEvent, DownloadProgress, DownloadTarget, GpuRuntimeInfo, TranscriptionBackend,
    TranscriptionPreferences, TranscriptionStatus, VoiceModelInfo, WhisperInfo,
};
use tokio::task::AbortHandle;

use super::{
    TranscriptionService,
    catalogue::{self, Hardware, VOICE_MODEL},
    models::InstalledModels,
    preferences,
};
use crate::{AppError, ports::TranscriberEnvironment};

/// A download or installation in progress (or failed).
#[derive(Debug)]
pub(super) struct Download {
    pub progress: DownloadProgress,
    /// Stops the task (none once it ended).
    pub task: Option<AbortHandle>,
}

/// The in-memory state of the service.
#[derive(Debug, Default)]
pub(super) struct State {
    pub environment: Option<TranscriberEnvironment>,
    pub installed: InstalledModels,
    pub preferences: TranscriptionPreferences,
    pub downloads: HashMap<DownloadTarget, Download>,
}

impl State {
    /// The hardware the recommendation and the automatic model choice look at: a GPU counts when
    /// one was detected (usable now or once the GPU pack is installed).
    pub fn hardware(&self, cpu_threads: usize) -> Hardware {
        Hardware {
            gpu: self
                .environment
                .as_ref()
                .is_some_and(|environment| environment.gpu_name.is_some()),
            cpu_threads,
        }
    }

    fn progress_of(&self, target: &DownloadTarget) -> Option<DownloadProgress> {
        self.downloads
            .get(target)
            .map(|download| download.progress.clone())
    }

    /// Everything « Réglages → Transcription » shows.
    pub fn status(
        &self,
        cpu_threads: usize,
        models_dir: &str,
        last_backend: Option<TranscriptionBackend>,
    ) -> TranscriptionStatus {
        let recommended = catalogue::recommended_model(self.hardware(cpu_threads));
        let models = self.installed.infos(recommended, |id| {
            self.progress_of(&DownloadTarget::Model {
                model_id: id.to_owned(),
            })
        });
        let environment = self.environment.as_ref();
        let whisper = match environment.map(|environment| &environment.whisper) {
            Some(Ok(path)) => WhisperInfo {
                path: Some(path.display().to_string()),
                problem: None,
            },
            Some(Err(error)) => WhisperInfo {
                path: None,
                problem: Some(error.to_string()),
            },
            None => WhisperInfo {
                path: None,
                problem: None,
            },
        };
        let gpu_runtime = environment
            .and_then(|environment| environment.gpu_runtime.as_ref())
            .map(|offer| GpuRuntimeInfo {
                name: offer.name.clone(),
                download_size: offer.download_size,
                installed: offer.installed,
                download: self.progress_of(&DownloadTarget::GpuRuntime),
            });
        let backend = environment
            .filter(|_| self.preferences.use_gpu)
            .map_or(TranscriptionBackend::Cpu, |environment| environment.backend);
        TranscriptionStatus {
            whisper,
            gpu_name: environment.and_then(|environment| environment.gpu_name.clone()),
            backend,
            last_backend,
            gpu_runtime,
            models,
            voice_model: VoiceModelInfo {
                size: VOICE_MODEL.size,
                installed: self.installed.has_voice_model(),
                download: self.progress_of(&DownloadTarget::VoiceModel),
            },
            models_dir: models_dir.to_owned(),
            preferences: self.preferences.clone(),
        }
    }
}

impl TranscriptionService {
    /// Reloads the preferences, the environment and the models folder.
    ///
    /// # Errors
    /// [`AppError::Storage`] when `library.db` cannot be read.
    pub(super) async fn refresh(&self) -> Result<(), AppError> {
        let preferences = preferences::load(self.deps().library.as_ref()).await?;
        self.deps()
            .transcriber
            .set_whisper_dir(preferences.whisper_dir.as_ref().map(Into::into));
        let environment = self.deps().transcriber.environment().await;
        let installed = self.list_models().await;
        let mut state = self.state();
        state.preferences = preferences;
        state.environment = Some(environment);
        state.installed = installed;
        Ok(())
    }

    /// Re-reads the models folder into the state.
    pub(super) async fn refresh_models(&self) {
        let installed = self.list_models().await;
        self.state().installed = installed;
    }

    async fn list_models(&self) -> InstalledModels {
        InstalledModels {
            files: self.deps().model_files.list(&self.deps().models_dir).await,
        }
    }

    /// The status as it is now, after re-reading the models folder (files may have been added or
    /// deleted by hand).
    ///
    /// # Errors
    /// [`AppError::Storage`] when the preferences cannot be read the first time.
    pub async fn status(&self) -> Result<TranscriptionStatus, AppError> {
        if self.state().environment.is_none() {
            self.refresh().await?;
        } else {
            self.refresh_models().await;
        }
        Ok(self.current_status())
    }

    /// The status from memory (no IO).
    pub(super) fn current_status(&self) -> TranscriptionStatus {
        let last_backend = *self
            .inner
            .last_backend
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let models_dir = self.deps().models_dir.display().to_string();
        self.state()
            .status(self.deps().cpu_threads, &models_dir, last_backend)
    }

    /// Publishes `transcriptionStatusChanged` with the status from memory.
    pub(super) fn publish_status(&self) {
        let status = self.current_status();
        self.deps()
            .events
            .publish(CoreEvent::TranscriptionStatusChanged { status });
    }

    /// Saves `preferences`, applies the whisper folder, and returns the new status.
    ///
    /// # Errors
    /// [`AppError::InvalidSetting`] (relative folder, unknown model); [`AppError::Storage`].
    pub async fn update_preferences(
        &self,
        preferences: TranscriptionPreferences,
    ) -> Result<TranscriptionStatus, AppError> {
        if let Some(model) = &preferences.model {
            let known = catalogue::model(model).is_some()
                || self
                    .state()
                    .installed
                    .custom()
                    .any(|file| &file.file_name == model);
            if !known {
                return Err(AppError::InvalidSetting {
                    name: "model",
                    reason: format!("'{model}' is neither a catalogue model nor an imported one"),
                });
            }
        }
        preferences::save(self.deps().library.as_ref(), &preferences).await?;
        self.refresh().await?;
        Ok(self.current_status())
    }
}
