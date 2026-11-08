//! Media ports: probing files, fingerprinting them, running media jobs, extracting frames and
//! configuring the tools (implemented by se-media, `docs/architecture.md` §7).

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use se_domain::{
    AnalysisResult, AssetKind, ContentFingerprint, Flicks, MediaInfo, SilenceSettings,
};
use se_protocol::{JobKind, MediaToolsStatus};

use super::transcode::TranscodePlan;

/// Why a media port failed. Messages are actionable: they reach users and AI agents.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MediaError {
    /// Nothing exists at the path.
    #[error("{} does not exist: check the path, or relink the media", path.display())]
    NotFound {
        /// The path.
        path: PathBuf,
    },
    /// The file is not media ffprobe can read.
    #[error("{} is not a video, audio or image file SimpleEffect can read: {reason}", path.display())]
    Unsupported {
        /// The file.
        path: PathBuf,
        /// What the prober said.
        reason: String,
    },
    /// ffmpeg or ffprobe could not be found.
    #[error(
        "{tool} was not found ({searched}): run `pnpm fetch-binaries` in development, \
             reinstall the app, or set the folder holding ffmpeg and ffprobe"
    )]
    ToolsMissing {
        /// `ffmpeg` or `ffprobe`.
        tool: String,
        /// Where it was looked for.
        searched: String,
    },
    /// Reading the file failed (permissions, disk errors…).
    #[error("reading {} failed: {message}", path.display())]
    Io {
        /// The file.
        path: PathBuf,
        /// The OS error.
        message: String,
    },
    /// A media tool ran but failed (its exit status and last error lines).
    #[error("{0}")]
    Failed(String),
}

/// What probing a file revealed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbedMedia {
    /// Video, audio or image.
    pub kind: AssetKind,
    /// Its streams and duration.
    pub info: MediaInfo,
}

/// Reads the streams of a media file (`ffprobe`).
#[async_trait]
pub trait MediaProber: Send + Sync {
    /// Probes the file at `path`.
    ///
    /// # Errors
    /// [`MediaError::NotFound`], [`MediaError::Unsupported`], [`MediaError::ToolsMissing`].
    async fn probe(&self, path: &Path) -> Result<ProbedMedia, MediaError>;
}

/// Facts about media files on disk.
#[async_trait]
pub trait MediaFiles: Send + Sync {
    /// Whether a regular file exists at `path`.
    async fn is_file(&self, path: &Path) -> bool;

    /// The content fingerprint of the file at `path` (size, modification time, xxh3 of its first
    /// and last mebibyte).
    ///
    /// # Errors
    /// [`MediaError::NotFound`], [`MediaError::Io`].
    async fn fingerprint(&self, path: &Path) -> Result<ContentFingerprint, MediaError>;

    /// Bytes used by the files under `dir` (0 when it does not exist).
    async fn directory_size(&self, dir: &Path) -> u64;

    /// Deletes `dir` and everything in it, going on past files that cannot be deleted (in use on
    /// Windows). Nothing happens when it does not exist.
    ///
    /// # Errors
    /// [`MediaError::Io`] naming the first entry that could not be deleted (the others were).
    async fn remove_directory(&self, dir: &Path) -> Result<(), MediaError>;

    /// The bytes of a small derived file (an extracted frame), read whole.
    ///
    /// # Errors
    /// [`MediaError::NotFound`], [`MediaError::Io`].
    async fn read_file(&self, path: &Path) -> Result<Vec<u8>, MediaError>;
}

/// Receives the progress of a running job.
pub trait ProgressSink: Send + Sync {
    /// Reports the fraction done, in `[0, 1]`.
    fn report(&self, fraction: f64);
}

/// What a media job produces.
#[derive(Debug, Clone, PartialEq)]
pub enum MediaTask {
    /// The thumbnail sheet, its layout and the poster (cache files).
    Thumbnails,
    /// Waveform peaks at every level (cache files).
    WaveformPeaks,
    /// A mezzanine or proxy file (`plan.output`).
    Transcode(TranscodePlan),
    /// Silences, loudness and peaks of the first sound stream.
    AudioAnalysis {
        /// What counts as silence.
        silence: SilenceSettings,
    },
    /// Shot changes of the picture.
    SceneDetection {
        /// Scene score above which a frame starts a new shot.
        threshold: f64,
    },
}

impl MediaTask {
    /// The job kind (decides concurrency and how clients label it).
    #[must_use]
    pub fn kind(&self) -> JobKind {
        match self {
            Self::Thumbnails => JobKind::Thumbnails,
            Self::WaveformPeaks => JobKind::WaveformPeaks,
            Self::Transcode(plan) => plan.job_kind(),
            Self::AudioAnalysis { .. } => JobKind::AudioAnalysis,
            Self::SceneDetection { .. } => JobKind::SceneDetection,
        }
    }
}

/// One media job to run: what to produce, from which file, into which cache folder.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaJobSpec {
    /// What to produce.
    pub task: MediaTask,
    /// The original file.
    pub input: PathBuf,
    /// The asset's cache folder (created by the runner if needed).
    pub output_dir: PathBuf,
    /// Kind of the asset.
    pub asset_kind: AssetKind,
    /// Its probed streams (sizes, duration, channels).
    pub media_info: MediaInfo,
}

/// What a successful job produced besides files.
#[derive(Debug, Clone, PartialEq)]
pub enum MediaJobOutput {
    /// Only files (in the cache folder or at the plan's output path).
    Files,
    /// An analysis result, to store in the project database.
    Analysis(AnalysisResult),
}

/// Produces derived files and analyses of media with ffmpeg.
#[async_trait]
pub trait MediaJobRunner: Send + Sync {
    /// Whether the outputs of a cache job (`thumbnails`, `waveformPeaks`) are already complete in
    /// `output_dir`. Always `false` for other kinds: mezzanines and proxies are tracked by the
    /// document, analyses by the project database.
    async fn outputs_ready(&self, kind: JobKind, output_dir: &Path) -> bool;

    /// Runs the job, reporting progress. Dropping the returned future cancels it: every process
    /// it started must stop and no partial output may look complete.
    ///
    /// # Errors
    /// [`MediaError`] when a tool is missing or fails.
    async fn run(
        &self,
        spec: &MediaJobSpec,
        progress: &dyn ProgressSink,
    ) -> Result<MediaJobOutput, MediaError>;
}

/// One picture to extract from a media file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameRequest {
    /// The media file.
    pub input: PathBuf,
    /// Its kind (images ignore `time`).
    pub asset_kind: AssetKind,
    /// Source time of the picture, inside the media.
    pub time: Flicks,
    /// Largest width of the JPEG (never enlarged; the aspect ratio is kept).
    pub max_width: u32,
    /// Where to write the JPEG (atomically).
    pub output: PathBuf,
}

/// Extracts single pictures of media as JPEG files (for AI tools that look at footage).
#[async_trait]
pub trait FrameExtractor: Send + Sync {
    /// Writes the picture of `request` to `request.output`.
    ///
    /// # Errors
    /// [`MediaError`] when a tool is missing or fails.
    async fn extract_frame(&self, request: &FrameRequest) -> Result<(), MediaError>;
}

/// The ffmpeg tools themselves: where they are looked for, and what they can do.
#[async_trait]
pub trait MediaTools: Send + Sync {
    /// Searches `dir` first (the user's choice), or only the bundled tools when `None`.
    fn set_tools_dir(&self, dir: Option<PathBuf>);

    /// The tools found, their version and the hardware encoders that work.
    async fn status(&self) -> MediaToolsStatus;
}
