//! The project cache folder: its size, and cleaning it (every derived file is deleted, then
//! queued again as needed; analyses live in the project database and stay).

use se_domain::ProjectId;
use se_protocol::ProjectCacheInfo;

use super::{MediaService, import::display, media_paths::project_cache_dir, scheduling::Trigger};
use crate::AppError;

impl MediaService {
    /// The cache folder of an open project and the bytes it uses.
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`].
    pub async fn project_cache(&self, project_id: ProjectId) -> Result<ProjectCacheInfo, AppError> {
        let handle = self.handle(project_id)?;
        let dir = project_cache_dir(handle.path());
        Ok(ProjectCacheInfo {
            path: display(&dir),
            size_bytes: self.deps.files.directory_size(&dir).await,
        })
    }

    /// Stops the project's jobs, deletes its cache folder (thumbnails, peaks, mezzanines, proxies,
    /// frames) and queues again what its assets need. Files in use cannot be deleted on Windows:
    /// they stay (and stay usable); the returned size counts them.
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`], [`AppError::ActorStopped`].
    pub async fn clear_project_cache(
        &self,
        project_id: ProjectId,
    ) -> Result<ProjectCacheInfo, AppError> {
        let handle = self.handle(project_id)?;
        self.deps.jobs.cancel_project(project_id).await;
        let dir = project_cache_dir(handle.path());
        if let Err(error) = self.deps.files.remove_directory(&dir).await {
            tracing::warn!(%error, "some cache files could not be deleted");
        }
        let project = handle.view().await?.project;
        for asset in &project.assets {
            self.schedule_asset(&handle, &project, asset, Trigger::Opening)
                .await;
        }
        self.project_cache(project_id).await
    }
}
