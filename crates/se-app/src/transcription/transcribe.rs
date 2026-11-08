//! Queuing transcriptions: on request (« Transcrire ») or after an import when the preference is
//! on. What runs (program, model, voice model, GPU, language, vocabulary) is decided when the job
//! is queued.

use async_trait::async_trait;
use se_domain::{Asset, AssetId, AssetKind, JobId, ProjectId};
use se_protocol::JobKind;

use super::{TranscriptionService, job::TranscriptionJob};
use crate::{
    AppError,
    jobs::{JobPriority, JobRequest},
    media_service::{ImportListener, media_paths::media_cache_dir},
    ports::{ModelChoice, TranscriptionRequest},
    project_actor::ProjectActorHandle,
};

/// Name of the folder of a transcription's files in the asset's cache folder.
const WORK_DIR: &str = "transcription";
/// The extracted speech audio inside it.
const SPEECH_AUDIO: &str = "speech-16k.wav";

/// What every job queued now uses.
struct RunPlan {
    model: ModelChoice,
    voice_model: Option<std::path::PathBuf>,
    use_gpu: bool,
    keep_hesitations: bool,
}

impl TranscriptionService {
    /// Queues the transcription of `asset_ids` (assets already being transcribed are skipped).
    ///
    /// # Errors
    /// [`AppError::TranscriptionUnavailable`], [`AppError::NoTranscriptionModel`],
    /// [`AppError::AssetNotFound`], [`AppError::NoSpeechStream`], [`AppError::ProjectNotOpen`].
    pub async fn transcribe_assets(
        &self,
        project_id: ProjectId,
        asset_ids: &[AssetId],
    ) -> Result<Vec<JobId>, AppError> {
        let handle = self.handle(project_id)?;
        let plan = self.run_plan()?;
        let project = handle.view().await?.project;
        let mut assets = Vec::with_capacity(asset_ids.len());
        for asset_id in asset_ids {
            let asset = project
                .assets
                .iter()
                .find(|asset| asset.id == *asset_id)
                .ok_or(AppError::AssetNotFound {
                    project_id,
                    asset_id: *asset_id,
                })?;
            if !has_sound(asset) {
                return Err(AppError::NoSpeechStream {
                    asset_id: *asset_id,
                });
            }
            assets.push(asset);
        }
        let mut jobs = Vec::new();
        for asset in assets {
            if let Some(job) = self.enqueue(&handle, &project, asset, &plan, JobPriority::Normal)? {
                jobs.push(job);
            }
        }
        Ok(jobs)
    }

    /// What jobs queued now use.
    fn run_plan(&self) -> Result<RunPlan, AppError> {
        let state = self.state();
        let whisper = state
            .environment
            .as_ref()
            .map(|environment| environment.whisper.clone());
        if let Some(Err(error)) = whisper {
            return Err(AppError::TranscriptionUnavailable {
                reason: error.to_string(),
            });
        }
        let hardware = state.hardware(self.deps().cpu_threads);
        let model = state
            .installed
            .choose(
                state.preferences.model.as_deref(),
                hardware,
                &self.deps().models_dir,
            )
            .ok_or(AppError::NoTranscriptionModel)?;
        let voice_model = (state.preferences.voice_detection && state.installed.has_voice_model())
            .then(|| self.deps().models_dir.join(super::VOICE_MODEL.file_name));
        Ok(RunPlan {
            model,
            voice_model,
            use_gpu: state.preferences.use_gpu,
            keep_hesitations: state.preferences.keep_hesitations,
        })
    }

    /// Queues one asset's transcription; `None` when it cannot run now (missing original, no
    /// fingerprint yet) or one is already queued or running.
    fn enqueue(
        &self,
        handle: &ProjectActorHandle,
        project: &se_domain::Project,
        asset: &Asset,
        plan: &RunPlan,
        priority: JobPriority,
    ) -> Result<Option<JobId>, AppError> {
        let Some(fingerprint) = &asset.fingerprint else {
            return Ok(None);
        };
        let jobs = &self.deps().jobs;
        if asset.missing || jobs.has_active(project.id, asset.id, JobKind::Transcription) {
            return Ok(None);
        }
        let work_dir = media_cache_dir(handle.path(), fingerprint).join(WORK_DIR);
        let settings = &project.settings.transcription;
        let vocabulary = settings.vocabulary.trim();
        let request = TranscriptionRequest {
            audio: work_dir.join(SPEECH_AUDIO),
            work_dir,
            model: plan.model.clone(),
            voice_model: plan.voice_model.clone(),
            language: settings.language.clone(),
            vocabulary: (!vocabulary.is_empty()).then(|| vocabulary.to_owned()),
            keep_hesitations: plan.keep_hesitations,
            use_gpu: plan.use_gpu,
        };
        let work = TranscriptionJob {
            extractor: self.deps().extractor.clone(),
            transcriber: self.deps().transcriber.clone(),
            ids: self.deps().ids.clone(),
            clock: self.deps().clock.clone(),
            last_backend: self.inner.last_backend.clone(),
            handle: handle.clone(),
            asset_id: asset.id,
            content_key: fingerprint.cache_key(),
            source: std::path::PathBuf::from(&asset.original_path),
            request,
        };
        let job = jobs.enqueue(JobRequest {
            project_id: project.id,
            asset_id: Some(asset.id),
            kind: JobKind::Transcription,
            priority,
            work: std::sync::Arc::new(work),
        })?;
        Ok(Some(job))
    }
}

/// Whether an asset has sound to transcribe.
pub(super) fn has_sound(asset: &Asset) -> bool {
    asset.kind != AssetKind::Image
        && asset
            .media_info
            .as_ref()
            .is_some_and(|info| !info.audio_streams.is_empty())
}

#[async_trait]
impl ImportListener for TranscriptionService {
    /// Queues imported media with sound when automatic transcription is on and possible (a
    /// missing model or program is not an error here: the user did not ask).
    async fn assets_imported(&self, handle: &ProjectActorHandle, assets: &[AssetId]) {
        if !self.state().preferences.auto_transcribe {
            return;
        }
        let Ok(plan) = self.run_plan() else {
            tracing::info!("automatic transcription skipped: no model or no whisper.cpp");
            return;
        };
        let Ok(view) = handle.view().await else {
            return;
        };
        let project = &view.project;
        for asset in project
            .assets
            .iter()
            .filter(|asset| assets.contains(&asset.id))
        {
            if !has_sound(asset) {
                continue;
            }
            if let Err(error) = self.enqueue(handle, project, asset, &plan, JobPriority::Low) {
                tracing::warn!(%error, asset = %asset.id, "queuing an automatic transcription failed");
            }
        }
    }
}
