//! [`SmartCutService`]: the use cases of text-based editing, filler words and silence removal for
//! the user (one undoable transaction each), and the media facts AI cuts read.

use std::{collections::BTreeSet, sync::Arc};

use se_domain::{
    AnalysisKind, AnalysisResult, AssetId, ItemKind, ProjectId, SequenceId, TimeRange, TrackId,
    TrackKind,
};
use se_protocol::{
    CutSummary, FillerOccurrence, SilenceCutParams, SmartCutSettings, WordCutPurpose,
};

use super::{
    CutMedia, CutRequest, CutTarget, labels,
    resolve::filler_occurrences,
    resolve_with_tracks,
    settings::{self, THRESHOLD_RANGE},
};
use crate::{
    AppError, MediaService, ProjectService, TranscriptionService, edit_plan::find_sequence,
    ports::AppLibraryRepository,
};

/// What the smart-cut use cases work with.
#[derive(Clone)]
pub struct SmartCutServiceDeps {
    /// Open projects: documents and their actors.
    pub projects: Arc<ProjectService>,
    /// Stored audio analyses (level envelopes).
    pub media: Arc<MediaService>,
    /// Transcripts.
    pub transcription: TranscriptionService,
    /// Where the settings live.
    pub library: Arc<dyn AppLibraryRepository>,
}

/// Smart cuts. Cheap to clone.
#[derive(Clone)]
pub struct SmartCutService {
    deps: SmartCutServiceDeps,
}

/// A silence cut as the protocol asks for it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceCut {
    /// The open project.
    pub project_id: ProjectId,
    /// The sequence.
    pub sequence_id: SequenceId,
    /// Where (`None`: the whole sequence).
    pub range: Option<TimeRange>,
    /// Threshold, minimum silence, padding.
    pub params: SilenceCutParams,
    /// Sound tracks listened to (`None`: every unmuted one holding media).
    pub track_ids: Option<Vec<TrackId>>,
    /// Cut every unlocked track.
    pub all_tracks: bool,
}

impl SmartCutService {
    /// The service over `deps`.
    #[must_use]
    pub const fn new(deps: SmartCutServiceDeps) -> Self {
        Self { deps }
    }

    /// The stored settings.
    ///
    /// # Errors
    /// [`AppError::Storage`].
    pub async fn settings(&self) -> Result<SmartCutSettings, AppError> {
        settings::load(self.deps.library.as_ref()).await
    }

    /// Checks and saves `settings`; returns them as stored.
    ///
    /// # Errors
    /// [`AppError::InvalidSetting`], [`AppError::Storage`].
    pub async fn update_settings(
        &self,
        settings: &SmartCutSettings,
    ) -> Result<SmartCutSettings, AppError> {
        settings::save(self.deps.library.as_ref(), settings).await
    }

    /// The transcripts and level envelopes of the assets the sound clips of `sequence_id` play.
    ///
    /// # Errors
    /// Not open, unknown sequence, unreadable transcripts or analyses.
    pub async fn media(
        &self,
        project_id: ProjectId,
        sequence_id: SequenceId,
    ) -> Result<CutMedia, AppError> {
        let project = self.deps.projects.project(project_id).await?;
        let sequence = find_sequence(&project, sequence_id)?;
        let assets: BTreeSet<AssetId> = sequence
            .tracks
            .iter()
            .filter(|track| track.kind == TrackKind::Audio)
            .flat_map(|track| &track.items)
            .filter_map(|item| match &item.kind {
                ItemKind::MediaClip(clip) => Some(clip.asset_id),
                _ => None,
            })
            .collect();
        let mut media = CutMedia::default();
        for asset in assets {
            if let Some(transcript) = self
                .deps
                .transcription
                .transcript(project_id, asset)
                .await?
            {
                media.transcripts.insert(asset, Arc::new(transcript));
            }
            let analysis = self
                .deps
                .media
                .analysis(project_id, asset, AnalysisKind::AudioAnalysis)
                .await?;
            if let Some(AnalysisResult::AudioAnalysis(audio)) = analysis
                && let Some(levels) = audio.levels
            {
                media.levels.insert(asset, Arc::new(levels));
            }
        }
        Ok(media)
    }

