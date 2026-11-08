//! Imports and removals of model files and of the GPU pack.

use std::path::Path;

use se_protocol::{DownloadTarget, TranscriptionStatus};

use super::{TranscriptionService, catalogue::VOICE_MODEL, models, preferences};
use crate::AppError;

impl TranscriptionService {
    /// Copies a ggml model into the models folder; returns its id and the new status.
    ///
    /// # Errors
    /// [`AppError::InvalidModelFile`] for a file that is not a whisper.cpp model; media errors of
    /// the copy.
    pub async fn import_model(
        &self,
        path: &Path,
    ) -> Result<(String, TranscriptionStatus), AppError> {
        let id = models::import(
            self.deps().model_files.as_ref(),
            &self.deps().models_dir,
            path,
        )
        .await?;
        self.refresh_models().await;
        self.publish_status();
        Ok((id, self.current_status()))
    }

    /// Deletes an installed model, the voice model or the GPU pack; a removed chosen model falls
    /// back to automatic.
    ///
    /// # Errors
    /// [`AppError::TranscriptionTargetNotFound`] for something not installed; IO failures.
    pub async fn remove_file(
        &self,
        target: &DownloadTarget,
    ) -> Result<TranscriptionStatus, AppError> {
        match target {
            DownloadTarget::Model { model_id } => {
                let file = {
                    let state = self.state();
                    models::installed_file(&state.installed, &self.deps().models_dir, model_id)?
                };
                self.deps().model_files.remove(&file).await?;
                let current = self.state().preferences.clone();
                let updated = models::without_model(current.clone(), model_id);
                if updated != current {
                    preferences::save(self.deps().library.as_ref(), &updated).await?;
                    self.state().preferences = updated;
                }
            }
            DownloadTarget::VoiceModel => {
                let file = self.deps().models_dir.join(VOICE_MODEL.file_name);
                self.deps().model_files.remove(&file).await?;
            }
            DownloadTarget::GpuRuntime => {
                self.deps()
                    .transcriber
                    .remove_gpu_runtime()
                    .await
                    .map_err(|error| AppError::DownloadFailed {
                        reason: error.to_string(),
                    })?;
                self.refresh().await?;
            }
        }
        self.refresh_models().await;
        self.publish_status();
        Ok(self.current_status())
    }
}
