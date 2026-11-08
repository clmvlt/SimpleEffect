//! Agent ports (`docs/architecture.md` §9.1, ADR-0003, ADR-0027): [`AgentProvider`], an AI CLI
//! that runs agents (implemented in se-agents, one adapter per CLI), and [`AgentWorkspaces`], the
//! empty working folder of each run.
//!
//! Everything outside `crates/se-agents/src/providers/<provider>/` talks to AI CLIs through these
//! traits only (golden rule 3).

use std::{fmt, path::PathBuf};

use async_trait::async_trait;
use se_domain::TaskId;
use se_protocol::{
    AgentEvent, AgentFailureKind, AuthStatus, EffortLevel, ProviderCapabilities, ProviderInfo,
};
use tokio::sync::{mpsc, oneshot};

/// An AI CLI able to run agents in the background.
#[async_trait]
pub trait AgentProvider: Send + Sync {
    /// Who it is, and how users install it and log in.
    fn info(&self) -> ProviderInfo;

    /// What it supports (resume, images, models, effort levels).
    fn capabilities(&self) -> ProviderCapabilities;

    /// Uses the executable at `path` instead of searching for it (`None`: search again).
    fn set_executable(&self, path: Option<PathBuf>);

    /// Finds the CLI and reads its version.
    ///
    /// # Errors
    /// [`AgentError::NotInstalled`] when it is nowhere to be found; [`AgentError::Unusable`] when
    /// it is found but does not answer.
    async fn detect(&self) -> Result<ProviderInstallation, AgentError>;

    /// Asks the CLI whether it is logged in. Never reads credentials itself.
    ///
    /// # Errors
    /// [`AgentError::NotInstalled`]; [`AgentError::Failed`] when the CLI's answer cannot be read.
    async fn auth_status(&self) -> Result<AuthStatus, AgentError>;

    /// Starts an agent. Its events arrive on [`AgentRun::events`], ending with exactly one
    /// terminal event ([`AgentEvent::Completed`] or [`AgentEvent::Failed`]), after which the
    /// channel closes once the CLI process is gone.
    ///
    /// # Errors
    /// [`AgentError::NotInstalled`]; [`AgentError::Failed`] when the process cannot start.
    async fn start(&self, spec: AgentRunSpec) -> Result<AgentRun, AgentError>;
}

/// Where the CLI is and which version it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderInstallation {
    /// Absolute path of the executable.
    pub path: PathBuf,
    /// Its version (`2.1.317`).
    pub version: String,
}

/// What an agent run must do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRunSpec {
    /// The user's prompt.
    pub prompt: String,
    /// Instructions added to the CLI's system prompt (provider-agnostic markdown).
    pub system_instructions: String,
    /// The MCP endpoint the agent acts through. TODO(prompt-18): required once `/mcp` exists.
    pub mcp: Option<McpEndpoint>,
    /// The model (`None`: the CLI decides).
    pub model: Option<String>,
    /// The effort (`None`: the CLI decides).
    pub effort: Option<EffortLevel>,
    /// A new session or the one to resume.
    pub session: AgentSession,
    /// An empty folder the process runs in.
    pub working_dir: PathBuf,
}

/// The agent session of a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentSession {
    /// A new session with this id (a UUID; providers that cannot choose ids report theirs with
    /// [`AgentEvent::SessionStarted`]).
    Start(String),
    /// The session of a previous run, resumed (follow-ups).
    Resume(String),
}

/// The MCP endpoint of a task: se-core's `/mcp` and the task's bearer token.
#[derive(Clone, PartialEq, Eq)]
pub struct McpEndpoint {
    /// `http://127.0.0.1:<port>/mcp`.
    pub url: String,
    /// The task's token. Never logged: `Debug` redacts it.
    pub bearer_token: String,
}

impl fmt::Debug for McpEndpoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("McpEndpoint")
            .field("url", &self.url)
            .field("bearer_token", &"<redacted>")
            .finish()
    }
}

/// A started agent: its events and how to stop it.
#[derive(Debug)]
pub struct AgentRun {
    /// Events in order; the channel closes after the terminal event, once the process is gone.
    pub events: mpsc::Receiver<AgentEvent>,
    /// Stops the run (the whole process tree). Dropping it stops the run too.
    pub cancel: CancelHandle,
}

