//! [`ProjectObserver`]: services that follow the life of open projects (the media service checks
//! missing files at opening and stops a project's jobs at closing).

use async_trait::async_trait;
use se_domain::ProjectId;

use super::ProjectService;
use crate::project_actor::ProjectActorHandle;

/// Told when a project opens or is about to close. Calls happen while the service's registry
/// is locked: an observer must never call back into [`ProjectService`].
#[async_trait]
pub trait ProjectObserver: Send + Sync {
    /// A project was opened (not called again when an open project is re-opened). Runs before
    /// the session is returned to the client, so commits made here are part of it.
    async fn project_opened(&self, handle: &ProjectActorHandle);

    /// A project is about to be saved and closed.
    async fn project_closing(&self, project_id: ProjectId);
}

impl ProjectService {
    /// The same service, also telling `observer` about openings and closings.
    #[must_use]
    pub fn with_observer(mut self, observer: std::sync::Arc<dyn ProjectObserver>) -> Self {
        self.observers.push(observer);
        self
    }

    pub(super) async fn notify_opened(&self, handle: &ProjectActorHandle) {
        for observer in &self.observers {
            observer.project_opened(handle).await;
        }
    }

    /// Tells the observers, then saves and closes the project; returns the close result.
    pub(super) async fn close_handle(
        &self,
        handle: &ProjectActorHandle,
    ) -> Result<(), crate::AppError> {
        for observer in &self.observers {
            observer.project_closing(handle.project_id()).await;
        }
        handle.close().await
    }
}
