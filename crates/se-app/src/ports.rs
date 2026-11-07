//! Ports: the traits se-app needs from the outside world, implemented by adapters and wired by
//! se-core (`docs/architecture.md` §3). Every port has an in-memory fake in
//! [`crate::test_support`] (feature `test-support`).

mod app_library;
mod project_repository;
mod runtime;
mod storage_error;

pub use app_library::{AppLibraryRepository, RecentProjectRecord};
pub use project_repository::{JournalEntry, ProjectRepository, ProjectStore};
pub use runtime::{Clock, EventSink, IdGenerator};
pub use storage_error::StorageError;
