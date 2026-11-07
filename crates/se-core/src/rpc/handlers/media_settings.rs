//! Media settings commands: settings, tools status, and the project cache.

use se_domain::ProjectId;
use se_protocol::{CommandResult, MediaSettings, ProtocolError};

use super::HandlerContext;

/// `getMediaSettings`.
pub async fn get_media_settings(context: &HandlerContext) -> Result<CommandResult, ProtocolError> {
    let settings = context.media.media_settings().await?;
    Ok(CommandResult::GetMediaSettings { settings })
}

/// `updateMediaSettings`.
pub async fn update_media_settings(
    context: &HandlerContext,
    settings: &MediaSettings,
) -> Result<CommandResult, ProtocolError> {
    let settings = context
        .media
        .update_media_settings(settings.clone())
        .await?;
    Ok(CommandResult::UpdateMediaSettings { settings })
}

/// `getMediaToolsStatus`.
pub async fn get_media_tools_status(
    context: &HandlerContext,
) -> Result<CommandResult, ProtocolError> {
    let status = context.media.tools_status().await;
    Ok(CommandResult::GetMediaToolsStatus { status })
}

/// `getProjectCache`.
pub async fn get_project_cache(
    context: &HandlerContext,
    project_id: ProjectId,
) -> Result<CommandResult, ProtocolError> {
    let cache = context.media.project_cache(project_id).await?;
    Ok(CommandResult::GetProjectCache { cache })
}

/// `clearProjectCache`.
pub async fn clear_project_cache(
    context: &HandlerContext,
    project_id: ProjectId,
) -> Result<CommandResult, ProtocolError> {
    let cache = context.media.clear_project_cache(project_id).await?;
    Ok(CommandResult::ClearProjectCache { cache })
}
