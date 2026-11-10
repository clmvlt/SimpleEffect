//! Command handlers, one file per command group. The router maps each `Command` variant to one
//! function here; handlers return the `CommandResult` variant of the same name (ADR-0012).
//!
//! Infallible handlers return `CommandResult`; fallible ones return
//! `Result<CommandResult, ProtocolError>` with a stable code and an actionable message
//! (`se_app::AppError` converts with `?`).

pub mod ai;
pub mod captions;
pub mod diagnostics;
pub mod editing;
pub mod exports;
pub mod frames;
pub mod graphics;
pub mod items;
pub mod library;
pub mod media;
pub mod media_settings;
pub mod projects;
pub mod smart_cuts;
pub mod sound;
pub mod system;
pub mod tasks;
pub mod templates;
pub mod timeline;
pub mod transcription;
pub mod vision;

use std::{path::Path, sync::Arc};

use se_app::{
    BrandKitService, CaptionService, DiagnosticsService, ExportService, FrameRenderService,
    MediaService, ProjectService, ReframeService, SmartCutService, TaskService,
    TemplateCheckService, TemplateLibraryService, TranscriptionService, ports::FontCatalog,
};
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
    /// AI tasks, provider status and settings (also observes project openings and closings).
    pub tasks: Arc<TaskService>,
    /// Bundled and installed fonts.
    pub fonts: Arc<dyn FontCatalog>,
    /// Frames drawn by the renderer for agents (failure reports).
    pub renders: Arc<FrameRenderService>,
    /// Motion template checks run by the renderer (prompt 24).
    pub template_checks: Arc<TemplateCheckService>,
    /// Transcription: models, downloads, jobs, transcripts (also observes projects).
    pub transcription: TranscriptionService,
    /// Smart cuts (prompt 22).
    pub smart_cuts: SmartCutService,
    /// Captions (prompt 23).
    pub captions: CaptionService,
    /// The template library (prompt 25).
    pub library_templates: TemplateLibraryService,
    /// Brand kits (prompt 25).
    pub brand_kits: BrandKitService,
    /// Automatic reframing (prompt 26).
    pub reframing: ReframeService,
    /// Video exports (prompt 27).
    pub exports: ExportService,
    /// Diagnostics archives (prompt 30).
    pub diagnostics: DiagnosticsService,
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
