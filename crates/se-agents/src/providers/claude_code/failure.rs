//! Classifying Claude Code failures from their messages (usage limits, missing login, sessions
//! that cannot be resumed, crashes) and making them actionable. Phrases come from real CLI
//! output (fixtures in `tests/fixtures/claude_code/`).

use se_protocol::{AgentEvent, AgentFailureKind};

/// Phrases of usage-limit errors (lowercase).
const RATE_LIMIT_PHRASES: &[&str] = &[
    "usage limit",
    "rate limit",
    "rate_limit",
    "hit your limit",
    "limit reached",
    "too many requests",
];

/// Phrases of missing or expired logins (lowercase).
const AUTH_PHRASES: &[&str] = &[
    "/login",
    "not logged in",
    "invalid api key",
    "authentication_error",
    "authentication failed",
    "oauth token has expired",
    "please log in",
];

/// The CLI refusing our isolation flags because the program run adds its own permission options
/// (2.1.319 through a user's `claude.cmd` wrapper adding `--allow-dangerously-skip-permissions`:
/// `Error: bypassPermissions not supported in restricted mode`, stderr only, exit 1).
const MISCONFIGURED_PHRASES: &[&str] = &["not supported in restricted mode"];

/// `--resume` of a session the CLI does not have (2.1.319: `No conversation found with session
/// ID: <id>`, on stderr and in the `result` line's `errors`).
const LOST_SESSION: &str = "no conversation found";

/// `--session-id` of a session that already exists (2.1.319: `Error: Session ID <id> is already
/// in use.`, stderr only).
const SESSION_IN_USE: &str = "is already in use";

/// The terminal event of a run that failed with `detail` (a CLI message). `context` prefixes
/// details the app cannot explain better (`Claude Code stopped (exit status: 1)`).
pub(super) fn failed(detail: &str, context: Option<&str>) -> AgentEvent {
    let failure = classify(detail);
    AgentEvent::Failed {
        failure,
        message: explain(failure, detail.trim(), context),
    }
}

/// The failure kind a Claude Code error message describes.
fn classify(message: &str) -> AgentFailureKind {
    let lower = message.to_lowercase();
    if RATE_LIMIT_PHRASES
        .iter()
        .any(|phrase| lower.contains(phrase))
    {
        AgentFailureKind::RateLimited
    } else if AUTH_PHRASES.iter().any(|phrase| lower.contains(phrase)) {
        AgentFailureKind::NotAuthenticated
    } else if MISCONFIGURED_PHRASES
        .iter()
        .any(|phrase| lower.contains(phrase))
    {
        AgentFailureKind::Misconfigured
    } else {
        AgentFailureKind::Crashed
    }
}

/// The message of a failure, saying what to do when the user can do something.
fn explain(failure: AgentFailureKind, detail: &str, context: Option<&str>) -> String {
    let lower = detail.to_lowercase();
    match failure {
        AgentFailureKind::RateLimited => format!(
            "Claude Code's usage limit is reached ({detail}): retry the task once it resets"
        ),
        AgentFailureKind::NotAuthenticated => format!(
            "Claude Code is not logged in ({detail}): open a terminal, run `claude`, then type \
             `/login`, and retry the task"
        ),
        AgentFailureKind::Misconfigured => format!(
            "Claude Code refused the options SimpleEffect runs it with ({detail}): the program \
             used adds options of its own, often a wrapper script found on the PATH; choose the \
             claude executable itself in Settings → AI (CLI location), then retry the task"
        ),
        _ if lower.contains(LOST_SESSION) => format!(
            "the agent session to resume no longer exists ({detail}): retry the task, or submit \
             a new one"
        ),
        _ if lower.contains(SESSION_IN_USE) => {
            format!("the agent session id was already used ({detail}): retry the task")
        }
        _ => match context {
            Some(context) => format!("{context}: {detail}"),
            None => detail.to_owned(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind_and_message(event: &AgentEvent) -> (AgentFailureKind, &str) {
        match event {
            AgentEvent::Failed { failure, message } => (*failure, message),
            other => panic!("expected a failure, got {other:?}"),
        }
    }

    #[test]
    fn messages_are_classified() {
        let limit = failed("Claude AI usage limit reached|1767225600", None);
        assert_eq!(kind_and_message(&limit).0, AgentFailureKind::RateLimited);
        let limit = failed("You've hit your limit · resets 3pm", None);
        assert_eq!(kind_and_message(&limit).0, AgentFailureKind::RateLimited);
        let login = failed("Not logged in · Please run /login", None);
        let (kind, message) = kind_and_message(&login);
        assert_eq!(kind, AgentFailureKind::NotAuthenticated);
        assert!(message.contains("open a terminal"));
    }

    #[test]
    fn a_wrapper_adding_permission_options_is_a_configuration_problem() {
        // Recorded on Windows through a `claude.cmd` adding --allow-dangerously-skip-permissions.
        let refused = failed(
            "Error: bypassPermissions not supported in restricted mode",
            Some("Claude Code stopped (exit code: 1)"),
        );
        let (kind, message) = kind_and_message(&refused);
        assert_eq!(kind, AgentFailureKind::Misconfigured);
        assert!(message.contains("Settings → AI"), "{message}");
    }

    #[test]
    fn session_problems_say_to_retry() {
        let lost = failed("No conversation found with session ID: 42", None);
        let (kind, message) = kind_and_message(&lost);
        assert_eq!(kind, AgentFailureKind::Crashed);
        assert!(message.starts_with("the agent session to resume no longer exists"));
        let used = failed("Error: Session ID 42 is already in use.", Some("ignored"));
        assert!(kind_and_message(&used).1.contains("already used"));
    }

    #[test]
    fn other_failures_keep_their_detail_after_the_context() {
        let crash = failed(" boom ", Some("Claude Code stopped (signal: 9)"));
        assert_eq!(
            kind_and_message(&crash),
            (
                AgentFailureKind::Crashed,
                "Claude Code stopped (signal: 9): boom"
            )
        );
    }
}
