//! Ports: the traits se-app needs from the outside world, implemented by adapters and wired by
//! se-core (`docs/architecture.md` §3). Every port has an in-memory fake in
//! [`crate::test_support`] (feature `test-support`).

mod analysis_store;
mod app_library;
mod fonts;
mod media;
mod project_repository;
mod runtime;
mod storage_error;
mod transcode;

pub use analysis_store::{AnalysisStore, StoredAnalysis};
pub use app_library::{AppLibraryRepository, RecentProjectRecord};
pub use fonts::{FontCatalog, FontData};
pub use media::{
    FrameExtractor, FrameRequest, MediaError, MediaFiles, MediaJobOutput, MediaJobRunner,
    MediaJobSpec, MediaProber, MediaTask, MediaTools, ProbedMedia, ProgressSink,
};
pub use project_repository::{JournalEntry, ProjectRepository, ProjectStore};
pub use runtime::{Clock, EventSink, IdGenerator};
pub use storage_error::StorageError;
pub use transcode::{
    AudioEncoding, AudioPlan, Container, TranscodePlan, TranscodeRecipe, VideoCodecTarget,
    VideoEncoding, VideoPlan,
};
