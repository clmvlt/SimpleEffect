//! The command line of a Claude Code run (`docs/architecture.md` §9.2; flags validated in
//! `CLAUDE.md` next to this file).
//!
//! The prompt goes to stdin and the system instructions to a file, never on the command line:
//! on Windows `claude` is often a `.cmd` shim, and Rust refuses to pass arguments holding line
//! breaks to batch files (they cannot be escaped safely).

use std::path::Path;

use se_app::ports::{AgentRunSpec, AgentSession};
use se_protocol::EffortLevel;

/// Name of the system instructions file written in the run's working folder.
pub(super) const INSTRUCTIONS_FILE: &str = ".simpleeffect-instructions.md";

/// Name of the MCP server in the CLI (tools become `mcp__simpleeffect__<tool>`).
const MCP_SERVER_NAME: &str = "simpleeffect";

/// Arguments of `claude --version`.
pub(super) const VERSION_ARGUMENTS: &[&str] = &["--version"];

/// Arguments of `claude auth status --json`.
pub(super) const AUTH_STATUS_ARGUMENTS: &[&str] = &["auth", "status", "--json"];

/// The arguments of a run of `spec` whose instructions are in `instructions_file`.
pub(super) fn run_arguments(spec: &AgentRunSpec, instructions_file: &Path) -> Vec<String> {
    let mut args: Vec<String> = [
        // Headless: one prompt (from stdin), then exit.
        "-p",
        "--output-format",
        "stream-json",
        // Required by stream-json in print mode.
        "--verbose",
        // No built-in tool at all: no shell, no file access, no web (golden rule 2).
        "--tools",
        "",
        // Anything that would ask for a permission is refused (nobody can answer).
        "--permission-prompts",
        "none",
        // Isolation from the user's own configuration: no settings files (hooks, permissions,
        // default model), no skills or slash commands, no MCP servers but ours, no auto-memory.
        "--setting-sources",
        "",
        "--restricted",
        "--disable-slash-commands",
        "--strict-mcp-config",
    ]
    .map(str::to_owned)
    .to_vec();
    args.push("--append-system-prompt-file".to_owned());
    args.push(instructions_file.display().to_string());
    if let Some(endpoint) = &spec.mcp {
        args.push("--mcp-config".to_owned());
        args.push(mcp_config(&endpoint.url, &endpoint.bearer_token));
        args.push("--allowedTools".to_owned());
        args.push(format!("mcp__{MCP_SERVER_NAME}__*"));
    }
    if let Some(model) = &spec.model {
        args.push("--model".to_owned());
        args.push(model.clone());
    }
    if let Some(effort) = spec.effort {
        args.push("--effort".to_owned());
        args.push(effort_flag(effort).to_owned());
    }
    match &spec.session {
        AgentSession::Start(id) => {
            args.push("--session-id".to_owned());
            args.push(id.clone());
        }
        AgentSession::Resume(id) => {
            args.push("--resume".to_owned());
            args.push(id.clone());
        }
    }
    args
}

/// The arguments with secrets replaced, for logs.
pub(super) fn redacted(args: &[String]) -> Vec<String> {
    let mut redacted = Vec::with_capacity(args.len());
    let mut hide_next = false;
    for arg in args {
        if hide_next {
            redacted.push("<redacted>".to_owned());
        } else {
            redacted.push(arg.clone());
        }
        hide_next = arg == "--mcp-config";
    }
    redacted
}

/// The inline MCP configuration: se-core's `/mcp` with the task's bearer token.
fn mcp_config(url: &str, token: &str) -> String {
    serde_json::json!({
        "mcpServers": {
            MCP_SERVER_NAME: {
                "type": "http",
                "url": url,
                "headers": { "Authorization": format!("Bearer {token}") }
            }
        }
    })
    .to_string()
}

const fn effort_flag(effort: EffortLevel) -> &'static str {
    match effort {
        EffortLevel::Low => "low",
        EffortLevel::Medium => "medium",
        EffortLevel::High => "high",
        EffortLevel::XHigh => "xhigh",
        EffortLevel::Max => "max",
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use se_app::ports::McpEndpoint;

    use super::*;

    fn spec(session: AgentSession) -> AgentRunSpec {
        AgentRunSpec {
            prompt: "Cut the silences\nthen add captions".to_owned(),
            system_instructions: "# Role\n".to_owned(),
            mcp: None,
            model: Some("opus".to_owned()),
            effort: Some(EffortLevel::XHigh),
            session,
            working_dir: PathBuf::from("/work"),
        }
    }

    fn value_after<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
        let index = args.iter().position(|arg| arg == flag)?;
        args.get(index + 1).map(String::as_str)
    }

    #[test]
    fn new_runs_choose_their_session_and_carry_no_prompt() {
        let args = run_arguments(
            &spec(AgentSession::Start("6f1c2b8e".to_owned())),
            Path::new("/work/.simpleeffect-instructions.md"),
        );
        assert_eq!(value_after(&args, "--session-id"), Some("6f1c2b8e"));
        assert_eq!(value_after(&args, "--tools"), Some(""));
        assert_eq!(value_after(&args, "--model"), Some("opus"));
        assert_eq!(value_after(&args, "--effort"), Some("xhigh"));
        assert_eq!(value_after(&args, "--setting-sources"), Some(""));
        assert!(args.iter().all(|arg| !arg.contains('\n')), "{args:?}");
        assert!(!args.contains(&"--resume".to_owned()));
        assert!(!args.contains(&"--mcp-config".to_owned()));
    }

    #[test]
    fn follow_ups_resume_and_mcp_endpoints_are_inline_and_redacted() {
        let mut spec = spec(AgentSession::Resume("6f1c2b8e".to_owned()));
        spec.mcp = Some(McpEndpoint {
            url: "http://127.0.0.1:4100/mcp".to_owned(),
            bearer_token: "secret-token".to_owned(),
        });
        let args = run_arguments(&spec, Path::new("/work/i.md"));
        assert_eq!(value_after(&args, "--resume"), Some("6f1c2b8e"));
        let config: serde_json::Value =
            serde_json::from_str(value_after(&args, "--mcp-config").unwrap()).unwrap();
        assert_eq!(
            config["mcpServers"]["simpleeffect"]["headers"]["Authorization"],
            "Bearer secret-token"
        );
        assert_eq!(
            value_after(&args, "--allowedTools"),
            Some("mcp__simpleeffect__*")
        );
        let logged = redacted(&args).join(" ");
        assert!(!logged.contains("secret-token"));
    }
}
