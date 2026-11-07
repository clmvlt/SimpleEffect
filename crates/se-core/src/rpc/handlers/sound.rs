//! Sound and transition commands on an open project (prompt 14): item patches, keyframes of an
//! animatable property and cross-fades. One undoable transaction each, through
//! `se_app::ProjectService`.

use se_domain::{ItemId, ProjectId, PropertyAnimation, TransitionId, TransitionPatch};
use se_protocol::{CommandResult, ItemAnimation, ItemUpdate, NewTransition, ProtocolError};

use super::HandlerContext;

/// `updateItems`.
pub async fn update_items(
    context: &HandlerContext,
    project_id: ProjectId,
    updates: &[ItemUpdate],
    animations: &[ItemAnimation],
) -> Result<CommandResult, ProtocolError> {
    context
        .projects
        .update_items(project_id, updates, animations)
        .await?;
    Ok(CommandResult::UpdateItems)
}

/// `setItemAnimation`.
pub async fn set_item_animation(
    context: &HandlerContext,
    project_id: ProjectId,
    item_id: ItemId,
    animation: &PropertyAnimation,
) -> Result<CommandResult, ProtocolError> {
    context
        .projects
        .set_item_animation(project_id, item_id, animation.clone())
        .await?;
    Ok(CommandResult::SetItemAnimation)
}

/// `addTransitions`.
pub async fn add_transitions(
    context: &HandlerContext,
    project_id: ProjectId,
    transitions: &[NewTransition],
) -> Result<CommandResult, ProtocolError> {
    let transition_ids = context
        .projects
        .add_transitions(project_id, transitions)
        .await?;
    Ok(CommandResult::AddTransitions { transition_ids })
}

/// `updateTransition`.
pub async fn update_transition(
    context: &HandlerContext,
    project_id: ProjectId,
    transition_id: TransitionId,
    patch: &TransitionPatch,
) -> Result<CommandResult, ProtocolError> {
    context
        .projects
        .update_transition(project_id, transition_id, patch.clone())
        .await?;
    Ok(CommandResult::UpdateTransition)
}

/// `removeTransitions`.
pub async fn remove_transitions(
    context: &HandlerContext,
    project_id: ProjectId,
    transition_ids: &[TransitionId],
) -> Result<CommandResult, ProtocolError> {
    context
        .projects
        .remove_transitions(project_id, transition_ids)
        .await?;
    Ok(CommandResult::RemoveTransitions)
}
