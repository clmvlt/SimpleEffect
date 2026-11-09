//! Sequence use cases (prompt 16, ADR-0026): create, rename, change the format of, duplicate (as
//! is or as a version in another format) and remove sequences. Each is one user transaction, so
//! one undo step.

use se_domain::{
    AddSequence, DomainError, Format, FormatAdaptation, ItemId, LinkGroupId, MarkerId, ProjectId,
    RemoveSequence, SequenceId, SequenceIds, SequencePatch, TrackId, TransitionId, UpdateSequence,
    format_change, sequence_version,
};

use super::{ProjectService, checked_name};
use crate::{AppError, edit_plan::find_sequence, new_ids::NewIds, new_project::default_sequence};

/// Labels of the sequence transactions (undo menu). Stable identifiers: the renderer translates
/// them (`apps/desktop/src/app/editor/history-label.ts`).
pub mod sequence_labels {
    /// A sequence was added.
    pub const CREATE: &str = "Create sequence";
    /// A sequence was renamed.
    pub const RENAME: &str = "Rename sequence";
    /// A sequence took another format.
    pub const FORMAT: &str = "Change sequence format";
    /// A sequence was copied in its format.
    pub const DUPLICATE: &str = "Duplicate sequence";
    /// A sequence was copied into another format.
    pub const CREATE_VERSION: &str = "Create version";
    /// A sequence was removed.
    pub const REMOVE: &str = "Remove sequence";
}

impl ProjectService {
    /// Adds a sequence with tracks V1, V2, A1, A2. `format` defaults to the project's default
    /// format.
    ///
    /// # Errors
    /// Not open, empty name, a format the document refuses, storage errors.
    pub async fn create_sequence(
        &self,
        project_id: ProjectId,
        name: &str,
        format: Option<Format>,
    ) -> Result<SequenceId, AppError> {
        let handle = self.handle(project_id).await?;
        let name = checked_name("sequence", name)?;
        let format = match format {
            Some(format) => format,
            None => handle.view().await?.project.settings.default_format,
        };
        let sequence = default_sequence(self.new_ids(), &name, format)?;
        let sequence_id = sequence.id;
        let add = AddSequence {
            index: None,
            sequence,
        };
        self.commit_user_edit(&handle, sequence_labels::CREATE, vec![add.into()])
            .await?;
        Ok(sequence_id)
    }

    /// Renames a sequence.
    ///
    /// # Errors
    /// Not open, unknown sequence, empty name, storage errors.
    pub async fn rename_sequence(
        &self,
        project_id: ProjectId,
        sequence_id: SequenceId,
        name: &str,
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id).await?;
        let name = checked_name("sequence", name)?;
        let rename = UpdateSequence {
            sequence: sequence_id,
            patch: SequencePatch {
                name: Some(name),
                format: None,
            },
        };
        self.commit_user_edit(&handle, sequence_labels::RENAME, vec![rename.into()])
            .await
    }

    /// Gives a sequence another format and adapts its items as `adaptation` says
    /// ([`se_domain::format_change`]; items on locked tracks stay).
    ///
    /// # Errors
    /// Not open, unknown sequence, a frame-rate change while the sequence holds items
    /// (`FRAME_RATE_LOCKED`), a format the document refuses, storage errors.
    pub async fn set_sequence_format(
        &self,
        project_id: ProjectId,
        sequence_id: SequenceId,
        format: Format,
        adaptation: FormatAdaptation,
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let ops = format_change(&view.project, sequence_id, format, adaptation)?;
        self.commit_user_edit(&handle, sequence_labels::FORMAT, ops)
            .await
    }

    /// Copies a sequence under `name`, right after it: in its own format (`format: None`), or as
    /// a version in another format with its items adapted ([`se_domain::sequence_version`]). The
    /// copy shares nothing with its source: later edits of one never reach the other.
    ///
    /// # Errors
    /// Not open, unknown sequence, empty name, a frame-rate change while the sequence holds items
    /// (`FRAME_RATE_LOCKED`), a format the document refuses, storage errors.
    pub async fn duplicate_sequence(
        &self,
        project_id: ProjectId,
        sequence_id: SequenceId,
        name: &str,
        format: Option<Format>,
        adaptation: FormatAdaptation,
    ) -> Result<SequenceId, AppError> {
        let handle = self.handle(project_id).await?;
        let name = checked_name("sequence", name)?;
        let view = handle.view().await?;
        let source = find_sequence(&view.project, sequence_id)?;
        let index = view
            .project
            .sequences
            .iter()
            .position(|sequence| sequence.id == sequence_id)
            .map(|position| position + 1);
        let target = format.unwrap_or(source.format);
        let label = if target == source.format {
            sequence_labels::DUPLICATE
        } else {
            sequence_labels::CREATE_VERSION
        };
        let mut ids = CopyIds(self.new_ids());
        let copy = sequence_version(source, name, target, adaptation, &mut ids)?;
        let copy_id = copy.id;
        let add = AddSequence {
            index,
            sequence: copy,
        };
        self.commit_user_edit(&handle, label, vec![add.into()])
            .await?;
        Ok(copy_id)
    }

    /// Removes a sequence and everything in it.
    ///
    /// # Errors
    /// Not open, unknown sequence, the project's only sequence (`LAST_SEQUENCE`), a locked track
    /// in it, storage errors.
    pub async fn remove_sequence(
        &self,
        project_id: ProjectId,
        sequence_id: SequenceId,
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        find_sequence(&view.project, sequence_id)?;
        if view.project.sequences.len() <= 1 {
            return Err(DomainError::LastSequence {
                sequence: sequence_id,
            }
            .into());
        }
        let remove = RemoveSequence {
            sequence: sequence_id,
        };
        self.commit_user_edit(&handle, sequence_labels::REMOVE, vec![remove.into()])
            .await
    }
}

/// The ids of a copied sequence, from the service's id generator.
/// Ids of a sequence copy from the service's generator (also used by agents' versions).
pub(crate) struct CopyIds<'a>(pub(crate) NewIds<'a>);

impl SequenceIds for CopyIds<'_> {
    type Error = AppError;

    fn sequence(&mut self) -> Result<SequenceId, AppError> {
        self.0.sequence()
    }

    fn track(&mut self) -> Result<TrackId, AppError> {
        self.0.track()
    }

    fn item(&mut self) -> Result<ItemId, AppError> {
        self.0.item()
    }

    fn transition(&mut self) -> Result<TransitionId, AppError> {
        self.0.transition()
    }

    fn marker(&mut self) -> Result<MarkerId, AppError> {
        self.0.marker()
    }

    fn link_group(&mut self) -> Result<LinkGroupId, AppError> {
        self.0.link_group()
    }
}
