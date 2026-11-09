//! # se-app
//!
//! Application layer of SimpleEffect: use cases, ports (traits implemented by adapters),
//! the `ProjectActor` (single writer of a project), the `JobQueue` and the media use cases, and
//! the AI task service (`TaskScheduler` of the architecture).
//!
//! Allowed workspace dependencies: `se-domain`, `se-protocol` (see `docs/architecture.md` §3).
//! Adapters (`se-store`, `se-media`, `se-transcribe`, `se-vision`, `se-agents`, `se-export`, `se-mcp`)
//! depend on this crate, never the other way round.
//!
//! ## Map
//! - [`ports`]: [`ports::ProjectRepository`], [`ports::ProjectStore`],
//!   [`ports::AppLibraryRepository`], [`ports::EventSink`], [`ports::Clock`],
//!   [`ports::IdGenerator`], [`ports::MediaProber`], [`ports::MediaFiles`],
//!   [`ports::MediaJobRunner`], [`ports::FrameExtractor`], [`ports::MediaTools`],
//!   [`ports::AnalysisStore`], [`ports::TaskStore`], [`ports::AgentProvider`],
//!   [`ports::AgentWorkspaces`].
//! - [`project_actor`]: [`ProjectActorHandle`], the single writer of an open project.
//! - [`project_service`]: [`ProjectService`], the project use cases behind the protocol.
//! - [`project_view`]: the document as clients see it (snapshot, patches, undo state).
//! - [`jobs`]: [`JobQueue`], background jobs (priorities, concurrency per kind, cancellation).
//! - [`media_service`]: [`MediaService`], media import, relinking, decodability, jobs,
//!   mezzanines, proxies, analyses, settings and the project cache.
//! - [`media_policy`]: which mezzanine and proxy an asset needs (pure decisions).
//! - [`tasks`]: [`TaskService`], AI tasks run in the background by an AI CLI (queue, concurrency
//!   limit, cancellation, retries, follow-ups, persistence, reverts), the AI status and settings.
//! - [`agent_edits`]: [`AgentEditService`], the edits of AI agents (planned, scope-checked,
//!   committed as one AI transaction recorded in the task's changeset).
//! - [`new_project`], [`recovery`]: new documents and crash recovery (pure).
//!
//! ## Rules
//! - No concrete IO: no `std::fs`, `std::process` or network access. IO lives behind ports.
//! - Every port gets an in-memory fake in [`test_support`] (feature `test-support`).

pub mod agent_access;
pub mod agent_edits;
pub mod brand_kits;
pub mod captions;
mod edit_plan;
mod error;
pub mod frame_renders;
pub mod jobs;
pub mod media_policy;
pub mod media_service;
pub mod new_ids;
pub mod new_project;
pub mod ports;
pub mod project_actor;
pub mod project_service;
pub mod project_view;
pub mod recovery;
pub mod smart_cuts;
pub mod tasks;
pub mod template_checks;
pub mod template_library;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
pub mod transcription;

pub use agent_access::{AgentContext, McpAccess, TaskToken, TaskTokenRegistry};
pub use agent_edits::{AgentEditService, EditFailure};
pub use brand_kits::{BrandKitService, BrandKitServiceDeps};
pub use captions::{CaptionService, CaptionServiceDeps};
pub use edit_plan::DEFAULT_IMAGE_DURATION;
pub use error::AppError;
pub use frame_renders::{FrameRenderService, FrameRenderSpec};
pub use jobs::{JobAttempt, JobPriority, JobQueue, JobQueueConfig, JobRequest, JobWork};
pub use media_service::{AssetFiles, ImportListener, MediaService, MediaServiceDeps};
pub use project_actor::{
    ActorDeps, AiCommit, ChangesetItems, DEFAULT_SAVE_DELAY, ProjectActorHandle, ProjectView,
};
pub use project_service::{
    ProjectObserver, ProjectService, ProjectServiceConfig, ProjectServiceDeps,
    animation_preset_infos, graphic_track, item_labels, marker_labels, sequence_labels,
    track_labels, transition_labels,
};
pub use smart_cuts::{SmartCutService, SmartCutServiceDeps};
pub use tasks::{TaskService, TaskServiceDeps};
pub use template_checks::{CheckedTemplate, TemplateCheckService};
pub use template_library::{TemplateLibraryDeps, TemplateLibraryService};
pub use transcription::{TranscriptionService, TranscriptionServiceDeps};
