//! Caption commands (prompt 23): generation, SRT import and export, text, emphasis, merge,
//! split, shift and style edits — each edit one undoable transaction.

use se_domain::{Flicks, ItemId, ProjectId, SequenceId};
use se_protocol::{
    CaptionEmphasisEdit, CaptionStyleEdit, CaptionTextEdit, CommandResult, GenerateCaptionsRequest,
    ImportSubtitlesRequest, ProtocolError,
};

use super::{HandlerContext, absolute_path};

/// `generateCaptions`.
pub async fn generate(
    context: &HandlerContext,
    request: &GenerateCaptionsRequest,
) -> Result<CommandResult, ProtocolError> {
    Ok(CommandResult::GenerateCaptions(
        context.captions.generate(request).await?,
    ))
}

/// `importSubtitles`.
pub async fn import_subtitles(
    context: &HandlerContext,
    request: &ImportSubtitlesRequest,
) -> Result<CommandResult, ProtocolError> {
    absolute_path(&request.path)?;
    Ok(CommandResult::ImportSubtitles(
        context.captions.import_subtitles(request).await?,
    ))
}

/// `exportSubtitles`.
pub async fn export_subtitles(
    context: &HandlerContext,
    project_id: ProjectId,
    sequence_id: SequenceId,
    path: &str,
) -> Result<CommandResult, ProtocolError> {
    let path = absolute_path(path)?;
    let cue_count = context
        .captions
        .export_subtitles(project_id, sequence_id, path)
        .await?;
    Ok(CommandResult::ExportSubtitles { cue_count })
}

/// `editCaptionText`.
pub async fn edit_text(
    context: &HandlerContext,
    edit: &CaptionTextEdit,
) -> Result<CommandResult, ProtocolError> {
    context.captions.edit_text(edit).await?;
    Ok(CommandResult::EditCaptionText)
}

/// `setCaptionEmphasis`.
pub async fn set_emphasis(
    context: &HandlerContext,
    edit: &CaptionEmphasisEdit,
) -> Result<CommandResult, ProtocolError> {
    context.captions.set_emphasis(edit).await?;
    Ok(CommandResult::SetCaptionEmphasis)
}

/// `mergeCaptions`.
pub async fn merge(
    context: &HandlerContext,
    project_id: ProjectId,
    item_ids: &[ItemId],
) -> Result<CommandResult, ProtocolError> {
    let item_id = context.captions.merge(project_id, item_ids).await?;
    Ok(CommandResult::MergeCaptions { item_id })
}

/// `splitCaption`.
pub async fn split(
    context: &HandlerContext,
    project_id: ProjectId,
    item_id: ItemId,
    word_index: u32,
) -> Result<CommandResult, ProtocolError> {
    let item_id = context
        .captions
        .split(project_id, item_id, word_index)
        .await?;
    Ok(CommandResult::SplitCaption { item_id })
}

/// `shiftCaptions`.
pub async fn shift(
    context: &HandlerContext,
    project_id: ProjectId,
    item_ids: &[ItemId],
    delta: Flicks,
) -> Result<CommandResult, ProtocolError> {
    context.captions.shift(project_id, item_ids, delta).await?;
    Ok(CommandResult::ShiftCaptions)
}

/// `updateCaptionStyle`.
pub async fn update_style(
    context: &HandlerContext,
    edit: &CaptionStyleEdit,
) -> Result<CommandResult, ProtocolError> {
    let style_ids = context.captions.update_style(edit).await?;
    Ok(CommandResult::UpdateCaptionStyle { style_ids })
}

/// `applyCaptionPreset`.
pub async fn apply_preset(
    context: &HandlerContext,
    project_id: ProjectId,
    item_ids: &[ItemId],
    preset: &str,
) -> Result<CommandResult, ProtocolError> {
    let style_id = context
        .captions
        .apply_preset(project_id, item_ids, preset)
        .await?;
    Ok(CommandResult::ApplyCaptionPreset { style_id })
}