    /// Cuts the words whose middle lies in `ranges` (one undo step).
    ///
    /// # Errors
    /// Not open, unknown sequence, locked tracks, the domain's refusals, storage errors.
    pub async fn remove_words(
        &self,
        project_id: ProjectId,
        sequence_id: SequenceId,
        ranges: Vec<TimeRange>,
        purpose: WordCutPurpose,
        all_tracks: bool,
    ) -> Result<CutSummary, AppError> {
        let request = CutRequest {
            sequence_id,
            target: CutTarget::Words { ranges },
            all_tracks,
            limit: None,
        };
        let label = match purpose {
            WordCutPurpose::Words => labels::REMOVE_WORDS,
            WordCutPurpose::Fillers => labels::REMOVE_FILLERS,
        };
        let media = self.media(project_id, sequence_id).await?;
        self.deps
            .projects
            .apply_cut(project_id, label, &request, &media)
            .await
    }

    /// The filler words of `range` (`None`: the whole sequence) and the media not transcribed.
    ///
    /// # Errors
    /// Not open, unknown sequence, storage errors.
    pub async fn find_fillers(
        &self,
        project_id: ProjectId,
        sequence_id: SequenceId,
        range: Option<TimeRange>,
    ) -> Result<(Vec<FillerOccurrence>, Vec<AssetId>), AppError> {
        let settings = self.settings().await?;
        let media = self.media(project_id, sequence_id).await?;
        let project = self.deps.projects.project(project_id).await?;
        let (occurrences, untranscribed) = filler_occurrences(
            &project,
            sequence_id,
            range,
            &media,
            &settings.fillers,
            settings.filler_max_duration,
        )?;
        Ok((occurrences, untranscribed.into_iter().collect()))
    }

    /// The cuts `cut` would make, changing nothing.
    ///
    /// # Errors
    /// Invalid parameters, not open, unknown sequence or tracks.
    pub async fn preview_silences(&self, cut: SilenceCut) -> Result<CutSummary, AppError> {
        check_params(&cut.params)?;
        let media = self.media(cut.project_id, cut.sequence_id).await?;
        let project = self.deps.projects.project(cut.project_id).await?;
        let (resolved, tracks) = resolve_with_tracks(&project, &silence_request(cut), &media)?;
        Ok(resolved.summary(&tracks))
    }

    /// Cuts the silences of `cut` (one undo step).
    ///
    /// # Errors
    /// Invalid parameters, not open, unknown sequence or tracks, locked tracks, storage errors.
    pub async fn remove_silences(&self, cut: SilenceCut) -> Result<CutSummary, AppError> {
        check_params(&cut.params)?;
        let media = self.media(cut.project_id, cut.sequence_id).await?;
        let project_id = cut.project_id;
        self.deps
            .projects
            .apply_cut(
                project_id,
                labels::REMOVE_SILENCES,
                &silence_request(cut),
                &media,
            )
            .await
    }
}

/// The request of a silence cut.
fn silence_request(cut: SilenceCut) -> CutRequest {
    CutRequest {
        sequence_id: cut.sequence_id,
        target: CutTarget::Silences {
            range: cut.range,
            params: cut.params,
            tracks: cut.track_ids,
        },
        all_tracks: cut.all_tracks,
        limit: None,
    }
}

/// Refuses silence parameters outside their ranges.
///
/// # Errors
/// [`AppError::InvalidSetting`].
pub fn check_params(params: &SilenceCutParams) -> Result<(), AppError> {
    let mut settings = settings::default_settings();
    settings.silence = *params;
    if !THRESHOLD_RANGE.contains(&params.threshold_dbfs) {
        return Err(AppError::InvalidSetting {
            name: "thresholdDbfs",
            reason: "choose a threshold from -80 to -10 dBFS".to_owned(),
        });
    }
    settings::check(&settings)
}
