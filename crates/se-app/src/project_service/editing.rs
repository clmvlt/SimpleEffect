//! Edits of open projects: undo and redo, and the commit of user edits.

use se_domain::{Actor, Operation, ProjectId, Transaction};

use super::ProjectService;
use crate::{AppError, edit_plan::EditPlan, new_ids::NewIds, project_actor::ProjectActorHandle};

impl ProjectService {
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

    /// Commits the operations of `plan` as one user transaction labelled `label`; a plan that
    /// changes nothing commits nothing.
    ///
    /// # Errors
    /// The document refuses the operations, storage errors.
    pub(super) async fn commit_plan(
        &self,
        handle: &ProjectActorHandle,
        label: &str,
        plan: EditPlan<'_>,
    ) -> Result<(), AppError> {
        let ops = plan.finish()?;
        if ops.is_empty() {
            return Ok(());
        }
        self.commit_user_edit(handle, label, ops).await
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
