//! Smart-cut commands (prompt 22, ADR-0032): settings, word and filler cuts, silence previews
//! and cuts.

use se_app::smart_cuts::SilenceCut;
use se_domain::{ProjectId, SequenceId, TimeRange, TrackId};
use se_protocol::{
    CommandResult, ProtocolError, SilenceCutParams, SmartCutSettings, WordCutPurpose,
};

use super::HandlerContext;

/// `getSmartCutSettings`.
pub async fn get_settings(context: &HandlerContext) -> Result<CommandResult, ProtocolError> {
    Ok(CommandResult::GetSmartCutSettings(
        context.smart_cuts.settings().await?,
    ))
}

/// `updateSmartCutSettings`.
pub async fn update_settings(
    context: &HandlerContext,
    settings: &SmartCutSettings,
) -> Result<CommandResult, ProtocolError> {
    Ok(CommandResult::UpdateSmartCutSettings(
        context.smart_cuts.update_settings(settings).await?,
    ))
}

/// `removeWords`.
pub async fn remove_words(
    context: &HandlerContext,
    project_id: ProjectId,
    sequence_id: SequenceId,
    ranges: &[TimeRange],
    purpose: WordCutPurpose,
    all_tracks: bool,
) -> Result<CommandResult, ProtocolError> {
    let summary = context
        .smart_cuts
        .remove_words(
            project_id,
            sequence_id,
            ranges.to_vec(),
            purpose,
            all_tracks,
        )
        .await?;
    Ok(CommandResult::RemoveWords(summary))
}

/// `findFillers`.
pub async fn find_fillers(
    context: &HandlerContext,
    project_id: ProjectId,
    sequence_id: SequenceId,
    range: Option<TimeRange>,
) -> Result<CommandResult, ProtocolError> {
    let (occurrences, untranscribed_assets) = context
        .smart_cuts
        .find_fillers(project_id, sequence_id, range)
        .await?;
    Ok(CommandResult::FindFillers {
        occurrences,
        untranscribed_assets,
    })
}

/// The arguments shared by `previewSilenceCuts` and `removeSilences`.
pub struct SilenceArguments<'a> {
    /// The open project.
    pub project_id: ProjectId,
    /// The sequence.
    pub sequence_id: SequenceId,
    /// Where (`None`: the whole sequence).
    pub range: Option<TimeRange>,
    /// Threshold, minimum silence, padding.
    pub params: SilenceCutParams,
    /// Sound tracks listened to.
    pub track_ids: Option<&'a [TrackId]>,
    /// Cut every unlocked track.
    pub all_tracks: bool,
}

impl SilenceArguments<'_> {
    fn cut(&self) -> SilenceCut {
        SilenceCut {
            project_id: self.project_id,
            sequence_id: self.sequence_id,
            range: self.range,
            params: self.params,
            track_ids: self.track_ids.map(<[TrackId]>::to_vec),
            all_tracks: self.all_tracks,
        }
    }
}

/// `previewSilenceCuts`.
pub async fn preview_silences(
    context: &HandlerContext,
    arguments: &SilenceArguments<'_>,
) -> Result<CommandResult, ProtocolError> {
    let summary = context.smart_cuts.preview_silences(arguments.cut()).await?;
    Ok(CommandResult::PreviewSilenceCuts(summary))
}

/// `removeSilences`.
pub async fn remove_silences(
    context: &HandlerContext,
    arguments: &SilenceArguments<'_>,
) -> Result<CommandResult, ProtocolError> {
    let summary = context.smart_cuts.remove_silences(arguments.cut()).await?;
    Ok(CommandResult::RemoveSilences(summary))
}
