//! The actor task: its loop, the commit flow and the debounced snapshot.

use std::{sync::Arc, time::Duration};

use se_domain::{Changeset, Committed, History, Project, Transaction, commit};
use se_protocol::{CoreEvent, ProjectHistory, ProjectSaveState};
use tokio::{
    sync::mpsc,
    time::{Instant, sleep_until},
};

use super::{ActorDeps, Message, ProjectView, ai_edits::OpenChangesets};
use crate::{
    AppError,
    new_ids::NewIds,
    ports::{ChangesetWrite, JournalEntry, ProjectStore},
    project_view::{history_state, project_patch},
};

/// The longest a commit waits for its snapshot while commits keep coming (each one would
/// otherwise push the debounced save back: an agent editing every second never got saved).
pub(super) const MAX_SAVE_WAIT: Duration = Duration::from_secs(30);

/// Unsaved commits after which the snapshot is written at once (keeps the journal and the
/// replay at the next opening short).
pub(super) const MAX_UNSAVED_COMMITS: u64 = 200;

/// State owned by the actor task.
pub(super) struct ProjectActor {
    pub(super) project: Arc<Project>,
    pub(super) history: History,
    store: Box<dyn ProjectStore>,
    pub(super) deps: ActorDeps,
    /// The open changesets of AI tasks.
    pub(super) changesets: OpenChangesets,
    /// Revision of the last snapshot written (or loaded).
    saved_revision: u64,
    /// When the next snapshot is due; `None` when nothing waits to be saved.
    save_due: Option<Instant>,
    /// When the oldest commit not in the snapshot was installed.
    unsaved_since: Option<Instant>,
    /// Whether the last snapshot write failed (clients were told).
    save_failing: bool,
}

/// Runs the actor until it is closed or every handle is dropped.
pub(super) async fn run(
    project: Project,
    changesets: Vec<Changeset>,
    store: Box<dyn ProjectStore>,
    deps: ActorDeps,
    mut inbox: mpsc::Receiver<Message>,
) {
    let mut actor = ProjectActor {
        saved_revision: project.revision,
        project: Arc::new(project),
        history: History::default(),
        store,
        deps,
        changesets: OpenChangesets::new(changesets),
        save_due: None,
        unsaved_since: None,
        save_failing: false,
    };
    loop {
        tokio::select! {
            message = inbox.recv() => match message {
                Some(Message::Close { reply }) => {
                    let _ = reply.send(actor.shut_down().await);
                    return;
                }
                Some(message) => actor.handle(message).await,
                None => {
                    if let Err(error) = actor.shut_down().await {
                        tracing::error!(%error, "closing an abandoned project failed");
                    }
                    return;
                }
            },
            () = wait_until(actor.save_due) => actor.save_snapshot().await,
        }
    }
}

/// Resolves at `deadline`, or never.
async fn wait_until(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => sleep_until(deadline).await,
        None => std::future::pending().await,
    }
}

impl ProjectActor {
    async fn handle(&mut self, message: Message) {
        // A dropped reply channel means the caller gave up: the work is done anyway.
        match message {
            Message::Apply { transaction, reply } => {
                let _ = reply.send(self.apply(transaction).await);
            }
            Message::Undo { reply } => {
                let _ = reply.send(self.step_history(History::undo).await);
            }
            Message::Redo { reply } => {
                let _ = reply.send(self.step_history(History::redo).await);
            }
            Message::ApplyAi {
                transaction,
                base_revision,
                reply,
            } => {
                let outcome = if base_revision.is_some_and(|base| base != self.project.revision) {
                    Ok(None)
                } else {
                    self.apply_ai(transaction).await.map(Some)
                };
                let _ = reply.send(outcome);
            }
            Message::RevertTask {
                task_id,
                label,
                reply,
            } => {
                let _ = reply.send(self.revert_task(task_id, label).await);
            }
            Message::Changeset { task_id, reply } => {
                let _ = reply.send(self.changesets.get(task_id).cloned());
            }
            Message::PreviewRevert { task_id, reply } => {
                let _ = reply.send(self.preview_revert(task_id));
            }
            Message::ChangesetItems { task_id, reply } => {
                let _ = reply.send(self.changeset_items(task_id));
            }
            Message::Snapshot { reply } => {
                let _ = reply.send(ProjectView {
                    project: Arc::clone(&self.project),
                    history: history_state(&self.history),
                });
            }
            Message::Close { .. } => {}
        }
    }

    async fn apply(&mut self, transaction: Transaction) -> Result<u64, AppError> {
        let committed = commit(&self.project, &transaction)?;
        self.journal(&transaction, &committed).await?;
        self.history.record(&transaction, &committed);
        Ok(self.install(committed))
    }

