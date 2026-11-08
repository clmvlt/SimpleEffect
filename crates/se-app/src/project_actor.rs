//! [`ProjectActorHandle`]: the single writer of an open project (golden rule 4, ADR-0005,
//! `docs/architecture.md` §5.5).
//!
//! One tokio task per open project owns the [`Project`], its undo [`se_domain::History`] and its
//! [`ProjectStore`]. Messages are handled one at a time, so commits are totally ordered. A commit
//! is applied to a draft, journaled durably, then installed and announced
//! ([`se_protocol::CoreEvent::ProjectPatched`], then
//! [`se_protocol::CoreEvent::HistoryChanged`]); the snapshot is saved
//! [`ActorDeps::save_delay`] after the last commit and when the actor stops. Readers get an
//! `Arc<Project>`, never a lock.
//!
//! The actor also owns the open **changesets** of AI tasks (ADR-0005, ADR-0029): an edit of a
//! task ([`ProjectActorHandle::apply_ai`]) is journaled together with its changeset entry in one
//! atomic write, and [`ProjectActorHandle::revert_task`] undoes a task's changeset as one user
//! transaction, reporting what conflicts.

mod ai_edits;
mod runner;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use se_domain::{Changeset, ChangesetId, ItemId, Project, ProjectId, TaskId, Transaction};
use se_protocol::{HistoryState, TaskRevertReport};
use tokio::sync::{mpsc, oneshot};
use tracing::Instrument;

use crate::{
    AppError,
    ports::{AnalysisStore, Clock, EventSink, IdGenerator, ProjectStore, TaskStore},
};

/// Messages waiting before senders are slowed down.
const INBOX_CAPACITY: usize = 64;

/// Snapshot delay after the last commit used by the app.
pub const DEFAULT_SAVE_DELAY: Duration = Duration::from_secs(2);

/// What an actor needs besides its project and store.
#[derive(Clone)]
pub struct ActorDeps {
    /// Where patches and history changes go.
    pub events: Arc<dyn EventSink>,
    /// Timestamps of journal entries.
    pub clock: Arc<dyn Clock>,
    /// Ids of undo and redo transactions.
    pub ids: Arc<dyn IdGenerator>,
    /// How long after the last commit the snapshot is saved.
    pub save_delay: Duration,
}

/// The state of a project at one moment, shared without copying.
#[derive(Debug, Clone)]
pub struct ProjectView {
    /// The document.
    pub project: Arc<Project>,
    /// What undo and redo would do.
    pub history: HistoryState,
}

/// An AI task's edit, committed and recorded in the task's changeset.
#[derive(Debug, Clone)]
pub struct AiCommit {
    /// The project's revision after the edit.
    pub revision: u64,
    /// The document right after the edit (later commits may already have followed).
    pub project: Arc<Project>,
    /// The items the edit added, changed or removed.
    pub touched_items: Vec<ItemId>,
    /// The task's open changeset.
    pub changeset_id: ChangesetId,
    /// Whether this edit created that changeset (the task's first edit since it started or was
    /// reverted).
    pub changeset_created: bool,
}

/// The items an AI task added or changed that still exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangesetItems {
    /// The task's open changeset (`None` while it changed nothing or once reverted).
    pub changeset_id: Option<ChangesetId>,
    /// Its items still in the project, in id order.
    pub touched_items: Vec<ItemId>,
}

/// A message to the actor; every one carries the channel its answer goes to.
enum Message {
    Apply {
        transaction: Transaction,
        reply: oneshot::Sender<Result<u64, AppError>>,
    },
    ApplyAi {
        transaction: Transaction,
        base_revision: Option<u64>,
        reply: oneshot::Sender<Result<Option<AiCommit>, AppError>>,
    },
    RevertTask {
        task_id: TaskId,
        label: String,
        reply: oneshot::Sender<Result<TaskRevertReport, AppError>>,
    },
    Changeset {
        task_id: TaskId,
        reply: oneshot::Sender<Option<Changeset>>,
    },
    PreviewRevert {
        task_id: TaskId,
        reply: oneshot::Sender<Result<TaskRevertReport, AppError>>,
    },
    ChangesetItems {
        task_id: TaskId,
        reply: oneshot::Sender<ChangesetItems>,
    },
    Undo {
        reply: oneshot::Sender<Result<u64, AppError>>,
    },
    Redo {
        reply: oneshot::Sender<Result<u64, AppError>>,
    },
    Snapshot {
        reply: oneshot::Sender<ProjectView>,
    },
    Close {
        reply: oneshot::Sender<Result<(), AppError>>,
    },
}

/// Sends messages to a running project actor. Cheap to clone; the actor stops (saving its
/// snapshot) on [`ProjectActorHandle::close`] or once every handle is dropped.
#[derive(Clone)]
pub struct ProjectActorHandle {
    project_id: ProjectId,
    path: PathBuf,
    analyses: Arc<dyn AnalysisStore>,
    tasks: Arc<dyn TaskStore>,
    sender: mpsc::Sender<Message>,
}

impl ProjectActorHandle {
    /// Starts the actor of `project` with the open `changesets` of its AI tasks, persisted
    /// through `store` (already locked and loaded).
    #[must_use]
    pub fn spawn(
        project: Project,
        changesets: Vec<Changeset>,
        store: Box<dyn ProjectStore>,
        deps: ActorDeps,
    ) -> Self {
        let (sender, inbox) = mpsc::channel(INBOX_CAPACITY);
        let handle = Self {
            project_id: project.id,
            path: store.path().to_path_buf(),
            analyses: store.analyses(),
            tasks: store.tasks(),
            sender,
        };
        let span = tracing::info_span!("project_actor", project = %project.id);
        tokio::spawn(runner::run(project, changesets, store, deps, inbox).instrument(span));
        handle
    }

