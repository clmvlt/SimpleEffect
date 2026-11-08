//! [`CaptionService`]: the user's caption use cases, each one undoable transaction.

use std::{path::Path, sync::Arc};

use se_domain::{
    CaptionStyleId, Flicks, ItemId, Origin, ProjectId, SequenceId, captions::TimedWord,
};
use se_protocol::{
    CaptionEmphasisEdit, CaptionStyleEdit, CaptionTextEdit, CaptionsReport,
    GenerateCaptionsRequest, ImportSubtitlesRequest,
};

use super::{
    edits,
    generate::{CaptionPlacement, PlacedCaptions, plan_captions, plan_generation, sequence_extent},
    labels, styles,
    subtitles::{cues_of, drafts_extent, drafts_of_cues},
};
use crate::{
    AppError, ProjectService, TranscriptionService, edit_plan::find_sequence, ports::SubtitleFiles,
};

/// What the caption use cases work with.
#[derive(Clone)]
pub struct CaptionServiceDeps {
    /// Open projects: documents and their actors.
    pub projects: Arc<ProjectService>,
    /// Transcripts (the words captions are made of).
    pub transcription: TranscriptionService,
    /// SRT files.
    pub subtitles: Arc<dyn SubtitleFiles>,
}

/// Caption use cases. Cheap to clone.
#[derive(Clone)]
pub struct CaptionService {
    deps: CaptionServiceDeps,
}

impl CaptionService {
    /// The service over `deps`.
    #[must_use]
    pub const fn new(deps: CaptionServiceDeps) -> Self {
        Self { deps }
    }

    /// The words spoken in `sequence_id` (sequence time) and the media not transcribed yet.
    ///
    /// # Errors
    /// Not open, unknown sequence, storage errors.
    pub async fn spoken_words(
        &self,
        project_id: ProjectId,
        sequence_id: SequenceId,
    ) -> Result<(Vec<TimedWord>, Vec<se_domain::AssetId>), AppError> {
        let words = self
            .deps
            .transcription
            .sequence_words(project_id, sequence_id)
            .await?;
        let timed = words
            .transcript
            .words
            .into_iter()
            .map(|word| TimedWord {
                text: word.text,
                start: word.start,
                end: word.end,
            })
            .collect();
        Ok((timed, words.untranscribed))
    }

    /// Makes captions from the transcript over the request's range (one transaction « Generate
    /// captions »; nothing when no word is spoken there).
    ///
    /// # Errors
    /// Not open, unknown sequence or style, an unknown preset, overrides out of range, the
    /// domain's refusals, storage errors.
    pub async fn generate(
        &self,
        request: &GenerateCaptionsRequest,
    ) -> Result<CaptionsReport, AppError> {
        let (words, untranscribed) = self
            .spoken_words(request.project_id, request.sequence_id)
            .await?;
        let placed = self
            .deps
            .projects
            .plan_user_edit(request.project_id, labels::GENERATE_CAPTIONS, |plan| {
                let sequence = find_sequence(plan.project()?, request.sequence_id)?;
                let Some(range) = request.range.or_else(|| sequence_extent(sequence)) else {
                    return Ok(PlacedCaptions::default());
                };
                let placement = CaptionPlacement {
                    sequence_id: request.sequence_id,
                    range,
                    style: request.style.as_ref(),
                    track_name: request.track_name.as_deref(),
                    origin: Origin::User,
                };
                plan_generation(plan, &placement, &words, request.segmentation.as_ref())
            })
            .await?;
        Ok(report(placed, untranscribed))
    }

    /// Turns an SRT file into captions (one transaction « Import subtitles »).
    ///
    /// # Errors
    /// Unreadable or invalid file, not open, unknown sequence or style, storage errors.
    pub async fn import_subtitles(
        &self,
        request: &ImportSubtitlesRequest,
    ) -> Result<CaptionsReport, AppError> {
        let cues = self
            .deps
            .subtitles
            .read_srt(Path::new(&request.path))
            .await?;
        let drafts = drafts_of_cues(&cues);
        let Some(range) = drafts_extent(&drafts) else {
            return Ok(report(PlacedCaptions::default(), Vec::new()));
        };
        let placed = self
            .deps
            .projects
            .plan_user_edit(request.project_id, labels::IMPORT_SUBTITLES, |plan| {
                let placement = CaptionPlacement {
                    sequence_id: request.sequence_id,
                    range,
                    style: request.style.as_ref(),
                    track_name: request.track_name.as_deref(),
                    origin: Origin::User,
                };
                plan_captions(plan, &placement, |_, _| drafts)
            })
            .await?;
        Ok(report(placed, Vec::new()))
    }

