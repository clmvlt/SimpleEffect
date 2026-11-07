//! The home screen's use cases: recent list, rename, duplicate, trash.

use std::{path::Path, sync::Arc};

use se_domain::{Actor, Project, RenameProject, Transaction};
use se_protocol::{CoreEvent, RecentProject};

use super::{ProjectService, checked_name, display_path};
use crate::{
    AppError, new_ids::NewIds, ports::RecentProjectRecord, ports::StorageError,
    project_actor::ProjectActorHandle,
};

/// Label of the transaction renaming a project (undo menu).
const RENAME_LABEL: &str = "Rename project";

impl ProjectService {
    /// The recent projects, most recently opened first, each saying whether its folder is
    /// still there.
    ///
    /// # Errors
    /// Library storage errors.
    pub async fn list_recent(&self) -> Result<Vec<RecentProject>, AppError> {
        let records = self.deps.library.list_recent().await?;
        let mut projects = Vec::with_capacity(records.len());
        for record in records {
            let available = self.deps.repository.locate(&record.path).await.is_ok();
            projects.push(recent_project(&record, available));
        }
        Ok(projects)
    }

    /// Renames the project at `path`, open or not (a user transaction, journaled). The folder
    /// keeps its name.
    ///
    /// # Errors
    /// Empty name, not found, locked by another process, storage errors.
    pub async fn rename_project(&self, path: &Path, name: &str) -> Result<RecentProject, AppError> {
        let name = checked_name("project", name)?;
        let path = self.deps.repository.locate(path).await?;
        let open = self.open.lock().await;
        let transaction = Transaction {
            id: NewIds(self.deps.ids.as_ref()).transaction()?,
            actor: Actor::User,
            label: RENAME_LABEL.to_owned(),
            ops: vec![RenameProject { name }.into()],
        };
        let project = self
            .with_project(&open, &path, async |handle| {
                handle.apply(transaction).await?;
                Ok(handle.view().await?.project)
            })
            .await?;
        Ok(self.refresh_recent(&path, &project).await)
    }

    /// Copies the project at `path` into a new folder next to it, with a new id and `name`, and
    /// puts the copy first in the recent list.
    ///
    /// # Errors
    /// Empty name, not found, locked by another process, storage errors.
    pub async fn duplicate_project(
        &self,
        path: &Path,
        name: &str,
    ) -> Result<RecentProject, AppError> {
        let name = checked_name("project", name)?;
        let path = self.deps.repository.locate(path).await?;
        let open = self.open.lock().await;
        let source = self
            .with_project(&open, &path, async |handle| {
                Ok(handle.view().await?.project)
            })
            .await?;
        let copy = Project {
            id: NewIds(self.deps.ids.as_ref()).project()?,
            name,
            ..Project::clone(&source)
        };
        let copy_path = self.deps.repository.duplicate(&path, &copy).await?;
        let record = RecentProjectRecord {
            path: copy_path,
            id: copy.id,
            name: copy.name.clone(),
            format: copy.settings.default_format,
            last_opened_at: self.deps.clock.now_unix_millis(),
        };
        if let Err(error) = self.deps.library.upsert_recent(&record).await {
            tracing::warn!(%error, "adding the copy to the recent projects failed");
        }
        Ok(recent_project(&record, true))
    }

    /// Closes the project at `path` if it is open, moves its folder to the OS trash and forgets
    /// it. A folder that is already gone is only forgotten.
    ///
    /// # Errors
    /// Storage errors (e.g. no trash on that drive).
    pub async fn trash_project(&self, path: &Path) -> Result<(), AppError> {
        let path = match self.deps.repository.locate(path).await {
            Ok(path) => path,
            Err(StorageError::NotFound { .. }) => {
                self.deps.library.remove_recent(path).await?;
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };
        let mut open = self.open.lock().await;
        if let Some(index) = open.iter().position(|handle| handle.path() == path) {
            let handle = open.remove(index);
            if let Err(error) = handle.close().await {
                tracing::warn!(%error, "saving a project before trashing it failed");
            }
            self.deps.events.publish(CoreEvent::ProjectClosed {
                project_id: handle.project_id(),
            });
        }
        self.deps.repository.trash(&path).await?;
        if let Err(error) = self.deps.library.remove_recent(&path).await {
            tracing::warn!(%error, "removing a trashed project from the recent list failed");
        }
        Ok(())
    }

    /// Runs `work` with the actor of the project at `path`: the open one, or a temporary one
    /// (events discarded) closed afterwards.
    async fn with_project<T>(
        &self,
        open: &[ProjectActorHandle],
        path: &Path,
        work: impl AsyncFnOnce(&ProjectActorHandle) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        if let Some(handle) = open.iter().find(|handle| handle.path() == path) {
            return work(handle).await;
        }
        let handle = self.start_actor(path, false).await?;
        let result = work(&handle).await;
        let closed = handle.close().await;
        let value = result?;
        closed?;
        Ok(value)
    }

    /// Updates the name and format of a recent entry, keeping its date.
    async fn refresh_recent(&self, path: &Path, project: &Arc<Project>) -> RecentProject {
        let last_opened_at = match self.deps.library.list_recent().await {
            Ok(records) => records
                .into_iter()
                .find(|record| record.path == path)
                .map(|record| record.last_opened_at),
            Err(error) => {
                tracing::warn!(%error, "reading the recent projects failed");
                None
            }
        };
        let record = RecentProjectRecord {
            path: path.to_path_buf(),
            id: project.id,
            name: project.name.clone(),
            format: project.settings.default_format,
            last_opened_at: last_opened_at.unwrap_or_else(|| self.deps.clock.now_unix_millis()),
        };
        if let Err(error) = self.deps.library.upsert_recent(&record).await {
            tracing::warn!(%error, "updating the recent projects failed");
        }
        recent_project(&record, true)
    }
}

fn recent_project(record: &RecentProjectRecord, available: bool) -> RecentProject {
    RecentProject {
        id: record.id,
        name: record.name.clone(),
        path: display_path(&record.path),
        format: record.format,
        last_opened_at: record.last_opened_at,
        available,
    }
}
