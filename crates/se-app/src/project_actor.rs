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

mod runner;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use se_domain::{Project, ProjectId, Transaction};
use se_protocol::HistoryState;
use tokio::sync::{mpsc, oneshot};
use tracing::Instrument;

use crate::{
    AppError,
    ports::{Clock, EventSink, IdGenerator, ProjectStore},
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

/// A message to the actor; every one carries the channel its answer goes to.
enum Message {
    Apply {
        transaction: Transaction,
        reply: oneshot::Sender<Result<u64, AppError>>,
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
    sender: mpsc::Sender<Message>,
}

impl ProjectActorHandle {
    /// Starts the actor of `project`, persisted through `store` (already locked and loaded).
    #[must_use]
    pub fn spawn(project: Project, store: Box<dyn ProjectStore>, deps: ActorDeps) -> Self {
        let (sender, inbox) = mpsc::channel(INBOX_CAPACITY);
        let handle = Self {
            project_id: project.id,
            path: store.path().to_path_buf(),
            sender,
        };
        let span = tracing::info_span!("project_actor", project = %project.id);
        tokio::spawn(runner::run(project, store, deps, inbox).instrument(span));
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

    /// Commits `transaction` and returns the new revision.
    ///
    /// # Errors
    /// [`AppError::Domain`] when the document refuses it, [`AppError::Storage`] when it cannot be
    /// journaled (nothing changes then), [`AppError::ActorStopped`].
    pub async fn apply(&self, transaction: Transaction) -> Result<u64, AppError> {
        self.ask(|reply| Message::Apply { transaction, reply })
            .await?
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
