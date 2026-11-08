//! Transcription commands (prompt 21): status and preferences, downloads, model files, jobs,
//! transcripts, search and the project's transcription settings.

use se_domain::{AssetId, ProjectId, TranscriptionSettings};
use se_protocol::{
    CommandResult, DownloadTarget, ProtocolError, TranscriptSearchHit, TranscriptionPreferences,
};

use super::{HandlerContext, absolute_path};

/// `getTranscriptionStatus`.
pub async fn get_status(context: &HandlerContext) -> Result<CommandResult, ProtocolError> {
    let status = context.transcription.status().await?;
    Ok(CommandResult::GetTranscriptionStatus { status })
}

/// `updateTranscriptionPreferences`.
pub async fn update_preferences(
    context: &HandlerContext,
    preferences: &TranscriptionPreferences,
) -> Result<CommandResult, ProtocolError> {
    let status = context
        .transcription
        .update_preferences(preferences.clone())
        .await?;
    Ok(CommandResult::UpdateTranscriptionPreferences { status })
}

/// `startTranscriptionDownload`.
pub fn start_download(
    context: &HandlerContext,
    target: &DownloadTarget,
) -> Result<CommandResult, ProtocolError> {
    context.transcription.start_download(target)?;
    Ok(CommandResult::StartTranscriptionDownload)
}

/// `cancelTranscriptionDownload`.
pub fn cancel_download(
    context: &HandlerContext,
    target: &DownloadTarget,
) -> Result<CommandResult, ProtocolError> {
    context.transcription.cancel_download(target)?;
    Ok(CommandResult::CancelTranscriptionDownload)
}

/// `importTranscriptionModel`.
pub async fn import_model(
    context: &HandlerContext,
    path: &str,
) -> Result<CommandResult, ProtocolError> {
    let (model_id, status) = context
        .transcription
        .import_model(absolute_path(path)?)
        .await?;
    Ok(CommandResult::ImportTranscriptionModel { model_id, status })
}

/// `removeTranscriptionFile`.
pub async fn remove_file(
    context: &HandlerContext,
    target: &DownloadTarget,
) -> Result<CommandResult, ProtocolError> {
    let status = context.transcription.remove_file(target).await?;
    Ok(CommandResult::RemoveTranscriptionFile { status })
}

/// `transcribeAssets`.
pub async fn transcribe_assets(
    context: &HandlerContext,
    project_id: ProjectId,
    asset_ids: &[AssetId],
) -> Result<CommandResult, ProtocolError> {
    let job_ids = context
        .transcription
        .transcribe_assets(project_id, asset_ids)
        .await?;
    Ok(CommandResult::TranscribeAssets { job_ids })
}

/// `getTranscript`.
pub async fn get_transcript(
    context: &HandlerContext,
    project_id: ProjectId,
    asset_id: AssetId,
) -> Result<CommandResult, ProtocolError> {
    let transcript = context
        .transcription
        .transcript(project_id, asset_id)
        .await?;
    Ok(CommandResult::GetTranscript { transcript })
}

/// `searchTranscripts`.
pub async fn search(
    context: &HandlerContext,
    project_id: ProjectId,
    query: &str,
) -> Result<CommandResult, ProtocolError> {
    let hits = context
        .transcription
        .search(project_id, query)
        .await?
        .into_iter()
        .map(|hit| TranscriptSearchHit {
            asset_id: hit.asset_id,
            segment: hit.segment,
            start: hit.start,
            end: hit.end,
            text: hit.text,
        })
        .collect();
    Ok(CommandResult::SearchTranscripts { hits })
}

/// `setTranscriptionSettings`.
pub async fn set_settings(
    context: &HandlerContext,
    project_id: ProjectId,
    settings: &TranscriptionSettings,
) -> Result<CommandResult, ProtocolError> {
    context
        .transcription
        .set_project_settings(project_id, settings.clone())
        .await?;
    Ok(CommandResult::SetTranscriptionSettings)
}
