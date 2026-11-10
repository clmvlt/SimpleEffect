//! Ports of video exports (prompt 27, ADR-0038): where the file of an export is assembled.

use std::path::PathBuf;

use async_trait::async_trait;
use se_domain::FrameRate;
use se_protocol::{AudioExportSettings, ExportContainer, ExportPipeline, VideoExportSettings};

use super::MediaError;

/// What the output of an export must produce.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportOutputPlan {
    /// The final file (replaced when it exists, written only once complete).
    pub path: PathBuf,
    /// The file's container (the muxer ffmpeg uses).
    pub container: ExportContainer,
    /// How the file is put together.
    pub pipeline: ExportPipeline,
    /// The picture, if any.
    pub video: Option<VideoExportSettings>,
    /// The sound, if any.
    pub audio: Option<AudioExportSettings>,
    /// The sequence's frame rate (frames given to ffmpeg).
    pub frame_rate: FrameRate,
}

/// A stream of bytes the renderer writes at positions of its choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportStream {
    /// The muxed file (or the picture file before ffmpeg joins the sound).
    Data,
    /// The WAV of the sound, for ffmpeg.
    Audio,
}

/// Opens outputs of exports (files and ffmpeg in se-media).
#[async_trait]
pub trait ExportOutputs: Send + Sync {
    /// Prepares the output of `plan`: partial files next to the final file, ffmpeg started on the
    /// first frame.
    ///
    /// # Errors
    /// [`MediaError`] when the folder cannot be written or ffmpeg is missing.
    async fn open(&self, plan: ExportOutputPlan) -> Result<Box<dyn ExportOutput>, MediaError>;
}

/// The files of an export being written. Dropping an output that did not finish removes its
/// partial files and stops ffmpeg (cancellation, failure).
#[async_trait]
pub trait ExportOutput: Send {
    /// Writes `bytes` at `position` of `stream`.
    ///
    /// # Errors
    /// [`MediaError::Failed`] when the stream does not exist for this pipeline or the disk fails.
    async fn write(
        &mut self,
        stream: ExportStream,
        position: u64,
        bytes: &[u8],
    ) -> Result<(), MediaError>;

    /// Gives ffmpeg the next frame (RGBA, `width × height × 4` bytes).
    ///
    /// # Errors
    /// [`MediaError::Failed`] when the pipeline takes no frames, the size is wrong or ffmpeg
    /// stopped.
    async fn write_frame(&mut self, rgba: &[u8]) -> Result<(), MediaError>;

    /// Completes the file at its final path (ffmpeg finished, partial files renamed or removed)
    /// and returns its size in bytes.
    ///
    /// # Errors
    /// [`MediaError::Failed`] when ffmpeg fails or the file cannot be moved into place.
    async fn finish(self: Box<Self>) -> Result<u64, MediaError>;
}
