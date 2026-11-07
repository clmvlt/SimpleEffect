//! Command handlers, one file per command group. The router maps each `Command` variant to one
//! function here; handlers return the `CommandResult` variant of the same name (ADR-0012).
//!
//! Infallible handlers return `CommandResult`; fallible ones return
//! `Result<CommandResult, ProtocolError>` with a stable code and an actionable message
//! (`se_app::AppError` converts with `?`).

pub mod editing;
pub mod items;
pub mod media;
pub mod media_settings;
pub mod projects;
pub mod sound;
pub mod system;
pub mod timeline;

use std::{path::Path, sync::Arc};

use se_app::{MediaService, ProjectService};
use se_protocol::{ErrorCode, ProtocolError};

use crate::app_paths::AppPaths;

/// What handlers may read. Grows as later prompts wire `se-app` use cases into the core.
#[derive(Clone)]
pub struct HandlerContext {
    /// Version of the running binary.
    pub version: &'static str,
    /// Folders the core writes to.
    pub paths: AppPaths,
    /// Project use cases.
    pub projects: Arc<ProjectService>,
    /// Media use cases (also observes project openings and closings).
    pub media: Arc<MediaService>,
}

/// A path sent by a client, refused unless absolute (a relative path would depend on the
/// core's working directory).
fn absolute_path(text: &str) -> Result<&Path, ProtocolError> {
    let path = Path::new(text);
    if path.is_absolute() {
        Ok(path)
    } else {
        Err(ProtocolError::new(
            ErrorCode::InvalidRequest,
            format!("'{text}' is not an absolute path: send the full path of the folder"),
        ))
    }
}