    /// Id of the project.
    #[must_use]
    pub fn project_id(&self) -> ProjectId {
        self.project_id
    }

    /// Canonical path of the project folder.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The analyses of the project's assets (project database). Not document state: written
    /// directly, not through the actor.
    #[must_use]
    pub fn analyses(&self) -> &Arc<dyn AnalysisStore> {
        &self.analyses
    }

    /// The AI tasks of the project (project database). Not document state: written directly by
    /// the task service, not through the actor.
    #[must_use]
    pub fn tasks(&self) -> &Arc<dyn TaskStore> {
        &self.tasks
    }

    /// Commits `transaction` and returns the new revision.
    ///
    /// # Errors
    /// [`AppError::Domain`] when the document refuses it, [`AppError::Storage`] when it cannot be
    /// journaled (nothing changes then), [`AppError::ActorStopped`].
    pub async fn apply(&self, transaction: Transaction) -> Result<u64, AppError> {
        self.ask(|reply| Message::Apply { transaction, reply })
            .await?
    }

    /// Commits `transaction`, an edit by an AI task (`Actor::AiTask`), and records it in the
    /// task's open changeset (created by the task's first edit), journal and changeset in one
    /// atomic write; then publishes `projectPatched` and `taskChangesetUpdated`.
    ///
    /// # Errors
    /// The errors of [`ProjectActorHandle::apply`]; [`AppError::Internal`] for a transaction
    /// that is not by an AI task.
    pub async fn apply_ai(&self, transaction: Transaction) -> Result<AiCommit, AppError> {
        self.ask(|reply| Message::ApplyAi {
            transaction,
            base_revision: None,
            reply,
        })
        .await??
        .ok_or_else(|| AppError::Internal("an unconditional AI edit was not applied".to_owned()))
    }

    /// Like [`ProjectActorHandle::apply_ai`], but only if the project is still at
    /// `base_revision` (the state the edit was planned and scope-checked on); `None` when
    /// another commit came first: plan again.
    ///
    /// # Errors
    /// The errors of [`ProjectActorHandle::apply_ai`].
    pub async fn apply_ai_on(
        &self,
        transaction: Transaction,
        base_revision: u64,
    ) -> Result<Option<AiCommit>, AppError> {
        self.ask(|reply| Message::ApplyAi {
            transaction,
            base_revision: Some(base_revision),
            reply,
        })
        .await?
    }

    /// Reverts the open changeset of `task_id` against the current document: what nobody
    /// changed since is undone as one user transaction labelled `label` (undoable), the rest is
    /// reported as conflicts. When something was undone, the changeset is closed.
    ///
    /// # Errors
    /// [`AppError::NothingToRevert`] when the task has no open changeset; the errors of
    /// [`ProjectActorHandle::apply`].
    pub async fn revert_task(
        &self,
        task_id: TaskId,
        label: &str,
    ) -> Result<TaskRevertReport, AppError> {
        let label = label.to_owned();
        self.ask(|reply| Message::RevertTask {
            task_id,
            label,
            reply,
        })
        .await?
    }

    /// The open changeset of `task_id`, if it has one.
    ///
    /// # Errors
    /// [`AppError::ActorStopped`].
    pub async fn changeset(&self, task_id: TaskId) -> Result<Option<Changeset>, AppError> {
        self.ask(|reply| Message::Changeset { task_id, reply })
            .await
    }

    /// What [`ProjectActorHandle::revert_task`] would report now; nothing is committed.
    ///
    /// # Errors
    /// [`AppError::NothingToRevert`] when the task has no open changeset; the document's refusal.
    pub async fn preview_revert(&self, task_id: TaskId) -> Result<TaskRevertReport, AppError> {
        self.ask(|reply| Message::PreviewRevert { task_id, reply })
            .await?
    }

    /// The items of `task_id`'s open changeset that still exist.
    ///
    /// # Errors
    /// [`AppError::ActorStopped`].
    pub async fn changeset_items(&self, task_id: TaskId) -> Result<ChangesetItems, AppError> {
        self.ask(|reply| Message::ChangesetItems { task_id, reply })
            .await
    }

    /// Undoes the latest user transaction and returns the new revision.
    ///
    /// # Errors
    /// `NothingToUndo` and the other errors of [`ProjectActorHandle::apply`].
    pub async fn undo(&self) -> Result<u64, AppError> {
        self.ask(|reply| Message::Undo { reply }).await?
    }

    /// Redoes the latest undone transaction and returns the new revision.
    ///
    /// # Errors
    /// `NothingToRedo` and the other errors of [`ProjectActorHandle::apply`].
    pub async fn redo(&self) -> Result<u64, AppError> {
        self.ask(|reply| Message::Redo { reply }).await?
    }

    /// The current document and undo state.
    ///
    /// # Errors
    /// [`AppError::ActorStopped`].
    pub async fn view(&self) -> Result<ProjectView, AppError> {
        self.ask(|reply| Message::Snapshot { reply }).await
    }

    /// Saves the snapshot if needed, releases the store and stops the actor.
    ///
    /// # Errors
    /// The save or release error (the journal still holds every commit), or
    /// [`AppError::ActorStopped`] when it had already stopped.
    pub async fn close(&self) -> Result<(), AppError> {
        self.ask(|reply| Message::Close { reply }).await?
    }

    async fn ask<T>(
        &self,
        message: impl FnOnce(oneshot::Sender<T>) -> Message,
    ) -> Result<T, AppError> {
        let stopped = || AppError::ActorStopped {
            project_id: self.project_id,
        };
        let (reply, answer) = oneshot::channel();
        self.sender
            .send(message(reply))
            .await
            .map_err(|_| stopped())?;
        answer.await.map_err(|_| stopped())
    }
}
