//! Transcription ports (prompt 21, ADR-0031): the speech audio extractor (ffmpeg, se-media), the
//! transcriber (whisper.cpp, se-transcribe), the downloader of models and the model files.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use se_domain::Transcript;
use se_protocol::TranscriptionBackend;

use super::{MediaError, ProgressSink};

/// Writes the first sound stream of a file as 16 kHz mono 16-bit PCM WAV (what whisper reads).
#[async_trait]
pub trait SpeechAudioExtractor: Send + Sync {
    /// Extracts `source`'s sound into `output` (written under a temporary name, then renamed).
    ///
    /// # Errors
    /// [`MediaError`]: missing tools, no sound stream, ffmpeg failures.
    async fn extract_speech_audio(
        &self,
        source: &Path,
        output: &Path,
        progress: &dyn ProgressSink,
    ) -> Result<(), MediaError>;
}

/// A model file given to the transcriber.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelChoice {
    /// The model's id (`large-v3-turbo`, or a custom model's file name), recorded in transcripts.
    pub id: String,
    /// Absolute path of its ggml file.
    pub path: PathBuf,
    /// The alignment heads preset improving word timings (`large.v3.turbo`), for catalogue models.
    pub alignment: Option<String>,
}

/// One transcription to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriptionRequest {
    /// The 16 kHz mono WAV to transcribe.
    pub audio: PathBuf,
    /// An empty folder for the run's outputs (removed by the caller).
    pub work_dir: PathBuf,
    /// The model.
    pub model: ModelChoice,
    /// The voice detection model, when speech detection is on.
    pub voice_model: Option<PathBuf>,
    /// The spoken language (ISO 639), or `None` to detect it.
    pub language: Option<String>,
    /// Names and terms to spell this way (one line), or `None`.
    pub vocabulary: Option<String>,
    /// Write hesitations (« euh », « um ») instead of cleaning them away, so smart cuts can find
    /// them (prompt 22).
    pub keep_hesitations: bool,
    /// Whether the GPU may be used.
    pub use_gpu: bool,
}

/// What a transcription produced.
#[derive(Debug, Clone, PartialEq)]
pub struct TranscriptionOutput {
    /// The words, in the audio's time.
    pub transcript: Transcript,
    /// Where it ran.
    pub backend: TranscriptionBackend,
}

/// Why a transcription could not run.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TranscriberError {
    /// No `whisper-cli` was found.
    #[error(
        "whisper.cpp (whisper-cli) was not found; looked in: {searched}. Reinstall SimpleEffect, \
         or choose a folder holding whisper-cli in « Réglages → Transcription »"
    )]
    WhisperMissing {
        /// Where it looked.
        searched: String,
    },
    /// The program exists but cannot start (missing system libraries).
    #[error("whisper.cpp cannot start: {0}")]
    CannotStart(String),
    /// The run failed.
    #[error("{0}")]
    Failed(String),
}

/// What the transcriber found on this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriberEnvironment {
    /// The `whisper-cli` that will run, or why none.
    pub whisper: Result<PathBuf, TranscriberError>,
    /// The GPU detected, if any (its name).
    pub gpu_name: Option<String>,
    /// Where a transcription with the GPU allowed should run.
    pub backend: TranscriptionBackend,
    /// The GPU pack this platform offers, if any.
    pub gpu_runtime: Option<GpuRuntimeOffer>,
}

/// The GPU pack of a platform (Windows: the CUDA build of whisper.cpp).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuRuntimeOffer {
    /// What it is (`CUDA 12.4`).
    pub name: String,
    /// Download size in bytes.
    pub download_size: u64,
    /// Whether it is installed.
    pub installed: bool,
}

/// whisper.cpp behind a port (implemented by se-transcribe).
#[async_trait]
pub trait Transcriber: Send + Sync {
    /// Uses the user's folder holding `whisper-cli` first (`None`: bundled builds).
    fn set_whisper_dir(&self, dir: Option<PathBuf>);

