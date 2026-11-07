//! Item edit commands on an open project (prompt 11): one undoable transaction each, planned by
//! `se_app::ProjectService` (ADR-0021).

use se_domain::{Flicks, ItemId, ProjectId, TimeRange, TrackId};
use se_protocol::{
    CommandResult, ErrorCode, ItemPlacement, ItemTrim, PastedItem, PlaceMediaRequest,
    PlacementMode, ProtocolError,
};

use super::HandlerContext;

/// `placeMedia`.
pub async fn place_media(
    context: &HandlerContext,
    request: &PlaceMediaRequest,
) -> Result<CommandResult, ProtocolError> {
    let item_ids = context.projects.place_media(request).await?;
    Ok(CommandResult::PlaceMedia { item_ids })
}

/// `moveItems`.
pub async fn move_items(
    context: &HandlerContext,
    project_id: ProjectId,
    moves: &[ItemPlacement],
    mode: PlacementMode,
) -> Result<CommandResult, ProtocolError> {
    context.projects.move_items(project_id, moves, mode).await?;
    Ok(CommandResult::MoveItems)
}

/// `duplicateItems`.
pub async fn duplicate_items(
    context: &HandlerContext,
    project_id: ProjectId,
    copies: &[ItemPlacement],
    mode: PlacementMode,
) -> Result<CommandResult, ProtocolError> {
    let item_ids = context
        .projects
        .duplicate_items(project_id, copies, mode)
        .await?;
    Ok(CommandResult::DuplicateItems { item_ids })
}

/// `pasteItems`.
pub async fn paste_items(
    context: &HandlerContext,
    project_id: ProjectId,
    items: &[PastedItem],
    mode: PlacementMode,
) -> Result<CommandResult, ProtocolError> {
    let item_ids = context
        .projects
        .paste_items(project_id, items, mode)
        .await?;
    Ok(CommandResult::PasteItems { item_ids })
}

/// `trimItems`.
pub async fn trim_items(
    context: &HandlerContext,
    project_id: ProjectId,
    trims: &[ItemTrim],
    ripple: bool,
) -> Result<CommandResult, ProtocolError> {
    context
        .projects
        .trim_items(project_id, trims, ripple)
        .await?;
    Ok(CommandResult::TrimItems)
}

/// `splitItems`.
pub async fn split_items(
    context: &HandlerContext,
    project_id: ProjectId,
    item_ids: &[ItemId],
    at: Flicks,
) -> Result<CommandResult, ProtocolError> {
    let item_ids = context
        .projects
        .split_items(project_id, item_ids, at)
        .await?;
    Ok(CommandResult::SplitItems { item_ids })
}

/// `deleteItems`.
pub async fn delete_items(
    context: &HandlerContext,
    project_id: ProjectId,
    item_ids: &[ItemId],
    ripple: bool,
) -> Result<CommandResult, ProtocolError> {
    context
        .projects
        .delete_items(project_id, item_ids, ripple)
        .await?;
    Ok(CommandResult::DeleteItems)
}

/// `linkItems`.
pub async fn link_items(
    context: &HandlerContext,
    project_id: ProjectId,
    item_ids: &[ItemId],
) -> Result<CommandResult, ProtocolError> {
    let link_group = context.projects.link_items(project_id, item_ids).await?;
    Ok(CommandResult::LinkItems { link_group })
}

/// `unlinkItems`.
pub async fn unlink_items(
    context: &HandlerContext,
    project_id: ProjectId,
    item_ids: &[ItemId],
) -> Result<CommandResult, ProtocolError> {
    context.projects.unlink_items(project_id, item_ids).await?;
    Ok(CommandResult::UnlinkItems)
}

/// `resizeGap`.
pub async fn resize_gap(
    context: &HandlerContext,
    project_id: ProjectId,
    track_id: TrackId,
    (start, end): (Flicks, Flicks),
    duration: Flicks,
) -> Result<CommandResult, ProtocolError> {
    let gap = TimeRange::from_start_end(start, end).map_err(|error| {
        ProtocolError::new(
            ErrorCode::InvalidRequest,
            format!("invalid gap: {error}; send the start and end of the gap, start first"),
        )
    })?;
    context
        .projects
        .resize_gap(project_id, track_id, gap, duration)
        .await?;
    Ok(CommandResult::ResizeGap)
}