    /// Undo or redo: the history commits the stored operations as a new user transaction.
    async fn step_history(
        &mut self,
        step: fn(
            &mut History,
            &Project,
            se_domain::TransactionId,
        ) -> Result<se_domain::HistoryStep, se_domain::DomainError>,
    ) -> Result<u64, AppError> {
        let id = NewIds(self.deps.ids.as_ref()).transaction()?;
        let previous = self.history.clone();
        let step = step(&mut self.history, &self.project, id)?;
        if let Err(error) = self.journal(&step.transaction, &step.committed).await {
            // Not journaled, so not committed: the history must not have moved either.
            self.history = previous;
            return Err(error);
        }
        Ok(self.install(step.committed))
    }

    /// Durably records a commit before it becomes visible.
    async fn journal(
        &self,
        transaction: &Transaction,
        committed: &Committed,
    ) -> Result<(), AppError> {
        let entry = self.journal_entry(transaction, committed);
        self.store.append_journal(&entry).await?;
        Ok(())
    }

    /// Durably records a commit and its change of an AI changeset, atomically, before the commit
    /// becomes visible.
    pub(super) async fn journal_with_changeset(
        &self,
        transaction: &Transaction,
        committed: &Committed,
        change: &ChangesetWrite,
    ) -> Result<(), AppError> {
        let entry = self.journal_entry(transaction, committed);
        self.store
            .append_journal_with_changeset(&entry, change)
            .await?;
        Ok(())
    }

    fn journal_entry(&self, transaction: &Transaction, committed: &Committed) -> JournalEntry {
        JournalEntry {
            revision: committed.project.revision,
            committed_at: self.deps.clock.now_unix_millis(),
            transaction: transaction.clone(),
        }
    }

    /// Makes a journaled commit the current state, announces it and schedules the snapshot.
    pub(super) fn install(&mut self, committed: Committed) -> u64 {
        let before = std::mem::replace(&mut self.project, Arc::new(committed.project));
        let actor = committed.inverse.actor;
        let patch = project_patch(&before, &self.project, &committed.touched, actor);
        self.deps.events.publish(CoreEvent::ProjectPatched(patch));
        self.deps
            .events
            .publish(CoreEvent::HistoryChanged(ProjectHistory {
                project_id: self.project.id,
                history: history_state(&self.history),
            }));
        self.schedule_snapshot();
        self.project.revision
    }

    /// Debounces the snapshot after a commit, bounded by [`MAX_SAVE_WAIT`] since the oldest
    /// unsaved commit, and immediate after [`MAX_UNSAVED_COMMITS`].
    fn schedule_snapshot(&mut self) {
        let now = Instant::now();
        let oldest = *self.unsaved_since.get_or_insert(now);
        let unsaved = self.project.revision.saturating_sub(self.saved_revision);
        self.save_due = Some(if unsaved >= MAX_UNSAVED_COMMITS {
            now
        } else {
            (now + self.deps.save_delay).min(oldest + MAX_SAVE_WAIT)
        });
    }

    /// Writes the snapshot; on failure retries after another delay (the journal keeps every
    /// commit meanwhile).
    async fn save_snapshot(&mut self) {
        match self.store.save_snapshot(&self.project).await {
            Ok(()) => {
                self.saved_revision = self.project.revision;
                self.save_due = None;
                self.unsaved_since = None;
                tracing::debug!(revision = self.saved_revision, "snapshot saved");
                if std::mem::take(&mut self.save_failing) {
                    self.publish_save_state(None);
                }
            }
            Err(error) => {
                tracing::warn!(%error, "saving the snapshot failed; retrying later");
                self.save_due = Some(Instant::now() + self.deps.save_delay);
                self.save_failing = true;
                self.publish_save_state(Some(error.to_string()));
            }
        }
    }

    fn publish_save_state(&self, failure: Option<String>) {
        let unsaved = self.project.revision.saturating_sub(self.saved_revision);
        self.deps
            .events
            .publish(CoreEvent::ProjectSaveStateChanged(ProjectSaveState {
                project_id: self.project.id,
                failure,
                unsaved_changes: u32::try_from(unsaved).unwrap_or(u32::MAX),
            }));
    }

    /// Saves what is not saved yet, then releases the store.
    async fn shut_down(&mut self) -> Result<(), AppError> {
        let saved = if self.project.revision == self.saved_revision {
            Ok(())
        } else {
            self.store.save_snapshot(&self.project).await
        };
        let released = self.store.close().await;
        saved?;
        released?;
        Ok(())
    }
}
