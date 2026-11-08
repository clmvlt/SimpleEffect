//! Creating, opening and closing projects.

use std::{path::Path, sync::Arc};

use se_domain::{Format, FormatPreset, FrameRate, Project, ProjectId, validate_project};
use se_protocol::{CoreEvent, FormatPresetDimensions, NewProjectDefaults, ProjectSession};

use super::{ProjectService, checked_name, display_path, loading::session_of};
use crate::{
    AppError, new_ids::NewIds, new_project::new_project, ports::RecentProjectRecord,
    project_actor::ProjectActorHandle,
};

/// Setting holding the folder the last project was created in.
pub(super) const LAST_PARENT_DIR_SETTING: &str = "projects.lastParentDir";

/// Frame rate proposed for new projects.
const DEFAULT_FRAME_RATE: FrameRate = FrameRate::FPS_30;

impl ProjectService {
    /// Choices for the « new project » dialog.
    ///
    /// # Errors
    /// None today (a library failure falls back to the default folder); kept fallible for
    /// future settings.
    pub async fn new_project_defaults(&self) -> Result<NewProjectDefaults, AppError> {
        let remembered = match self.deps.library.setting(LAST_PARENT_DIR_SETTING).await {
            Ok(value) => value,
            Err(error) => {
                tracing::warn!(%error, "reading the last project folder failed");
                None
            }
        };
        let presets = FormatPreset::NAMED
            .into_iter()
            .filter_map(|preset| {
                preset
                    .dimensions()
                    .map(|(width, height)| FormatPresetDimensions {
                        preset,
                        width,
                        height,
                    })
            })
            .collect();
        let format = Format::from_preset(FormatPreset::Hd16x9, DEFAULT_FRAME_RATE)
            .ok_or_else(|| AppError::Internal("the 16:9 preset has no dimensions".to_owned()))?;
        Ok(NewProjectDefaults {
            parent_dir: remembered.unwrap_or_else(|| display_path(&self.config.default_parent_dir)),
            format,
            presets,
            frame_rates: FrameRate::STANDARD.to_vec(),
        })
    }

    /// Creates `<parent_dir>/<name>.seproj` with a default sequence, then opens it.
    ///
    /// # Errors
    /// Empty name, invalid format, existing folder, storage errors.
    pub async fn create_project(
        &self,
        name: &str,
        parent_dir: &Path,
        format: Format,
    ) -> Result<ProjectSession, AppError> {
        let name = checked_name("project", name)?;
        let project = new_project(NewIds(self.deps.ids.as_ref()), &name, format)?;
        let violations = validate_project(&project);
        if !violations.is_empty() {
            let details = violations
                .iter()
                .map(|violation| violation.message.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(AppError::InvalidFormat { details });
        }
        let mut open = self.open.lock().await;
        let path = self.deps.repository.create(parent_dir, &project).await?;
        let parent = display_path(parent_dir);
        if let Err(error) = self
            .deps
            .library
            .set_setting(LAST_PARENT_DIR_SETTING, &parent)
            .await
        {
            tracing::warn!(%error, "remembering the project folder failed");
        }
        self.open_locked(&mut open, &path).await
    }

    /// Opens the project folder at `path` (or returns the session of the already open project).
    ///
    /// # Errors
    /// Not found, locked by another process, newer format, unreadable files.
    pub async fn open_project(&self, path: &Path) -> Result<ProjectSession, AppError> {
        let path = self.deps.repository.locate(path).await?;
        let mut open = self.open.lock().await;
        self.open_locked(&mut open, &path).await
    }

    async fn open_locked(
        &self,
        open: &mut Vec<ProjectActorHandle>,
        path: &Path,
    ) -> Result<ProjectSession, AppError> {
        let handle = if let Some(handle) = open.iter().find(|handle| handle.path() == path) {
            handle.clone()
        } else {
            let handle = self.start_actor(path, true).await?;
            if let Some(other) = open
                .iter()
                .find(|other| other.project_id() == handle.project_id())
            {
                let error = AppError::SameProjectOpen {
                    project_id: handle.project_id(),
                    path: path.to_path_buf(),
                    open_path: other.path().to_path_buf(),
                };
                if let Err(close_error) = handle.close().await {
                    tracing::warn!(%close_error, "closing a duplicate project failed");
                }
                return Err(error);
            }
            open.push(handle.clone());
            self.notify_opened(&handle).await;
            handle
        };
        let session = session_of(&handle).await?;
        self.remember_opened(path, &session).await;
        self.deps
            .events
            .publish(CoreEvent::ProjectOpened(session.clone()));
        Ok(session)
    }

    /// Puts the project first in the recent list.
    async fn remember_opened(&self, path: &Path, session: &ProjectSession) {
        let header = &session.snapshot.project;
        let record = RecentProjectRecord {
            path: path.to_path_buf(),
            id: header.id,
            name: header.name.clone(),
            format: header.settings.default_format,
            last_opened_at: self.deps.clock.now_unix_millis(),
        };
        if let Err(error) = self.deps.library.upsert_recent(&record).await {
            tracing::warn!(%error, "updating the recent projects failed");
        }
    }

    /// Saves and closes an open project.
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`]; the save error (the project is closed anyway and its journal
    /// keeps every commit).
    pub async fn close_project(&self, project_id: ProjectId) -> Result<(), AppError> {
        let handle = {
            let mut open = self.open.lock().await;
            let index = open
                .iter()
                .position(|handle| handle.project_id() == project_id)
                .ok_or(AppError::ProjectNotOpen { project_id })?;
            open.remove(index)
        };
        let closed = self.close_handle(&handle).await;
        self.deps
            .events
            .publish(CoreEvent::ProjectClosed { project_id });
        closed
    }

    /// Closes every open project (when the core shuts down), logging failures.
    pub async fn close_all(&self) {
        let handles = std::mem::take(&mut *self.open.lock().await);
        for handle in handles {
            let project_id = handle.project_id();
            if let Err(error) = self.close_handle(&handle).await {
                tracing::error!(%error, %project_id, "closing a project at shutdown failed");
            }
            self.deps
                .events
                .publish(CoreEvent::ProjectClosed { project_id });
        }
    }

    /// The current session of an open project.
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`].
    pub async fn project_session(&self, project_id: ProjectId) -> Result<ProjectSession, AppError> {
        session_of(&self.handle(project_id).await?).await
    }

    /// The current document of an open project (read-only: AI read tools, prompt 18).
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`].
    pub async fn project(&self, project_id: ProjectId) -> Result<Arc<Project>, AppError> {
        Ok(self.handle(project_id).await?.view().await?.project)
    }
}
