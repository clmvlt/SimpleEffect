//! Who Claude Code is for the app: id, names shown, how users install it and log in, and what it
//! supports.

use se_protocol::{EffortLevel, ModelOption, ProviderCapabilities, ProviderId, ProviderInfo};

/// Id of this provider (settings value `ai.provider`).
pub const PROVIDER_ID: &str = "claude-code";

/// Name of the executable (`claude.exe` / `claude.cmd` on Windows, found through PATHEXT).
pub(super) const EXECUTABLE: &str = "claude";

/// Installation guide.
const HELP_URL: &str = "https://code.claude.com/docs/en/setup";

/// The provider as clients show it.
pub(super) fn info() -> ProviderInfo {
    ProviderInfo {
        id: ProviderId(PROVIDER_ID.to_owned()),
        name: "Claude Code".to_owned(),
        command: EXECUTABLE.to_owned(),
        login_command: "/login".to_owned(),
        help_url: HELP_URL.to_owned(),
    }
}

/// What Claude Code supports: resumable sessions (`--resume`), images in tool results, the
/// model aliases of `--model` and the levels of `--effort`.
pub(super) fn capabilities() -> ProviderCapabilities {
    let model = |id: &str, label: &str| ModelOption {
        id: id.to_owned(),
        label: label.to_owned(),
    };
    ProviderCapabilities {
        resume_sessions: true,
        image_input: true,
        models: vec![
            model("fable", "Fable"),
            model("opus", "Opus"),
            model("sonnet", "Sonnet"),
            model("haiku", "Haiku"),
        ],
        effort_levels: vec![
            EffortLevel::Low,
            EffortLevel::Medium,
            EffortLevel::High,
            EffortLevel::XHigh,
            EffortLevel::Max,
        ],
    }
}
