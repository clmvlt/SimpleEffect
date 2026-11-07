//! Track and marker commands on an open project: one undoable transaction each.

use se_domain::{
    Flicks, MarkerColor, MarkerId, MarkerPatch, ProjectId, SequenceId, TrackId, TrackKind,
    TrackPatch,
};
use se_protocol::{CommandResult, ProtocolError};

use super::HandlerContext;

/// `addTrack`.
pub async fn add_track(
    context: &HandlerContext,
    project_id: ProjectId,
    sequence_id: SequenceId,
    kind: TrackKind,
    index: Option<usize>,
) -> Result<CommandResult, ProtocolError> {
    let track_id = context
        .projects
        .add_track(project_id, sequence_id, kind, index)
        .await?;
    Ok(CommandResult::AddTrack { track_id })
}

/// `removeTrack`.
pub async fn remove_track(
    context: &HandlerContext,
    project_id: ProjectId,
    track_id: TrackId,
) -> Result<CommandResult, ProtocolError> {
    context.projects.remove_track(project_id, track_id).await?;
    Ok(CommandResult::RemoveTrack)
}

/// `updateTrack`.
pub async fn update_track(
    context: &HandlerContext,
    project_id: ProjectId,
    track_id: TrackId,
    patch: &TrackPatch,
) -> Result<CommandResult, ProtocolError> {
    context
        .projects
        .update_track(project_id, track_id, patch.clone())
        .await?;
    Ok(CommandResult::UpdateTrack)
}

/// `reorderTracks`.
pub async fn reorder_tracks(
    context: &HandlerContext,
    project_id: ProjectId,
    sequence_id: SequenceId,
    order: &[TrackId],
) -> Result<CommandResult, ProtocolError> {
    context
        .projects
        .reorder_tracks(project_id, sequence_id, order.to_vec())
        .await?;
    Ok(CommandResult::ReorderTracks)
}

/// `addMarker`.
pub async fn add_marker(
    context: &HandlerContext,
    project_id: ProjectId,
    sequence_id: SequenceId,
    time: Flicks,
    label: &str,
    color: Option<MarkerColor>,
) -> Result<CommandResult, ProtocolError> {
    let marker_id = context
        .projects
        .add_marker(project_id, sequence_id, time, label.to_owned(), color)
        .await?;
    Ok(CommandResult::AddMarker { marker_id })
}

/// `updateMarker`.
pub async fn update_marker(
    context: &HandlerContext,
    project_id: ProjectId,
    marker_id: MarkerId,
    patch: &MarkerPatch,
) -> Result<CommandResult, ProtocolError> {
    context
        .projects
        .update_marker(project_id, marker_id, patch.clone())
        .await?;
    Ok(CommandResult::UpdateMarker)
}

/// `removeMarker`.
pub async fn remove_marker(
    context: &HandlerContext,
    project_id: ProjectId,
    marker_id: MarkerId,
) -> Result<CommandResult, ProtocolError> {
    context
        .projects
        .remove_marker(project_id, marker_id)
        .await?;
    Ok(CommandResult::RemoveMarker)
}
