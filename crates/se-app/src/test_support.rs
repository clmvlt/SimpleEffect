//! In-memory fakes of every port (feature `test-support`), so use cases and the actor are
//! tested without IO, and deterministic ids and clocks.

mod font_fakes;
mod job_fakes;
mod media_fakes;
mod media_testbed;
mod memory_library;
mod memory_repository;
mod runtime_fakes;

pub use font_fakes::{FAKE_FONT_BYTES, FakeFontCatalog};
pub use job_fakes::{
    FakeFrameExtractor, FakeJobRunner, FakeMediaTools, fake_audio_analysis, fake_scene_detection,
};
pub use media_fakes::{
    FAKE_FILE_BYTES, FakeMedia, fingerprint, probed_audio, probed_prores, probed_video,
};
pub use media_testbed::{MediaTestbed, settle};
pub use memory_library::InMemoryAppLibrary;
pub use memory_repository::{InMemoryProjectRepository, StoredFolder};
pub use runtime_fakes::{FixedClock, RecordingEventSink, SequentialIds};

/// Locks a fake's state, ignoring poisoning (a panicking test already failed).
fn lock<T>(mutex: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
