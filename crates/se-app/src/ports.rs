//! Ports: the traits se-app needs from the outside world, implemented by adapters and wired by
//! se-core (`docs/architecture.md` §3). Every port has an in-memory fake in
//! [`crate::test_support`] (feature `test-support`).

mod agents;
mod analysis_store;
mod app_library;
mod fonts;
mod media;
mod project_repository;
mod renderer;
mod runtime;
mod storage_error;
mod task_store;
mod transcode;

pub use agents::{
    AgentError, AgentProvider, AgentRun, AgentRunSpec, AgentSession, AgentWorkspaces, CancelHandle,
    CancelSignal, McpEndpoint, ProviderInstallation,
};
pub use analysis_store::{AnalysisStore, StoredAnalysis};
pub use app_library::{AppLibraryRepository, RecentProjectRecord};
pub use fonts::{FontCatalog, FontData};
pub use media::{
    FrameExtractor, FrameRequest, MediaError, MediaFiles, MediaJobOutput, MediaJobRunner,
    MediaJobSpec, MediaProber, MediaTask, MediaTools, ProbedMedia, ProgressSink,
};
pub use project_repository::{JournalEntry, ProjectRepository, ProjectStore};
pub use renderer::RendererLink;
pub use runtime::{Clock, EventSink, IdGenerator, TokenSource};
pub use storage_error::StorageError;
pub use task_store::TaskStore;
pub use transcode::{
    AudioEncoding, AudioPlan, Container, TranscodePlan, TranscodeRecipe, VideoCodecTarget,
    VideoEncoding, VideoPlan,
};
