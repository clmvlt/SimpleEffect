//! Classifying Claude Code failures from their messages: usage limits, missing login, sessions
//! that cannot be resumed, crashes.

use se_protocol::AgentFailureKind;

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

/// The failure kind a Claude Code error message describes.
pub(super) fn classify(message: &str) -> AgentFailureKind {
    let lower = message.to_lowercase();
    if RATE_LIMIT_PHRASES
        .iter()
        .any(|phrase| lower.contains(phrase))
    {
        AgentFailureKind::RateLimited
    } else if AUTH_PHRASES.iter().any(|phrase| lower.contains(phrase)) {
        AgentFailureKind::NotAuthenticated
    } else {
        AgentFailureKind::Crashed
    }
}

/// The message of a failure, made actionable for the kinds the user can fix.
pub(super) fn explain(failure: AgentFailureKind, detail: &str) -> String {
    let detail = detail.trim();
    match failure {
        AgentFailureKind::RateLimited => format!(
            "Claude Code's usage limit is reached ({detail}): retry the task once it resets"
        ),
        AgentFailureKind::NotAuthenticated => format!(
            "Claude Code is not logged in ({detail}): open a terminal, run `claude`, then type \
             `/login`, and retry the task"
        ),
        _ => detail.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_are_classified() {
        assert_eq!(
            classify("Claude AI usage limit reached|1767225600"),
            AgentFailureKind::RateLimited
        );
        assert_eq!(
            classify("You've hit your limit · resets 3pm"),
            AgentFailureKind::RateLimited
        );
        assert_eq!(
            classify("Invalid API key · Please run /login"),
            AgentFailureKind::NotAuthenticated
        );
        assert_eq!(
            classify("No conversation found with session ID: x"),
            AgentFailureKind::Crashed
        );
    }

    #[test]
    fn fixable_failures_say_what_to_do() {
        let message = explain(AgentFailureKind::NotAuthenticated, "Not logged in");
        assert!(message.contains("/login"));
        assert_eq!(explain(AgentFailureKind::Crashed, " boom "), "boom");
    }
}
