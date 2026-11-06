//! Commands a client asks `se-core` to execute.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A command sent inside a [`crate::RequestEnvelope`], tagged by `kind` (camelCase).
///
/// Every variant has a [`crate::CommandResult`] variant **with the same name**: that pairing is
/// how the TypeScript client derives the result type of each command without a hand-written table
/// (ADR-0012). Adding a command: see the recipe in `crates/se-protocol/CLAUDE.md`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Command {
    /// Liveness check. Answered by [`crate::CommandResult::Ping`].
    Ping,
    /// Describes the running core (version, platform, folders). Answered by
    /// [`crate::CommandResult::GetCoreInfo`].
    GetCoreInfo,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_are_tagged_by_camel_case_kind() {
        assert_eq!(
            serde_json::to_string(&Command::GetCoreInfo).unwrap(),
            r#"{"kind":"getCoreInfo"}"#
        );
        let parsed: Command = serde_json::from_str(r#"{"kind":"ping"}"#).unwrap();
        assert_eq!(parsed, Command::Ping);
    }
}