    /// The program, GPU and GPU pack of this machine.
    async fn environment(&self) -> TranscriberEnvironment;

    /// Transcribes `request.audio`; dropping the future kills the process.
    ///
    /// # Errors
    /// [`TranscriberError`].
    async fn transcribe(
        &self,
        request: &TranscriptionRequest,
        progress: &dyn ProgressSink,
    ) -> Result<TranscriptionOutput, TranscriberError>;

    /// Downloads, checks and unpacks the GPU pack; dropping the future cancels it.
    ///
    /// # Errors
    /// [`DownloadError`]; `NotOffered` when this platform has none.
    async fn install_gpu_runtime(&self, progress: &dyn DownloadSink) -> Result<(), DownloadError>;

    /// Deletes the GPU pack (nothing when absent).
    ///
    /// # Errors
    /// [`DownloadError::Io`].
    async fn remove_gpu_runtime(&self) -> Result<(), DownloadError>;
}

/// A file to download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadSpec {
    /// `https://` address.
    pub url: String,
    /// Expected SHA-256 (64 lowercase hex digits).
    pub sha256: String,
    /// Expected size in bytes.
    pub size: u64,
    /// Where the checked file goes (written as `<name>.part`, resumed, then renamed).
    pub destination: PathBuf,
}

/// Progress of a download.
pub trait DownloadSink: Send + Sync {
    /// `received` of `total` bytes arrived.
    fn received(&self, received: u64, total: u64);

    /// Every byte arrived: checking (and unpacking) starts.
    fn verifying(&self);
}

/// Why a download failed (nothing was installed).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DownloadError {
    /// The server or the network failed.
    #[error("the download from {url} failed: {message}. Check the connection and try again")]
    Network {
        /// The address.
        url: String,
        /// What failed.
        message: String,
    },
    /// The file is not the expected one.
    #[error(
        "the file downloaded from {url} is not the expected one (size or SHA-256 differs); it was \
         deleted. Try again later"
    )]
    Mismatch {
        /// The address.
        url: String,
    },
    /// This platform offers no such file.
    #[error("there is no GPU pack for this platform")]
    NotOffered,
    /// A file could not be written.
    #[error("{0}")]
    Io(String),
}

/// Downloads files with checked size and SHA-256 (implemented by se-transcribe, the only crate
/// that reaches the network, and only after the user agreed).
#[async_trait]
pub trait FileDownloader: Send + Sync {
    /// Downloads `spec`, resuming a previous `.part`; dropping the future stops it (the `.part`
    /// stays for the next attempt).
    ///
    /// # Errors
    /// [`DownloadError`].
    async fn download(
        &self,
        spec: &DownloadSpec,
        progress: &dyn DownloadSink,
    ) -> Result<(), DownloadError>;
}

/// A file of the models folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelFile {
    /// Its name.
    pub file_name: String,
    /// Its size in bytes.
    pub size: u64,
}

/// The models folder (implemented by se-transcribe).
#[async_trait]
pub trait ModelFiles: Send + Sync {
    /// The complete `.bin` files of `dir` (not `.part` files); empty when it does not exist.
    async fn list(&self, dir: &Path) -> Vec<ModelFile>;

    /// The SHA-256 of a file (lowercase hex).
    ///
    /// # Errors
    /// [`MediaError::Io`] / [`MediaError::NotFound`].
    async fn sha256(&self, path: &Path) -> Result<String, MediaError>;

    /// Whether `path` starts like a whisper.cpp (ggml) model.
    async fn is_whisper_model(&self, path: &Path) -> bool;

    /// Copies `source` to `destination` (temporary name, then renamed).
    ///
    /// # Errors
    /// [`MediaError::Io`].
    async fn copy(&self, source: &Path, destination: &Path) -> Result<(), MediaError>;

    /// Deletes a file (nothing when absent).
    ///
    /// # Errors
    /// [`MediaError::Io`].
    async fn remove(&self, path: &Path) -> Result<(), MediaError>;
}
