//! Where the transcriptions of assets stand, and transcribing on behalf of an AI agent: start what
//! is missing, then wait a while for it (ADR-0034). Waiting reads the shared job queue; nothing
//! here changes a document.

use std::time::Duration;

use se_domain::{Asset, AssetId, ProjectId};
use se_protocol::{JobKind, JobSnapshot, JobState};

use super::{TranscriptionService, reading::current_transcript, transcribe::has_sound};
use crate::{AppError, project_actor::ProjectActorHandle};

/// How often [`TranscriptionService::wait_for_transcriptions`] looks at the job queue.
const POLL_INTERVAL: Duration = Duration::from_millis(250);

/// Where one asset's transcription stands.
#[derive(Debug, Clone, PartialEq)]
pub enum TranscriptionState {
    /// Its current content has a transcript of `word_count` words.
    Transcribed {
        /// Words of the transcript.
        word_count: usize,
    },
    /// Waiting for the transcription slot.
    Queued,
    /// Being transcribed.
    Running {
        /// Fraction done in `[0, 1]`, when known.
        progress: Option<f64>,
    },
    /// Its last transcription of this session failed.
    Failed {
        /// Why (whisper.cpp's or ffmpeg's message).
        message: String,
    },
    /// Its last transcription of this session was cancelled (by the user, or the project closed).
    Cancelled,
    /// It cannot be transcribed now.
    Unavailable {
        /// Why.
        reason: UnavailableReason,
    },
    /// No transcript and no transcription asked for in this session.
    NotStarted,
}

impl TranscriptionState {
    /// Whether a transcription is queued or running.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Queued | Self::Running { .. })
    }

    /// Whether a transcription could be started now (none ran, or the last one did not succeed).
    #[must_use]
    pub const fn can_start(&self) -> bool {
        matches!(
            self,
            Self::NotStarted | Self::Failed { .. } | Self::Cancelled
        )
    }
}

/// Why an asset cannot be transcribed now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnavailableReason {
    /// Its original file is missing: the user must relink it.
    MediaMissing,
    /// It has no sound (an image, or a video without audio stream).
    NoSound,
    /// It is still being imported (no content fingerprint yet): try again in a moment.
    NotReady,
}

/// An asset and where its transcription stands.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetTranscription {
    /// The asset.
    pub asset_id: AssetId,
    /// Where it stands.
    pub state: TranscriptionState,
}

/// What [`TranscriptionService::transcribe_and_wait`] did.
#[derive(Debug, Clone, PartialEq)]
pub struct TranscriptionRound {
    /// The assets whose transcription this call queued.
    pub started: Vec<AssetId>,
    /// Where every asset asked about stands after the wait, in the order asked.
    pub assets: Vec<AssetTranscription>,
}

impl TranscriptionRound {
    /// Whether no asset is still queued or running.
    #[must_use]
    pub fn finished(&self) -> bool {
        !self.assets.iter().any(|asset| asset.state.is_active())
    }
}

