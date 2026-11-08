//! Starting actors: locking a folder, loading and recovering its document, building sessions.

use std::{path::Path, sync::Arc};

use se_domain::{Changeset, Project, validate_project};
use se_protocol::{CoreEvent, ProjectSession};

use super::{ProjectService, display_path};
use crate::{
    AppError,
    ports::{EventSink, ProjectStore},
    project_actor::{ActorDeps, ProjectActorHandle},
    project_view::project_snapshot,
    recovery::replay,
};

/// An event sink dropping everything: temporary actors (rename or duplicate a closed project)
/// have no client watching them.
struct DiscardEvents;

impl EventSink for DiscardEvents {
    fn publish(&self, _event: CoreEvent) {}
}

impl ProjectService {
    /// Locks the folder at `path`, recovers its document and starts its actor, publishing to
    /// clients (`announce`) or to nobody.
    pub(super) async fn start_actor(
        &self,
        path: &Path,
        announce: bool,
    ) -> Result<ProjectActorHandle, AppError> {
        let store = self.deps.repository.open(path).await?;
        match load(store.as_ref()).await {
            Ok((project, changesets)) => {
                let events: Arc<dyn EventSink> = if announce {
                    Arc::clone(&self.deps.events)
                } else {
                    Arc::new(DiscardEvents)
                };
                let deps = ActorDeps {
                    events,
                    clock: Arc::clone(&self.deps.clock),
                    ids: Arc::clone(&self.deps.ids),
                    save_delay: self.config.save_delay,
                };
                Ok(ProjectActorHandle::spawn(project, changesets, store, deps))
            }
            Err(error) => {
                if let Err(close_error) = store.close().await {
                    tracing::warn!(%close_error, "releasing a project that failed to load");
                }
                Err(error)
            }
        }
    }
}

/// The document of an open store — its snapshot plus the journal written after it (crash
/// recovery) — and the open changesets of its AI tasks. A recovered state is saved as the new
/// snapshot right away.
async fn load(store: &dyn ProjectStore) -> Result<(Project, Vec<Changeset>), AppError> {
    let snapshot = store.load_snapshot().await?;
    let violations = validate_project(&snapshot);
    if !violations.is_empty() {
        let details = violations
            .iter()
            .map(|violation| violation.message.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(AppError::InvalidDocument {
            path: store.path().to_path_buf(),
            details,
        });
    }
    let journal = store.read_journal_since(snapshot.revision).await?;
    let project = replay(snapshot, &journal).map_err(|reason| AppError::JournalBroken {
        path: store.path().to_path_buf(),
        reason,
    })?;
    if !journal.is_empty() {
        tracing::info!(
            entries = journal.len(),
            revision = project.revision,
            "recovered commits from the journal"
        );
        if let Err(error) = store.save_snapshot(&project).await {
            // The journal still holds the commits: they will be replayed again next time.
            tracing::warn!(%error, "saving the recovered snapshot failed");
        }
    }
    let changesets = store.load_changesets().await?;
    Ok((project, changesets))
}

/// The session clients receive for an open project.
pub(super) async fn session_of(handle: &ProjectActorHandle) -> Result<ProjectSession, AppError> {
    let view = handle.view().await?;
    Ok(ProjectSession {
        path: display_path(handle.path()),
        snapshot: project_snapshot(&view.project),
        history: view.history,
    })
}
