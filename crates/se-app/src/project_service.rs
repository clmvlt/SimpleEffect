//! [`ProjectService`]: the project use cases behind the protocol commands (create, open, close,
//! recent list, rename, duplicate, trash, sequences, tracks, markers, undo/redo).
//!
//! It keeps the registry of open projects (one [`ProjectActorHandle`] each) and never mutates a
//! document itself: every change is a transaction sent to the project's actor. Operations on a
//! project that is not open (renaming it from the home screen) start a temporary actor whose
//! events are discarded.

mod editing;
mod library;
mod lifecycle;
mod loading;
mod markers;
mod observer;
mod tracks;

pub use markers::labels as marker_labels;
pub use observer::ProjectObserver;
pub use tracks::labels as track_labels;

use std::{path::PathBuf, sync::Arc, time::Duration};

use se_domain::ProjectId;
use tokio::sync::Mutex;

use crate::{
    AppError,
    ports::{AppLibraryRepository, Clock, EventSink, IdGenerator, ProjectRepository},
    project_actor::ProjectActorHandle,
};

/// The ports the service works with.
#[derive(Clone)]
pub struct ProjectServiceDeps {
    /// Project folders.
    pub repository: Arc<dyn ProjectRepository>,
    /// Recent projects and settings.
    pub library: Arc<dyn AppLibraryRepository>,
    /// Events to clients.
    pub events: Arc<dyn EventSink>,
    /// Timestamps.
    pub clock: Arc<dyn Clock>,
    /// New ids.
    pub ids: Arc<dyn IdGenerator>,
}

/// Settings of the service chosen by the composition root.
#[derive(Debug, Clone)]
pub struct ProjectServiceConfig {
    /// Folder proposed for new projects until the user picks one (`<Documents>/SimpleEffect`).
    pub default_parent_dir: PathBuf,
    /// Snapshot delay after the last commit (see `project_actor::DEFAULT_SAVE_DELAY`).
    pub save_delay: Duration,
}

/// The project use cases.
pub struct ProjectService {
    deps: ProjectServiceDeps,
    config: ProjectServiceConfig,
    /// Open projects. Held for the whole of open/close/rename/duplicate/trash so two commands
    /// never race on the same folder.
    open: Mutex<Vec<ProjectActorHandle>>,
    /// Told about openings and closings (see [`ProjectObserver`]).
    observers: Vec<Arc<dyn ProjectObserver>>,
}

impl ProjectService {
    /// A service with no project open.
    #[must_use]
    pub fn new(deps: ProjectServiceDeps, config: ProjectServiceConfig) -> Self {
        Self {
            deps,
            config,
            open: Mutex::new(Vec::new()),
            observers: Vec::new(),
        }
    }

    /// The actor of an open project.
    async fn handle(&self, project_id: ProjectId) -> Result<ProjectActorHandle, AppError> {
        self.open
            .lock()
            .await
            .iter()
            .find(|handle| handle.project_id() == project_id)
            .cloned()
            .ok_or(AppError::ProjectNotOpen { project_id })
    }
}

/// `name` without surrounding spaces, refused when empty.
fn checked_name(what: &'static str, name: &str) -> Result<String, AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::EmptyName { what });
    }
    Ok(trimmed.to_owned())
}

/// A path as sent to clients. Non-UTF-8 characters are replaced (display only).
fn display_path(path: &std::path::Path) -> String {
    path.to_string_lossy().into_owned()
}
