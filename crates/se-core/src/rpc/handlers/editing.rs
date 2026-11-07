//! Commands on an open project: snapshot, sequences, undo and redo.

use se_domain::{Format, ProjectId};
use se_protocol::{CommandResult, ProtocolError};

use super::HandlerContext;

/// `getProjectSnapshot`.
pub async fn get_project_snapshot(
    context: &HandlerContext,
    project_id: ProjectId,
) -> Result<CommandResult, ProtocolError> {
    let session = context.projects.project_session(project_id).await?;
    Ok(CommandResult::GetProjectSnapshot(session))
}

/// `createSequence`.
pub async fn create_sequence(
    context: &HandlerContext,
    project_id: ProjectId,
    name: &str,
    format: Option<Format>,
) -> Result<CommandResult, ProtocolError> {
    let sequence_id = context
        .projects
        .create_sequence(project_id, name, format)
        .await?;
    Ok(CommandResult::CreateSequence { sequence_id })
}

/// `undo`.
pub async fn undo(
    context: &HandlerContext,
    project_id: ProjectId,
) -> Result<CommandResult, ProtocolError> {
    context.projects.undo(project_id).await?;
    Ok(CommandResult::Undo)
}

/// `redo`.
pub async fn redo(
    context: &HandlerContext,
    project_id: ProjectId,
) -> Result<CommandResult, ProtocolError> {
    context.projects.redo(project_id).await?;
    Ok(CommandResult::Redo)
}
