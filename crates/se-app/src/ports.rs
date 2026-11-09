//! Ports: the traits se-app needs from the outside world, implemented by adapters and wired by
//! se-core (`docs/architecture.md` §3). Every port has an in-memory fake in
//! [`crate::test_support`] (feature `test-support`).

mod agents;
mod analysis_store;
mod app_library;
mod brand_kit_library;
mod fonts;
mod media;
mod project_repository;
mod renderer;
mod runtime;
mod storage_error;
mod subtitles;
mod task_store;
mod template_library;
mod transcode;
mod transcript_store;
mod transcription;
mod vision;

pub use agents::{
    AgentError, AgentProvider, AgentRun, AgentRunSpec, AgentSession, AgentWorkspaces, CancelHandle,
    CancelSignal, McpEndpoint, ProviderInstallation,
};
pub use analysis_store::{AnalysisStore, StoredAnalysis};
pub use app_library::{AppLibraryRepository, RecentProjectRecord};
pub use brand_kit_library::BrandKitLibrary;
pub use fonts::{FontCatalog, FontData};
pub use media::{
    FrameExtractor, FrameRequest, MediaError, MediaFiles, MediaJobOutput, MediaJobRunner,
    MediaJobSpec, MediaProber, MediaTask, MediaTools, ProbedMedia, ProgressSink,
};
pub use project_repository::{ChangesetWrite, JournalEntry, ProjectRepository, ProjectStore};
pub use renderer::RendererLink;
pub use runtime::{Clock, EventSink, IdGenerator, TokenSource};
pub use storage_error::StorageError;
pub use subtitles::{SubtitleCue, SubtitleError, SubtitleFiles};
pub use task_store::TaskStore;
pub use template_library::TemplateLibrary;
pub use transcode::{
    AudioEncoding, AudioPlan, Container, TranscodePlan, TranscodeRecipe, VideoCodecTarget,
    VideoEncoding, VideoPlan,
};
pub use transcript_store::{StoredTranscript, TranscriptHit, TranscriptStore};
pub use transcription::{
    DownloadError, DownloadSink, DownloadSpec, FileDownloader, GpuRuntimeOffer, ModelChoice,
    ModelFile, ModelFiles, SpeechAudioExtractor, Transcriber, TranscriberEnvironment,
    TranscriberError, TranscriptionOutput, TranscriptionRequest,
};
pub use vision::{FrameSampleRequest, FrameSampler, SampledFrame, SubjectDetector, VisionError};
