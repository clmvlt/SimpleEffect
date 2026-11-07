//! # se-app
//!
//! Application layer of SimpleEffect: use cases, ports (traits implemented by adapters),
//! the `ProjectActor` (single writer of a project), and later the `JobQueue` (prompt 08) and the
//! `TaskScheduler` (prompt 17).
//!
//! Allowed workspace dependencies: `se-domain`, `se-protocol` (see `docs/architecture.md` §3).
//! Adapters (`se-store`, `se-media`, `se-transcribe`, `se-vision`, `se-agents`, `se-export`, `se-mcp`)
//! depend on this crate, never the other way round.
//!
//! ## Map
//! - [`ports`]: [`ports::ProjectRepository`], [`ports::ProjectStore`],
//!   [`ports::AppLibraryRepository`], [`ports::EventSink`], [`ports::Clock`],
//!   [`ports::IdGenerator`].
//! - [`project_actor`]: [`ProjectActorHandle`], the single writer of an open project.
//! - [`project_service`]: [`ProjectService`], the project use cases behind the protocol.
//! - [`project_view`]: the document as clients see it (snapshot, patches, undo state).
//! - [`new_project`], [`recovery`]: new documents and crash recovery (pure).
//!
//! ## Rules
//! - No concrete IO: no `std::fs`, `std::process` or network access. IO lives behind ports.
//! - Every port gets an in-memory fake in [`test_support`] (feature `test-support`).

mod error;
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
pub use project_actor::{ActorDeps, DEFAULT_SAVE_DELAY, ProjectActorHandle, ProjectView};
pub use project_service::{ProjectService, ProjectServiceConfig, ProjectServiceDeps};
