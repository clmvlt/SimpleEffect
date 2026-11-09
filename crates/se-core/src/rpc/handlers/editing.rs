//! Commands on an open project: snapshot, sequences, undo and redo.

use se_domain::{Format, FormatAdaptation, ProjectId, SequenceId, TrackingSettings};
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

/// `renameSequence`.
pub async fn rename_sequence(
    context: &HandlerContext,
    project_id: ProjectId,
    sequence_id: SequenceId,
    name: &str,
) -> Result<CommandResult, ProtocolError> {
    context
        .projects
        .rename_sequence(project_id, sequence_id, name)
        .await?;
    Ok(CommandResult::RenameSequence)
}

/// `setSequenceFormat`.
pub async fn set_sequence_format(
    context: &HandlerContext,
    project_id: ProjectId,
    sequence_id: SequenceId,
    format: Format,
    adaptation: FormatAdaptation,
) -> Result<CommandResult, ProtocolError> {
    context
        .projects
        .set_sequence_format(project_id, sequence_id, format, adaptation)
        .await?;
    Ok(CommandResult::SetSequenceFormat)
}

/// What a sequence copy is made of (`duplicateSequence`).
pub struct SequenceCopy<'a> {
    /// The sequence copied.
    pub sequence_id: SequenceId,
    /// Name of the copy.
    pub name: &'a str,
    /// Its format (`None`: the source's).
    pub format: Option<Format>,
    /// How its items adapt.
    pub adaptation: FormatAdaptation,
    /// Make its clips follow their subject with these settings (prompt 26).
    pub auto_reframe: Option<TrackingSettings>,
}

/// `duplicateSequence`.
pub async fn duplicate_sequence(
    context: &HandlerContext,
    project_id: ProjectId,
    copy: SequenceCopy<'_>,
) -> Result<CommandResult, ProtocolError> {
    if let Some(settings) = copy.auto_reframe {
        let (sequence_id, report) = context
            .reframing
            .create_version(
                project_id,
                copy.sequence_id,
                copy.name,
                copy.format,
                copy.adaptation,
                settings,
            )
            .await?;
        return Ok(CommandResult::DuplicateSequence {
            sequence_id,
            reframe: Some(report),
        });
    }
    let sequence_id = context
        .projects
        .duplicate_sequence(
            project_id,
            copy.sequence_id,
            copy.name,
            copy.format,
            copy.adaptation,
        )
        .await?;
    Ok(CommandResult::DuplicateSequence {
        sequence_id,
        reframe: None,
    })
}

/// `removeSequence`.
pub async fn remove_sequence(
    context: &HandlerContext,
    project_id: ProjectId,
    sequence_id: SequenceId,
) -> Result<CommandResult, ProtocolError> {
    context
        .projects
        .remove_sequence(project_id, sequence_id)
        .await?;
    Ok(CommandResult::RemoveSequence)
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
