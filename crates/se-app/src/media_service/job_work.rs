//! Media jobs as queue work: run a spec with the runner, then record the outcome in the project
//! (a derived file's state, an analysis and its summary). Recording happens inside the job, so a
//! failure to record is a job failure (retried once).

use std::sync::Arc;

use async_trait::async_trait;
use se_domain::{
    Actor, AnalysisResult, AssetId, AssetPatch, Assign, DerivedFile, DerivedStatus, Operation,
    UpdateAsset,
};
use se_protocol::DerivedKind;

use super::{commit_transaction, labels};
use crate::{
    jobs::{JobAttempt, JobWork},
    ports::{
        Clock, IdGenerator, MediaJobOutput, MediaJobRunner, MediaJobSpec, ProgressSink,
        StoredAnalysis,
    },
    project_actor::ProjectActorHandle,
};

/// Records what a job produced, or that it failed for good.
#[async_trait]
pub(super) trait OutcomeRecorder: Send + Sync {
    /// The job succeeded with `output`. An error makes the job retry (or fail).
    async fn succeeded(&self, output: MediaJobOutput) -> Result<(), String>;

    /// The last attempt failed with `message`.
    async fn failed(&self, message: &str);
}

/// A media job: the runner's work, then the recorder's.
pub(super) struct MediaJobWork {
    /// Runs the spec.
    pub runner: Arc<dyn MediaJobRunner>,
    /// What to run.
    pub spec: MediaJobSpec,
    /// What to record afterwards; `None` for cache files (thumbnails, peaks).
    pub recorder: Option<Arc<dyn OutcomeRecorder>>,
}

#[async_trait]
impl JobWork for MediaJobWork {
    async fn run(&self, progress: &dyn ProgressSink, attempt: JobAttempt) -> Result<(), String> {
        match self.runner.run(&self.spec, progress).await {
            Ok(output) => match &self.recorder {
                Some(recorder) => recorder.succeeded(output).await,
                None => Ok(()),
            },
            Err(error) => {
                let message = error.to_string();
                if attempt.last
                    && let Some(recorder) = &self.recorder
                {
                    recorder.failed(&message).await;
                }
                Err(message)
            }
        }
    }
}

/// Marks a mezzanine or proxy ready or failed, if the asset still waits for this very file
/// (it may have been removed, relinked or regenerated meanwhile).
pub(super) struct DerivedRecorder {
    pub handle: ProjectActorHandle,
    pub ids: Arc<dyn IdGenerator>,
    pub asset_id: AssetId,
    pub derived: DerivedKind,
    pub relative_path: String,
}

impl DerivedRecorder {
    async fn record(&self, status: DerivedStatus) {
        let Ok(view) = self.handle.view().await else {
            return;
        };
        let Some(asset) = view
            .project
            .assets
            .iter()
            .find(|asset| asset.id == self.asset_id)
        else {
            return;
        };
        let current = match self.derived {
            DerivedKind::Mezzanine => &asset.derived.mezzanine,
            DerivedKind::Proxy => &asset.derived.proxy,
        };
        let waiting = current.as_ref().is_some_and(|file| {
            file.relative_path == self.relative_path && file.status == DerivedStatus::Pending
        });
        if !waiting {
            return;
        }
        let file = Assign {
            value: Some(DerivedFile {
                relative_path: self.relative_path.clone(),
                status,
            }),
        };
        let patch = match self.derived {
            DerivedKind::Mezzanine => AssetPatch {
                mezzanine: Some(file),
                ..AssetPatch::default()
            },
            DerivedKind::Proxy => AssetPatch {
                proxy: Some(file),
                ..AssetPatch::default()
            },
        };
        let operation = UpdateAsset {
            asset: self.asset_id,
            patch,
        };
        let committed = commit_transaction(
            &self.handle,
            self.ids.as_ref(),
            Actor::System,
            labels::RECORD_DERIVED_MEDIA,
            vec![operation.into()],
        )
        .await;
        if let Err(error) = committed {
            tracing::warn!(%error, asset = %self.asset_id, "recording a derived file failed");
        }
    }
}

#[async_trait]
impl OutcomeRecorder for DerivedRecorder {
    async fn succeeded(&self, _output: MediaJobOutput) -> Result<(), String> {
        self.record(DerivedStatus::Ready).await;
        Ok(())
    }

    async fn failed(&self, message: &str) {
        self.record(DerivedStatus::Failed {
            message: message.to_owned(),
        })
        .await;
    }
}

/// Stores an analysis in the project database and its summary in the asset (when the asset
/// still holds the analysed content).
pub(super) struct AnalysisRecorder {
    pub handle: ProjectActorHandle,
    pub ids: Arc<dyn IdGenerator>,
    pub clock: Arc<dyn Clock>,
    pub asset_id: AssetId,
    pub content_key: String,
}

impl AnalysisRecorder {
    /// Puts `result`'s summary in the asset (a system transaction), unless it is already there or
    /// the asset changed content or is gone. Failures are logged.
    pub(super) async fn record_summary(&self, result: &AnalysisResult) {
        let Some(operation) = self.summary_update(result).await else {
            return;
        };
        let committed = commit_transaction(
            &self.handle,
            self.ids.as_ref(),
            Actor::System,
            labels::RECORD_ANALYSIS,
            vec![operation],
        )
        .await;
        if let Err(error) = committed {
            tracing::warn!(%error, asset = %self.asset_id, "recording an analysis summary failed");
        }
    }

    /// The operation putting `result`'s summary in the asset, unless it is already there or the
    /// asset changed content or is gone.
    async fn summary_update(&self, result: &AnalysisResult) -> Option<Operation> {
        let view = self.handle.view().await.ok()?;
        let asset = view
            .project
            .assets
            .iter()
            .find(|asset| asset.id == self.asset_id)?;
        let same_content = asset
            .fingerprint
            .as_ref()
            .is_some_and(|fingerprint| fingerprint.cache_key() == self.content_key);
        if !same_content {
            return None;
        }
        let patch = match result {
            AnalysisResult::AudioAnalysis(analysis) => {
                let summary = Some(analysis.summary());
                (asset.analysis.audio != summary).then(|| AssetPatch {
                    audio_analysis: Some(Assign { value: summary }),
                    ..AssetPatch::default()
                })?
            }
            AnalysisResult::SceneDetection(detection) => {
                let summary = Some(detection.summary());
                (asset.analysis.scenes != summary).then(|| AssetPatch {
                    scene_detection: Some(Assign { value: summary }),
                    ..AssetPatch::default()
                })?
            }
            // Faces are read by automatic reframing only: the asset keeps no summary of them.
            AnalysisResult::SubjectDetection(_) => return None,
        };
        Some(
            UpdateAsset {
                asset: self.asset_id,
                patch,
            }
            .into(),
        )
    }
}

#[async_trait]
impl OutcomeRecorder for AnalysisRecorder {
    async fn succeeded(&self, output: MediaJobOutput) -> Result<(), String> {
        let MediaJobOutput::Analysis(result) = output else {
            return Err("the analysis produced no result: try again".to_owned());
        };
        let stored = StoredAnalysis {
            asset_id: self.asset_id,
            content_key: self.content_key.clone(),
            created_at: self.clock.now_unix_millis(),
            result,
        };
        self.handle
            .analyses()
            .save_analysis(&stored)
            .await
            .map_err(|error| error.to_string())?;
        self.record_summary(&stored.result).await;
        Ok(())
    }

    async fn failed(&self, _message: &str) {}
}
