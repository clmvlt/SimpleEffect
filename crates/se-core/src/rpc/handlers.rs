//! Command handlers, one file per command group. The router maps each `Command` variant to one
//! function here; handlers return the `CommandResult` variant of the same name (ADR-0012).
//!
//! Infallible handlers return `CommandResult`; fallible ones return
//! `Result<CommandResult, ProtocolError>` with a stable code and an actionable message.

pub mod system;

use crate::app_paths::AppPaths;

/// What handlers may read. Grows as later prompts wire `se-app` use cases into the core.
#[derive(Debug, Clone)]
pub struct HandlerContext {
    /// Version of the running binary.
    pub version: &'static str,
    /// Folders the core writes to.
    pub paths: AppPaths,
}