impl TranscriptionService {
    /// Where the transcription of each of `asset_ids` stands, in the order given.
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`], [`AppError::AssetNotFound`], [`AppError::Storage`].
    pub async fn transcription_states(
        &self,
        project_id: ProjectId,
        asset_ids: &[AssetId],
    ) -> Result<Vec<AssetTranscription>, AppError> {
        let handle = self.handle(project_id)?;
        let project = handle.view().await?.project;
        let jobs = self.deps().jobs.list(project_id);
        let mut states = Vec::with_capacity(asset_ids.len());
        for asset_id in asset_ids {
            let asset = project
                .assets
                .iter()
                .find(|asset| asset.id == *asset_id)
                .ok_or(AppError::AssetNotFound {
                    project_id,
                    asset_id: *asset_id,
                })?;
            states.push(AssetTranscription {
                asset_id: *asset_id,
                state: state_of(&handle, asset, last_job(&jobs, *asset_id)).await?,
            });
        }
        Ok(states)
    }

    /// Waits until none of `asset_ids` is queued or running, or until `timeout` passed, then
    /// tells where each stands.
    ///
    /// # Errors
    /// Those of [`TranscriptionService::transcription_states`].
    pub async fn wait_for_transcriptions(
        &self,
        project_id: ProjectId,
        asset_ids: &[AssetId],
        timeout: Duration,
    ) -> Result<Vec<AssetTranscription>, AppError> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let states = self.transcription_states(project_id, asset_ids).await?;
            let busy = states.iter().any(|asset| asset.state.is_active());
            if !busy || tokio::time::Instant::now() >= deadline {
                return Ok(states);
            }
            tokio::time::sleep_until(deadline.min(tokio::time::Instant::now() + POLL_INTERVAL))
                .await;
        }
    }

    /// Queues the transcription of those of `asset_ids` that have none and none running (failed
    /// or cancelled ones are tried again), then waits up to `timeout` for every one of them
    /// (an agent asked: ADR-0034). Already transcribed assets cost nothing.
    ///
    /// # Errors
    /// [`AppError::NoTranscriptionModel`] or [`AppError::TranscriptionUnavailable`] when one must
    /// be started and cannot; those of [`TranscriptionService::transcription_states`].
    pub async fn transcribe_and_wait(
        &self,
        project_id: ProjectId,
        asset_ids: &[AssetId],
        timeout: Duration,
    ) -> Result<TranscriptionRound, AppError> {
        let before = self.transcription_states(project_id, asset_ids).await?;
        let to_start: Vec<AssetId> = before
            .iter()
            .filter(|asset| asset.state.can_start())
            .map(|asset| asset.asset_id)
            .collect();
        if !to_start.is_empty() {
            self.transcribe_assets(project_id, &to_start).await?;
        }
        let assets = self
            .wait_for_transcriptions(project_id, asset_ids, timeout)
            .await?;
        Ok(TranscriptionRound {
            started: to_start,
            assets,
        })
    }
}

/// The most recent transcription job of `asset_id` (the queue lists oldest first).
fn last_job(jobs: &[JobSnapshot], asset_id: AssetId) -> Option<&JobSnapshot> {
    jobs.iter()
        .rev()
        .find(|job| job.kind == JobKind::Transcription && job.asset_id == Some(asset_id))
}

/// Where `asset`'s transcription stands: an active job first, then a current transcript, then
/// the last job's failure, then what keeps it from running.
async fn state_of(
    handle: &ProjectActorHandle,
    asset: &Asset,
    job: Option<&JobSnapshot>,
) -> Result<TranscriptionState, AppError> {
    match job.map(|job| &job.state) {
        Some(JobState::Queued) => return Ok(TranscriptionState::Queued),
        Some(JobState::Running { progress }) => {
            return Ok(TranscriptionState::Running {
                progress: *progress,
            });
        }
        _ => {}
    }
    if let Some(fingerprint) = &asset.fingerprint {
        let transcript = current_transcript(handle, asset.id, &fingerprint.cache_key()).await?;
        if let Some(transcript) = transcript {
            return Ok(TranscriptionState::Transcribed {
                word_count: transcript.words.len(),
            });
        }
    }
    Ok(match job.map(|job| &job.state) {
        Some(JobState::Failed { message }) => TranscriptionState::Failed {
            message: message.clone(),
        },
        Some(JobState::Cancelled) => TranscriptionState::Cancelled,
        _ => blocker(asset).map_or(TranscriptionState::NotStarted, |reason| {
            TranscriptionState::Unavailable { reason }
        }),
    })
}

/// What keeps `asset` from being transcribed now, if anything.
fn blocker(asset: &Asset) -> Option<UnavailableReason> {
    if !has_sound(asset) {
        Some(UnavailableReason::NoSound)
    } else if asset.missing {
        Some(UnavailableReason::MediaMissing)
    } else if asset.fingerprint.is_none() {
        Some(UnavailableReason::NotReady)
    } else {
        None
    }
}
