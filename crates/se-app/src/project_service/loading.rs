//! Starting actors: locking a folder, loading and recovering its document, building sessions.

use std::{path::Path, sync::Arc};

use se_domain::{Changeset, Project, validate_project};
use se_protocol::{CoreEvent, ProjectRecovery, ProjectSession};

use super::{ProjectService, display_path};
use crate::{
    AppError,
    ports::{EventSink, JournalEntry, ProjectStore, RestoredSnapshot},
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
            Ok(Loaded {
                project,
                changesets,
                recovery,
            }) => {
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
                Ok(ProjectActorHandle::spawn(project, changesets, store, deps)
                    .with_recovery(recovery))
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

/// What [`load`] gives an actor.
struct Loaded {
    project: Project,
    changesets: Vec<Changeset>,
    recovery: Option<ProjectRecovery>,
}

/// The document of an open store — its snapshot (or its newest readable backup) plus the journal
/// written after it (crash recovery) — and the open changesets of its AI tasks. A recovered
/// state is saved as the new snapshot right away.
async fn load(store: &dyn ProjectStore) -> Result<Loaded, AppError> {
    let loaded = store.load_snapshot().await?;
    let snapshot = loaded.project;
    check_valid(store, &snapshot)?;
    let journal = store.read_journal_since(snapshot.revision).await?;
    let (project, lost_changes) =
        replay_journal(store, snapshot, &journal, loaded.restored.as_ref())?;
    let replayed_changes = journal.len().saturating_sub(lost_changes);
    let recovery = recovery_report(replayed_changes, lost_changes, loaded.restored);
    if recovery.is_some() {
        tracing::info!(
            entries = replayed_changes,
            lost = lost_changes,
            revision = project.revision,
            "recovered the project after an unexpected stop"
        );
        if let Err(error) = store.save_snapshot(&project).await {
            // The journal still holds the commits: they will be replayed again next time.
            tracing::warn!(%error, "saving the recovered snapshot failed");
        }
    }
    let changesets = store.load_changesets().await?;
    Ok(Loaded {
        project,
        changesets,
        recovery,
    })
}

/// Refuses a snapshot breaking the document's invariants.
fn check_valid(store: &dyn ProjectStore, snapshot: &Project) -> Result<(), AppError> {
    let violations = validate_project(snapshot);
    if violations.is_empty() {
        return Ok(());
    }
    let details = violations
        .iter()
        .map(|violation| violation.message.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    Err(AppError::InvalidDocument {
        path: store.path().to_path_buf(),
        details,
    })
}

/// Replays the journal on the snapshot; returns the project and how many entries were lost.
///
/// On `project.json` itself a broken journal is an error (nothing may be silently dropped). On
/// a restored backup the journal may not join it (an older backup, written before the journal
/// kept its commits): the backup is then used alone and the entries counted as lost.
fn replay_journal(
    store: &dyn ProjectStore,
    snapshot: Project,
    journal: &[JournalEntry],
    restored: Option<&RestoredSnapshot>,
) -> Result<(Project, usize), AppError> {
    match replay(snapshot.clone(), journal) {
        Ok(project) => Ok((project, 0)),
        Err(reason) if restored.is_some() => {
            tracing::warn!(%reason, "the journal does not apply to the restored backup");
            Ok((snapshot, journal.len()))
        }
        Err(reason) => Err(AppError::JournalBroken {
            path: store.path().to_path_buf(),
            reason,
        }),
    }
}

/// What clients are told about this opening; `None` when nothing had to be recovered.
fn recovery_report(
    replayed_changes: usize,
    lost_changes: usize,
    restored: Option<RestoredSnapshot>,
) -> Option<ProjectRecovery> {
    if replayed_changes == 0 && lost_changes == 0 && restored.is_none() {
        return None;
    }
    let count = |value: usize| u32::try_from(value).unwrap_or(u32::MAX);
    let (restored_from, problem) = restored.map_or((None, None), |restored| {
        (Some(restored.backup), Some(restored.problem))
    });
    Some(ProjectRecovery {
        replayed_changes: count(replayed_changes),
        restored_from,
        problem,
        lost_changes: count(lost_changes),
    })
}

/// The session clients receive for an open project.
pub(super) async fn session_of(handle: &ProjectActorHandle) -> Result<ProjectSession, AppError> {
    let view = handle.view().await?;
    Ok(ProjectSession {
        path: display_path(handle.path()),
        snapshot: project_snapshot(&view.project),
        history: view.history,
        recovery: handle.recovery().cloned(),
    })
}
