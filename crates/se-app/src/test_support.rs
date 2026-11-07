//! In-memory fakes of every port (feature `test-support`), so use cases and the actor are
//! tested without IO, and deterministic ids and clocks.

mod memory_library;
mod memory_repository;
mod runtime_fakes;

pub use memory_library::InMemoryAppLibrary;
pub use memory_repository::{InMemoryProjectRepository, StoredFolder};
pub use runtime_fakes::{FixedClock, RecordingEventSink, SequentialIds};

/// Locks a fake's state, ignoring poisoning (a panicking test already failed).
fn lock<T>(mutex: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
