//! AI provider DTOs (`docs/architecture.md` §9, ADR-0003, ADR-0027): which AI CLI is used,
//! whether it is installed and logged in, what it can do, the AI settings, and the events an
//! agent run produces. Provider-agnostic: only `crates/se-agents/src/providers/<provider>/` knows
//! a concrete CLI; clients show what these types say.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Identifies an AI provider (an AI CLI adapter of se-agents), e.g. `claude-code`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
pub struct ProviderId(pub String);

impl ProviderId {
    /// The id as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProviderId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// What clients show about a provider, and how a user installs it or logs in. Written by the
/// provider so the UI never names a CLI itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    /// Stable id.
    pub id: ProviderId,
    /// Product name (`Claude Code`).
    pub name: String,
    /// What a user types in a terminal to start the CLI (`claude`).
    pub command: String,
    /// What a user types inside the CLI to log in (`/login`).
    pub login_command: String,
    /// Web page explaining how to install the CLI (opened in the user's browser on request).
    pub help_url: String,
}

/// Whether the provider's CLI was found and runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum InstallationStatus {
    /// Found and answering.
    Installed {
        /// Absolute path of the executable in use.
        path: String,
        /// Its version (`2.1.317`).
        version: String,
    },
    /// Not found where it was looked for.
    NotInstalled {
        /// Where it was looked for and what to do (install it, or set its path).
        message: String,
    },
    /// Found but it does not answer (`--version` failed): a broken or foreign executable.
    Unusable {
        /// Absolute path of the executable tried.
        path: String,
        /// What went wrong.
        message: String,
    },
}

/// Whether the provider's CLI has a logged-in session. The app never reads, stores or forwards
/// credentials: it only asks the CLI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum AuthStatus {
    /// Logged in.
    LoggedIn {
        /// The kind of account when the CLI tells it (`max`, `pro`, `api key`…), never an
        /// identity (no e-mail, no organisation).
        account_kind: Option<String>,
    },
    /// Not logged in: AI features are disabled until the user logs in from a terminal.
    LoggedOut,
    /// Could not be determined (CLI missing, unreadable answer…).
    Unknown {
        /// Why.
        reason: String,
    },
}

/// A model a user can pick (an alias of the CLI for its latest model of a family).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ModelOption {
    /// What is passed to the CLI (`opus`).
    pub id: String,
    /// What is shown (`Opus`).
    pub label: String,
}

/// How much reasoning an agent spends (passed to CLIs that support it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
pub enum EffortLevel {
    /// Fastest, cheapest.
    Low,
    /// Balanced.
    Medium,
    /// Thorough.
    High,
    /// More thorough.
    XHigh,
    /// Everything it has.
    Max,
}

/// What a provider supports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCapabilities {
    /// Whether a finished run's session can be resumed (follow-ups).
    pub resume_sessions: bool,
    /// Whether runs accept images (frames sent by MCP tools).
    pub image_input: bool,
    /// Models a user can pick; empty when the CLI decides alone.
    pub models: Vec<ModelOption>,
    /// Effort levels a user can pick; empty when not supported.
    pub effort_levels: Vec<EffortLevel>,
}

/// Everything clients show about the AI provider in use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AiStatus {
    /// The provider.
    pub provider: ProviderInfo,
    /// Whether its CLI was found.
    pub installation: InstallationStatus,
    /// Whether its CLI is logged in.
    pub auth: AuthStatus,
    /// What it supports.
    pub capabilities: ProviderCapabilities,
}

/// AI settings of the app (`library.db`, edited in « Réglages » → IA).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AiSettings {
    /// The provider running tasks.
    pub provider: ProviderId,
    /// Absolute path of the CLI executable chosen by the user; `null` searches it.
    pub executable_path: Option<String>,
    /// Model of new tasks (a [`ModelOption::id`]); `null` lets the CLI decide.
    pub default_model: Option<String>,
    /// Effort of new tasks; `null` lets the CLI decide.
    pub default_effort: Option<EffortLevel>,
    /// Tasks running at once, in [`AiSettings::MAX_CONCURRENT_TASKS_RANGE`]; others wait.
    pub max_concurrent_tasks: u8,
}

