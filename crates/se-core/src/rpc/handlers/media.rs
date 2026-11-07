//! Media commands: import, relink, remove, jobs, decodability reports and derived copies.

use std::path::PathBuf;

use se_domain::{AssetId, JobId, ProjectId};
use se_protocol::{
    CommandResult, DecodabilityReport, DecoderCapabilities, GenerateDerivedRequest,
    ImportMediaRequest, ProtocolError, RelinkAssetRequest,
};

use super::{HandlerContext, absolute_path};

/// `importMedia`: every path must be absolute.
pub async fn import_media(
    context: &HandlerContext,
    request: &ImportMediaRequest,
) -> Result<CommandResult, ProtocolError> {
    let paths = request
        .paths
        .iter()
        .map(|path| absolute_path(path).map(PathBuf::from))
        .collect::<Result<Vec<_>, _>>()?;
    let report = context
        .media
        .import_files(request.project_id, &paths)
        .await?;
    Ok(CommandResult::ImportMedia(report))
}

/// `relinkAsset`.
pub async fn relink_asset(
    context: &HandlerContext,
    request: &RelinkAssetRequest,
) -> Result<CommandResult, ProtocolError> {
    let path = absolute_path(&request.path)?;
    let outcome = context
        .media
        .relink_asset(request.project_id, request.asset_id, path, request.force)
        .await?;
    Ok(CommandResult::RelinkAsset { outcome })
}

/// `removeAsset`.
pub async fn remove_asset(
    context: &HandlerContext,
    project_id: ProjectId,
    asset_id: AssetId,
) -> Result<CommandResult, ProtocolError> {
    context.media.remove_asset(project_id, asset_id).await?;
    Ok(CommandResult::RemoveAsset)
}

/// `listJobs`.
pub fn list_jobs(
    context: &HandlerContext,
    project_id: ProjectId,
) -> Result<CommandResult, ProtocolError> {
    let jobs = context.media.list_jobs(project_id)?;
    Ok(CommandResult::ListJobs { jobs })
}

/// `cancelJob`.
pub fn cancel_job(context: &HandlerContext, job_id: JobId) -> Result<CommandResult, ProtocolError> {
    context.media.cancel_job(job_id)?;
    Ok(CommandResult::CancelJob)
}

/// `reportDecodability`.
pub async fn report_decodability(
    context: &HandlerContext,
    report: &DecodabilityReport,
) -> Result<CommandResult, ProtocolError> {
    context.media.report_decodability(report).await?;
    Ok(CommandResult::ReportDecodability)
}

/// `reportDecoderCapabilities`.
pub async fn report_decoder_capabilities(
    context: &HandlerContext,
    capabilities: DecoderCapabilities,
) -> Result<CommandResult, ProtocolError> {
    context
        .media
        .report_decoder_capabilities(capabilities)
        .await?;
    Ok(CommandResult::ReportDecoderCapabilities)
}

/// `generateDerivedMedia`.
pub async fn generate_derived_media(
    context: &HandlerContext,
    request: &GenerateDerivedRequest,
) -> Result<CommandResult, ProtocolError> {
    context.media.generate_derived(request).await?;
    Ok(CommandResult::GenerateDerivedMedia)
}

/// `getAudioSilences`.
pub async fn get_audio_silences(
    context: &HandlerContext,
    project_id: ProjectId,
    asset_id: AssetId,
) -> Result<CommandResult, ProtocolError> {
    let silences = context.media.audio_silences(project_id, asset_id).await?;
    Ok(CommandResult::GetAudioSilences { silences })
}
