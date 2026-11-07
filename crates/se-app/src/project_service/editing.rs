//! Edits of open projects: sequences, undo and redo, and the commit of user edits.

use se_domain::{Actor, AddSequence, Format, Operation, ProjectId, SequenceId, Transaction};

use super::{ProjectService, checked_name};
use crate::{
    AppError, new_ids::NewIds, new_project::default_sequence, project_actor::ProjectActorHandle,
};

/// Label of the transaction adding a sequence (undo menu). A stable identifier: the renderer
/// translates it (`apps/desktop/src/app/editor/history-label.ts`).
const CREATE_SEQUENCE_LABEL: &str = "Create sequence";

impl ProjectService {
    /// Adds a sequence with tracks V1, V2, A1, A2 (a user transaction, undoable). `format`
    /// defaults to the project's default format.
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
        let ids = NewIds(self.deps.ids.as_ref());
        let sequence = default_sequence(ids, &name, format)?;
        let sequence_id = sequence.id;
        let transaction = Transaction {
            id: ids.transaction()?,
            actor: Actor::User,
            label: CREATE_SEQUENCE_LABEL.to_owned(),
            ops: vec![
                AddSequence {
                    index: None,
                    sequence,
                }
                .into(),
            ],
        };
        handle.apply(transaction).await?;
        Ok(sequence_id)
    }

    /// Commits `ops` as one user transaction labelled `label` (one undo step).
    ///
    /// # Errors
    /// The document refuses the operations, storage errors.
    pub(super) async fn commit_user_edit(
        &self,
        handle: &ProjectActorHandle,
        label: &str,
        ops: Vec<Operation>,
    ) -> Result<(), AppError> {
        let transaction = Transaction {
            id: NewIds(self.deps.ids.as_ref()).transaction()?,
            actor: Actor::User,
            label: label.to_owned(),
            ops,
        };
        handle.apply(transaction).await.map(drop)
    }

    /// Undoes the latest user transaction of an open project.
    ///
    /// # Errors
    /// Not open, nothing to undo, the document refuses the undo, storage errors.
    pub async fn undo(&self, project_id: ProjectId) -> Result<(), AppError> {
        self.handle(project_id).await?.undo().await.map(drop)
    }

    /// Redoes the latest undone transaction of an open project.
    ///
    /// # Errors
    /// Not open, nothing to redo, the document refuses the redo, storage errors.
    pub async fn redo(&self, project_id: ProjectId) -> Result<(), AppError> {
        self.handle(project_id).await?.redo().await.map(drop)
    }
}