impl AiSettings {
    /// Accepted values of [`AiSettings::max_concurrent_tasks`].
    pub const MAX_CONCURRENT_TASKS_RANGE: std::ops::RangeInclusive<u8> = 1..=8;
    /// Default of [`AiSettings::max_concurrent_tasks`].
    pub const DEFAULT_MAX_CONCURRENT_TASKS: u8 = 3;
}

/// Why an agent run failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum AgentFailureKind {
    /// The CLI was not found.
    NotInstalled,
    /// The CLI is not logged in.
    NotAuthenticated,
    /// The account's usage limit is reached: try again later.
    RateLimited,
    /// The CLI stopped unexpectedly (or reported an error).
    Crashed,
    /// Cancelled by the user, or because its project closed.
    Cancelled,
    /// The CLI's output could not be understood (a CLI version this app does not know).
    ProtocolError,
    /// The core stopped while the run was queued or running (set when the project reopens).
    Interrupted,
}

/// What an agent run reports, in order. A run ends with exactly one [`AgentEvent::Completed`]
/// or [`AgentEvent::Failed`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum AgentEvent {
    /// The CLI started its session (follow-ups resume it).
    SessionStarted {
        /// The CLI's session id.
        session_id: String,
        /// The model it runs, when it says.
        model: Option<String>,
    },
    /// Text written by the agent.
    AssistantText {
        /// The text (markdown).
        text: String,
    },
    /// The agent called a tool.
    ToolCallStarted {
        /// Id pairing the call with its result.
        call_id: String,
        /// The tool's name.
        tool: String,
        /// Its input, shortened for display.
        input_summary: String,
    },
    /// A tool call returned.
    ToolCallFinished {
        /// Id of the call.
        call_id: String,
        /// The tool's name (empty when the call was not seen).
        tool: String,
        /// Whether it succeeded.
        ok: bool,
        /// Its output, shortened for display.
        output_summary: String,
    },
    /// What the run consumed.
    Usage {
        /// Input tokens (cached ones included).
        input_tokens: u64,
        /// Output tokens.
        output_tokens: u64,
        /// Cost in US dollars, when the CLI reports it.
        cost_usd: Option<f64>,
        /// Duration of the run in milliseconds, when the CLI reports it.
        duration_ms: Option<u64>,
    },
    /// The run finished.
    Completed {
        /// The agent's final answer.
        summary: String,
    },
    /// The run failed.
    Failed {
        /// Why.
        failure: AgentFailureKind,
        /// What happened, actionable.
        message: String,
    },
}

impl AgentEvent {
    /// Whether this event ends a run.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed { .. } | Self::Failed { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_are_tagged_by_kind() {
        let auth = AuthStatus::LoggedIn {
            account_kind: Some("max".to_owned()),
        };
        assert_eq!(
            serde_json::to_string(&auth).unwrap(),
            r#"{"kind":"loggedIn","accountKind":"max"}"#
        );
        let installation = InstallationStatus::Installed {
            path: "/bin/claude".to_owned(),
            version: "2.1.317".to_owned(),
        };
        assert_eq!(
            serde_json::to_string(&installation).unwrap(),
            r#"{"kind":"installed","path":"/bin/claude","version":"2.1.317"}"#
        );
    }

    #[test]
    fn effort_levels_are_lowercase() {
        assert_eq!(
            serde_json::to_string(&EffortLevel::XHigh).unwrap(),
            r#""xhigh""#
        );
    }

    #[test]
    fn failed_events_name_their_failure() {
        let event = AgentEvent::Failed {
            failure: AgentFailureKind::RateLimited,
            message: "later".to_owned(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(
            json,
            r#"{"kind":"failed","failure":"rateLimited","message":"later"}"#
        );
        assert_eq!(serde_json::from_str::<AgentEvent>(&json).unwrap(), event);
        assert!(event.is_terminal());
    }
}
