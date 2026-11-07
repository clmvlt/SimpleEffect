//! # se-app
//!
//! Application layer of SimpleEffect: use cases, ports (traits implemented by adapters),
//! the `ProjectActor` (single writer of a project), the `JobQueue` and the media use cases, and
//! later the `TaskScheduler` (prompt 17).
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
//!   [`ports::AnalysisStore`].
//! - [`project_actor`]: [`ProjectActorHandle`], the single writer of an open project.
//! - [`project_service`]: [`ProjectService`], the project use cases behind the protocol.
//! - [`project_view`]: the document as clients see it (snapshot, patches, undo state).
//! - [`jobs`]: [`JobQueue`], background jobs (priorities, concurrency per kind, cancellation).
//! - [`media_service`]: [`MediaService`], media import, relinking, decodability, jobs,
//!   mezzanines, proxies, analyses, settings and the project cache.
//! - [`media_policy`]: which mezzanine and proxy an asset needs (pure decisions).
//! - [`new_project`], [`recovery`]: new documents and crash recovery (pure).
//!
//! ## Rules
//! - No concrete IO: no `std::fs`, `std::process` or network access. IO lives behind ports.
//! - Every port gets an in-memory fake in [`test_support`] (feature `test-support`).

mod error;
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
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

pub use error::AppError;
pub use jobs::{JobAttempt, JobPriority, JobQueue, JobQueueConfig, JobRequest, JobWork};
pub use media_service::{AssetFiles, MediaService, MediaServiceDeps};
pub use project_actor::{ActorDeps, DEFAULT_SAVE_DELAY, ProjectActorHandle, ProjectView};
pub use project_service::{
    ProjectObserver, ProjectService, ProjectServiceConfig, ProjectServiceDeps, marker_labels,
    track_labels,
};
