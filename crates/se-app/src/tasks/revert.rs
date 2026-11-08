//! The changesets of tasks (prompt 19, ADR-0029): the one an agent's first edit opens is
//! recorded on its task, and a finished task can be reverted as one user transaction.

use se_domain::{ChangesetId, ProjectId, TaskId};
use se_protocol::{CoreEvent, TaskRevertReport};

use super::{TaskService, task_state};
use crate::AppError;

/// Label of the user transaction reverting a task. Stable identifier: the renderer translates it
/// (`apps/desktop/src/app/editor/history-label.ts`).
pub const REVERT_TASK_LABEL: &str = "Revert AI task";

impl TaskService {
    /// Records that `task_id`'s edits now go to changeset `changeset_id` (saved, then
    /// `taskUpdated`).
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`], [`AppError::TaskNotFound`], [`AppError::Storage`].
    pub async fn attach_changeset(
        &self,
        project_id: ProjectId,
        task_id: TaskId,
        changeset_id: ChangesetId,
    ) -> Result<(), AppError> {
        self.set_changeset(project_id, task_id, Some(changeset_id))
            .await
    }

    /// Undoes the edits of a finished task that nobody changed since, as one user transaction
    /// (undoable). The report lists what was undone and what was left (conflicts); when something
    /// was undone the task's changeset is closed (`changesetId` becomes `null`).
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`], [`AppError::TaskNotFound`], [`AppError::TaskStillActive`]
    /// (cancel it first), [`AppError::NothingToRevert`], the document's refusal, storage errors.
    pub async fn revert(
        &self,
        project_id: ProjectId,
        task_id: TaskId,
    ) -> Result<TaskRevertReport, AppError> {
        let handle = {
            let mut state = self.shared.state.lock().await;
            let project = state.project_mut(project_id)?;
            let task = project.task_mut(task_id)?;
            if task_state::is_active(&task.status) {
                return Err(AppError::TaskStillActive { task_id });
            }
            project.handle.clone()
        };
        let report = handle.revert_task(task_id, REVERT_TASK_LABEL).await?;
        if report.reverted {
            self.set_changeset(project_id, task_id, None).await?;
        }
        Ok(report)
    }

    async fn set_changeset(
        &self,
        project_id: ProjectId,
        task_id: TaskId,
        changeset_id: Option<ChangesetId>,
    ) -> Result<(), AppError> {
        let now = self.shared.now();
        let mut state = self.shared.state.lock().await;
        let project = state.project_mut(project_id)?;
        let task = project.task_mut(task_id)?;
        if task.changeset_id == changeset_id {
            return Ok(());
        }
        task.changeset_id = changeset_id;
        task.updated_at = now;
        let task = task.clone();
        project.handle.tasks().save_task(&task).await?;
        self.shared
            .deps
            .events
            .publish(CoreEvent::TaskUpdated { task });
        Ok(())
    }
}