/// The caller's side of a run's cancellation. Cancelling twice does nothing more.
#[derive(Debug)]
pub struct CancelHandle {
    sender: Option<oneshot::Sender<()>>,
}

/// The provider's side of a run's cancellation.
#[derive(Debug)]
pub struct CancelSignal {
    receiver: oneshot::Receiver<()>,
    fired: bool,
}

impl CancelHandle {
    /// A handle and the signal it fires.
    #[must_use]
    pub fn new() -> (Self, CancelSignal) {
        let (sender, receiver) = oneshot::channel();
        (
            Self {
                sender: Some(sender),
            },
            CancelSignal {
                receiver,
                fired: false,
            },
        )
    }

    /// Asks the run to stop.
    pub fn cancel(&mut self) {
        if let Some(sender) = self.sender.take() {
            // The run may have ended already: nothing to stop then.
            let _ = sender.send(());
        }
    }
}

impl CancelSignal {
    /// Resolves once the run must stop: [`CancelHandle::cancel`] was called or the handle was
    /// dropped. Resolves at once on later calls.
    pub async fn cancelled(&mut self) {
        if !self.fired {
            // Both a message and a dropped handle mean "stop".
            let _ = (&mut self.receiver).await;
            self.fired = true;
        }
    }
}

/// Why a provider call failed. Messages are actionable: they reach users and AI agents.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AgentError {
    /// The CLI was not found.
    #[error("{message}")]
    NotInstalled {
        /// Where it was looked for, and what to do.
        message: String,
    },
    /// The CLI was found but does not answer.
    #[error("{} does not work: {message}", path.display())]
    Unusable {
        /// The executable tried.
        path: PathBuf,
        /// What went wrong.
        message: String,
    },
    /// The CLI is not logged in.
    #[error("{message}")]
    NotAuthenticated {
        /// What to do.
        message: String,
    },
    /// Anything else (the process could not start, unreadable output, IO).
    #[error("{message}")]
    Failed {
        /// What went wrong.
        message: String,
    },
}

impl AgentError {
    /// How a run failing with this error is classified.
    #[must_use]
    pub const fn failure_kind(&self) -> AgentFailureKind {
        match self {
            Self::NotInstalled { .. } | Self::Unusable { .. } => AgentFailureKind::NotInstalled,
            Self::NotAuthenticated { .. } => AgentFailureKind::NotAuthenticated,
            Self::Failed { .. } => AgentFailureKind::Crashed,
        }
    }

    /// The terminal event of a run that failed with this error.
    #[must_use]
    pub fn to_event(&self) -> AgentEvent {
        AgentEvent::Failed {
            failure: self.failure_kind(),
            message: self.to_string(),
        }
    }
}

/// The working folders of agent runs: one empty folder per task, created when a run starts and
/// removed when it ends (implemented in se-agents).
#[async_trait]
pub trait AgentWorkspaces: Send + Sync {
    /// Creates the task's folder, empty (leftovers of an earlier run are removed first), and
    /// returns its absolute path. The same task always gets the same path, so CLIs that key
    /// sessions by folder can resume them.
    ///
    /// # Errors
    /// [`AgentError::Failed`] when the folder cannot be created.
    async fn prepare(&self, task_id: TaskId) -> Result<PathBuf, AgentError>;

    /// Removes the task's folder. Failures are logged, never returned.
    async fn release(&self, task_id: TaskId);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cancelling_or_dropping_the_handle_fires_the_signal() {
        let (mut handle, mut signal) = CancelHandle::new();
        handle.cancel();
        handle.cancel();
        signal.cancelled().await;
        signal.cancelled().await;
        let (handle, mut signal) = CancelHandle::new();
        drop(handle);
        signal.cancelled().await;
    }

    #[test]
    fn tokens_are_redacted() {
        let endpoint = McpEndpoint {
            url: "http://127.0.0.1:1/mcp".to_owned(),
            bearer_token: "secret".to_owned(),
        };
        assert!(!format!("{endpoint:?}").contains("secret"));
    }

    #[test]
    fn errors_classify_runs() {
        let error = AgentError::NotAuthenticated {
            message: "log in".to_owned(),
        };
        assert_eq!(error.failure_kind(), AgentFailureKind::NotAuthenticated);
        assert!(matches!(error.to_event(), AgentEvent::Failed { .. }));
    }
}
