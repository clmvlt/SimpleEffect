//! The environment `claude` runs with. When SimpleEffect itself was started from inside a Claude
//! Code session (the development setup: `pnpm dev` from Claude Code's terminal), it inherits the
//! variables that session sets for its own children (`CLAUDECODE`, `CLAUDE_CODE_SIMPLE`,
//! `CLAUDE_CODE_SESSION_ID`, a local `ANTHROPIC_BASE_URL`…). Passed on, they would make our CLI a
//! nested child of that session (bare mode, another endpoint, another session's ids), so they are
//! removed. Started normally, the app passes its environment untouched.

use std::ffi::{OsStr, OsString};

/// Set by Claude Code in the environment of everything it starts.
const SESSION_MARKER: &str = "CLAUDECODE";

/// Prefixes of the variables a Claude Code session sets for its children.
const SESSION_PREFIXES: &[&str] = &["CLAUDE_CODE_", "CLAUDE_AGENT_SDK_"];

/// Other variables a Claude Code session sets for its children.
const SESSION_VARIABLES: &[&str] = &[
    SESSION_MARKER,
    "CLAUDE_PID",
    "CLAUDE_EFFORT",
    "ANTHROPIC_BASE_URL",
];

/// User configuration that shares the prefixes and must survive (headless token, cloud
/// providers, output limit).
const USER_CONFIGURATION: &[&str] = &[
    "CLAUDE_CODE_OAUTH_TOKEN",
    "CLAUDE_CODE_USE_BEDROCK",
    "CLAUDE_CODE_USE_VERTEX",
    "CLAUDE_CODE_USE_FOUNDRY",
    "CLAUDE_CODE_MAX_OUTPUT_TOKENS",
];

/// Names of the inherited session variables to remove from `variables` (none unless the
/// session marker is present).
pub(super) fn inherited_session_variables(
    variables: impl IntoIterator<Item = (OsString, OsString)>,
) -> Vec<OsString> {
    let names: Vec<OsString> = variables.into_iter().map(|(name, _)| name).collect();
    if !names.iter().any(|name| name == OsStr::new(SESSION_MARKER)) {
        return Vec::new();
    }
    names
        .into_iter()
        .filter(|name| {
            let Some(text) = name.to_str() else {
                return false;
            };
            let session = SESSION_VARIABLES.contains(&text)
                || SESSION_PREFIXES
                    .iter()
                    .any(|prefix| text.starts_with(prefix));
            session && !USER_CONFIGURATION.contains(&text)
        })
        .collect()
}

/// The variables to remove from this process's environment for `claude`.
pub(super) fn variables_to_remove() -> Vec<OsString> {
    inherited_session_variables(std::env::vars_os())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn variables(names: &[&str]) -> Vec<(OsString, OsString)> {
        names
            .iter()
            .map(|name| (OsString::from(name), OsString::from("x")))
            .collect()
    }

    #[test]
    fn nothing_is_removed_outside_a_claude_code_session() {
        let removed =
            inherited_session_variables(variables(&["PATH", "CLAUDE_CODE_SIMPLE", "HOME"]));
        assert_eq!(removed, Vec::<OsString>::new());
    }

    #[test]
    fn session_variables_go_and_user_configuration_stays() {
        let removed = inherited_session_variables(variables(&[
            "PATH",
            "CLAUDECODE",
            "CLAUDE_CODE_SIMPLE",
            "CLAUDE_CODE_SESSION_ID",
            "CLAUDE_AGENT_SDK_VERSION",
            "ANTHROPIC_BASE_URL",
            "CLAUDE_CODE_OAUTH_TOKEN",
            "CLAUDE_CONFIG_DIR",
            "ANTHROPIC_API_KEY",
        ]));
        let removed: Vec<&str> = removed.iter().filter_map(|name| name.to_str()).collect();
        assert_eq!(
            removed,
            [
                "CLAUDECODE",
                "CLAUDE_CODE_SIMPLE",
                "CLAUDE_CODE_SESSION_ID",
                "CLAUDE_AGENT_SDK_VERSION",
                "ANTHROPIC_BASE_URL"
            ]
        );
    }
}