    /// Writes the captions of a sequence to an SRT file; returns how many were written.
    ///
    /// # Errors
    /// Not open, unknown sequence, the file cannot be written.
    pub async fn export_subtitles(
        &self,
        project_id: ProjectId,
        sequence_id: SequenceId,
        path: &Path,
    ) -> Result<u32, AppError> {
        let project = self.deps.projects.project(project_id).await?;
        let sequence = find_sequence(&project, sequence_id)?;
        let cues = cues_of(&project, sequence);
        self.deps.subtitles.write_srt(path, &cues).await?;
        Ok(u32::try_from(cues.len()).unwrap_or(u32::MAX))
    }

    /// Changes a caption's text (« Edit caption text »).
    ///
    /// # Errors
    /// Not open, not a caption, an empty text, storage errors.
    pub async fn edit_text(&self, edit: &CaptionTextEdit) -> Result<(), AppError> {
        self.deps
            .projects
            .plan_user_edit(edit.project_id, labels::EDIT_CAPTION_TEXT, |plan| {
                edits::plan_text(plan, edit.item_id, &edit.text)
            })
            .await
    }

    /// Puts words of a caption forward or back (« Emphasize caption words »).
    ///
    /// # Errors
    /// Not open, not a caption, unknown word indices, storage errors.
    pub async fn set_emphasis(&self, edit: &CaptionEmphasisEdit) -> Result<(), AppError> {
        self.deps
            .projects
            .plan_user_edit(edit.project_id, labels::EMPHASIZE_WORDS, |plan| {
                edits::plan_emphasis(plan, edit.item_id, &edit.word_indices, edit.emphasis)
            })
            .await
    }

    /// Merges captions following each other on one track (« Merge captions »).
    ///
    /// # Errors
    /// Not open, fewer than two captions, captions apart or on several tracks, storage errors.
    pub async fn merge(&self, project_id: ProjectId, items: &[ItemId]) -> Result<ItemId, AppError> {
        self.deps
            .projects
            .plan_user_edit(project_id, labels::MERGE_CAPTIONS, |plan| {
                edits::plan_merge(plan, items)
            })
            .await
    }

    /// Splits a caption before a word (« Split caption »); returns the right part.
    ///
    /// # Errors
    /// Not open, not a caption, a word index leaving no word on one side, storage errors.
    pub async fn split(
        &self,
        project_id: ProjectId,
        item: ItemId,
        word_index: u32,
    ) -> Result<ItemId, AppError> {
        self.deps
            .projects
            .plan_user_edit(project_id, labels::SPLIT_CAPTION, |plan| {
                edits::plan_split(plan, item, word_index)
            })
            .await
    }

    /// Moves captions in time together (« Shift captions »).
    ///
    /// # Errors
    /// Not open, no item, items before 0 or overlapping others afterwards, storage errors.
    pub async fn shift(
        &self,
        project_id: ProjectId,
        items: &[ItemId],
        delta: Flicks,
    ) -> Result<(), AppError> {
        self.deps
            .projects
            .plan_user_edit(project_id, labels::SHIFT_CAPTIONS, |plan| {
                edits::plan_shift(plan, items, delta)
            })
            .await
    }

    /// Changes the look of captions, copying styles they share with others (« Change caption
    /// style »); returns the styles they use afterwards.
    ///
    /// # Errors
    /// Not open, not captions, an empty patch, values out of range, storage errors.
    pub async fn update_style(
        &self,
        edit: &CaptionStyleEdit,
    ) -> Result<Vec<CaptionStyleId>, AppError> {
        self.deps
            .projects
            .plan_user_edit(edit.project_id, labels::CHANGE_CAPTION_STYLE, |plan| {
                styles::patch_styles(plan, &edit.item_ids, &edit.patch)
            })
            .await
    }

    /// Gives captions a new style made from a preset (« Apply caption style »).
    ///
    /// # Errors
    /// Not open, not captions, an unknown preset, storage errors.
    pub async fn apply_preset(
        &self,
        project_id: ProjectId,
        items: &[ItemId],
        preset: &str,
    ) -> Result<CaptionStyleId, AppError> {
        self.deps
            .projects
            .plan_user_edit(project_id, labels::APPLY_CAPTION_PRESET, |plan| {
                styles::apply_preset(plan, items, preset)
            })
            .await
    }
}

/// The protocol report of placed captions.
fn report(placed: PlacedCaptions, untranscribed: Vec<se_domain::AssetId>) -> CaptionsReport {
    CaptionsReport {
        item_ids: placed.item_ids,
        track_id: placed.track_id,
        style_id: placed.style_id,
        replaced: placed.replaced,
        untranscribed_assets: untranscribed,
    }
}
