//! The transcription job: extract the speech audio, transcribe it, store the transcript, then put
//! its summary in the asset (a system transaction) when the asset still holds that content.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex, PoisonError},
};

use async_trait::async_trait;
use se_domain::{
    Actor, AssetId, AssetPatch, Assign, Operation, Transaction, TranscriptSummary, UpdateAsset,
};
use se_protocol::TranscriptionBackend;

use super::labels;
use crate::{
    AppError,
    jobs::{JobAttempt, JobWork},
    new_ids::NewIds,
    ports::{
        Clock, IdGenerator, ProgressSink, SpeechAudioExtractor, StoredTranscript, Transcriber,
        TranscriptionRequest,
    },
    project_actor::ProjectActorHandle,
};

/// Share of the job's progress taken by the audio extraction.
const EXTRACTION_SHARE: f64 = 0.05;

/// What a transcription job needs, decided when it was queued.
pub(super) struct TranscriptionJob {
    pub extractor: Arc<dyn SpeechAudioExtractor>,
    pub transcriber: Arc<dyn Transcriber>,
    pub ids: Arc<dyn IdGenerator>,
    pub clock: Arc<dyn Clock>,
    /// Where the last run computed (shown in the settings).
    pub last_backend: Arc<Mutex<Option<TranscriptionBackend>>>,
    pub handle: ProjectActorHandle,
    pub asset_id: AssetId,
    pub content_key: String,
    /// The original file.
    pub source: PathBuf,
    /// The transcription to run (its `audio` lies in its `work_dir`).
    pub request: TranscriptionRequest,
}

#[async_trait]
impl JobWork for TranscriptionJob {
    async fn run(&self, progress: &dyn ProgressSink, _attempt: JobAttempt) -> Result<(), String> {
        let extraction = Scaled {
            inner: progress,
            offset: 0.0,
            share: EXTRACTION_SHARE,
        };
        self.extractor
            .extract_speech_audio(&self.source, &self.request.audio, &extraction)
            .await
            .map_err(|error| error.to_string())?;
        let transcription = Scaled {
            inner: progress,
            offset: EXTRACTION_SHARE,
            share: 1.0 - EXTRACTION_SHARE,
        };
        let output = self
            .transcriber
            .transcribe(&self.request, &transcription)
            .await
            .map_err(|error| error.to_string())?;
        *self
            .last_backend
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(output.backend);
        let stored = StoredTranscript {
            asset_id: self.asset_id,
            content_key: self.content_key.clone(),
            created_at: self.clock.now_unix_millis(),
            backend: backend_name(output.backend).to_owned(),
            transcript: output.transcript,
        };
        self.handle
            .transcripts()
            .save_transcript(&stored)
            .await
            .map_err(|error| error.to_string())?;
        let summary = stored.transcript.summary(stored.created_at);
        if let Err(error) = self.record_summary(summary).await {
            tracing::warn!(%error, asset = %self.asset_id, "recording a transcript summary failed");
        }
        Ok(())
    }
}

impl TranscriptionJob {
    /// Puts `summary` in the asset unless it changed content or is gone.
    async fn record_summary(&self, summary: TranscriptSummary) -> Result<(), AppError> {
        let view = self.handle.view().await?;
        let Some(asset) = view
            .project
            .assets
            .iter()
            .find(|asset| asset.id == self.asset_id)
        else {
            return Ok(());
        };
        let same_content = asset
            .fingerprint
            .as_ref()
            .is_some_and(|fingerprint| fingerprint.cache_key() == self.content_key);
        if !same_content {
            return Ok(());
        }
        let operation: Operation = UpdateAsset {
            asset: self.asset_id,
            patch: AssetPatch {
                transcript: Some(Assign {
                    value: Some(summary),
                }),
                ..AssetPatch::default()
            },
        }
        .into();
        let transaction = Transaction {
            id: NewIds(self.ids.as_ref()).transaction()?,
            actor: Actor::System,
            label: labels::RECORD_TRANSCRIPT.to_owned(),
            ops: vec![operation],
        };
        self.handle.apply(transaction).await.map(drop)
    }
}

/// The name stored with a transcript.
fn backend_name(backend: TranscriptionBackend) -> &'static str {
    match backend {
        TranscriptionBackend::Cuda => "cuda",
        TranscriptionBackend::Metal => "metal",
        TranscriptionBackend::Vulkan => "vulkan",
        TranscriptionBackend::Cpu => "cpu",
    }
}

/// A part of a job's progress: `offset + fraction × share`.
struct Scaled<'a> {
    inner: &'a dyn ProgressSink,
    offset: f64,
    share: f64,
}

impl ProgressSink for Scaled<'_> {
    fn report(&self, fraction: f64) {
        self.inner
            .report(self.offset + fraction.clamp(0.0, 1.0) * self.share);
    }
}
