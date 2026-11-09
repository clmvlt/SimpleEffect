//! Automatic reframing commands (prompt 26, ADR-0037): whether faces can be detected here,
//! reframing clips, and the faces of a media for the viewer. Versions with reframing go through
//! `duplicateSequence` (`editing.rs`).

use se_domain::{AssetId, ItemId, ProjectId, TrackingSettings};
use se_protocol::{CommandResult, ProtocolError};

use super::HandlerContext;

/// `getVisionStatus`.
pub async fn get_status(context: &HandlerContext) -> Result<CommandResult, ProtocolError> {
    let status = context.reframing.status().await;
    Ok(CommandResult::GetVisionStatus { status })
}

/// `autoReframeItems`.
pub async fn auto_reframe_items(
    context: &HandlerContext,
    project_id: ProjectId,
    item_ids: &[ItemId],
    settings: TrackingSettings,
) -> Result<CommandResult, ProtocolError> {
    let report = context
        .reframing
        .reframe_items(project_id, item_ids, settings)
        .await?;
    Ok(CommandResult::AutoReframeItems(report))
}

/// `getSubjectDetection`.
pub async fn get_subject_detection(
    context: &HandlerContext,
    project_id: ProjectId,
    asset_id: AssetId,
) -> Result<CommandResult, ProtocolError> {
    let detection = context
        .reframing
        .subject_detection(project_id, asset_id)
        .await?;
    Ok(CommandResult::GetSubjectDetection { detection })
}
