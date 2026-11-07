//! The actor task: its loop, the commit flow and the debounced snapshot.

use std::sync::Arc;

use se_domain::{Committed, History, Project, Transaction, commit};
use se_protocol::{CoreEvent, ProjectHistory};
use tokio::{
    sync::mpsc,
    time::{Instant, sleep_until},
};

use super::{ActorDeps, Message, ProjectView};
use crate::{
    AppError,
    new_ids::NewIds,
    ports::{JournalEntry, ProjectStore},
    project_view::{history_state, project_patch},
};

/// State owned by the actor task.
struct ProjectActor {
    project: Arc<Project>,
    history: History,
    store: Box<dyn ProjectStore>,
    deps: ActorDeps,
    /// Revision of the last snapshot written (or loaded).
    saved_revision: u64,
    /// When the next snapshot is due; `None` when nothing waits to be saved.
    save_due: Option<Instant>,
}

/// Runs the actor until it is closed or every handle is dropped.
pub(super) async fn run(
    project: Project,
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
        save_due: None,
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
        let entry = JournalEntry {
            revision: committed.project.revision,
            committed_at: self.deps.clock.now_unix_millis(),
            transaction: transaction.clone(),
        };
        self.store.append_journal(&entry).await?;
        Ok(())
    }

    /// Makes a journaled commit the current state, announces it and schedules the snapshot.
    fn install(&mut self, committed: Committed) -> u64 {
        let before = std::mem::replace(&mut self.project, Arc::new(committed.project));
        let patch = project_patch(&before, &self.project, &committed.touched);
        self.deps.events.publish(CoreEvent::ProjectPatched(patch));
        self.deps
            .events
            .publish(CoreEvent::HistoryChanged(ProjectHistory {
                project_id: self.project.id,
                history: history_state(&self.history),
            }));
        self.save_due = Some(Instant::now() + self.deps.save_delay);
        self.project.revision
    }

    /// Writes the snapshot; on failure retries after another delay (the journal keeps every
    /// commit meanwhile).
    async fn save_snapshot(&mut self) {
        match self.store.save_snapshot(&self.project).await {
            Ok(()) => {
                self.saved_revision = self.project.revision;
                self.save_due = None;
                tracing::debug!(revision = self.saved_revision, "snapshot saved");
            }
            Err(error) => {
                tracing::warn!(%error, "saving the snapshot failed; retrying later");
                self.save_due = Some(Instant::now() + self.deps.save_delay);
            }
        }
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
