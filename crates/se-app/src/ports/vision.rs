//! Ports of automatic reframing (prompt 26, ADR-0037): pictures sampled from a video
//! ([`FrameSampler`], se-media) and the faces found in them ([`SubjectDetector`], se-vision).
//! Pictures never leave the machine: they go from ffmpeg's output to the detector in memory.

use std::path::PathBuf;

use async_trait::async_trait;
use se_domain::{FaceBox, Flicks};
use se_protocol::VisionStatus;
use thiserror::Error;
use tokio::sync::mpsc;

use super::MediaError;

/// One upright picture sampled from a video, scaled down: 8-bit blue-green-red pixels, row by
/// row from the top, without padding (`width × height × 3` bytes).
#[derive(Clone, PartialEq, Eq)]
pub struct SampledFrame {
    /// Source time of the picture (media time 0 is the first frame).
    pub time: Flicks,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// The pixels.
    pub pixels: Vec<u8>,
}

impl std::fmt::Debug for SampledFrame {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SampledFrame")
            .field("time", &self.time)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("bytes", &self.pixels.len())
            .finish()
    }
}

/// Pictures to sample from a video.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameSampleRequest {
    /// The file to read (the original, or a ready proxy: same timing).
    pub source: PathBuf,
    /// Time between two samples.
    pub interval: Flicks,
    /// Width of the sampled pictures (even).
    pub width: u32,
    /// Height of the sampled pictures (even).
    pub height: u32,
    /// Try hardware decoding first (a failed start is redone in software).
    pub hardware_decoding: bool,
}

/// Samples pictures from videos (se-media, ffmpeg).
#[async_trait]
pub trait FrameSampler: Send + Sync {
    /// Decodes `request.source` and sends one upright picture per interval to `frames`, in time
    /// order, starting at the first frame. Returns early, without error, once the receiver is
    /// dropped (the consumer stopped).
    ///
    /// # Errors
    /// [`MediaError`]: missing tools, no picture stream, ffmpeg failures.
    async fn sample_frames(
        &self,
        request: &FrameSampleRequest,
        frames: mpsc::Sender<SampledFrame>,
    ) -> Result<(), MediaError>;
}

/// Why faces could not be detected.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum VisionError {
    /// The detector cannot run on this machine (missing library, unsupported system).
    #[error("face detection is unavailable: {reason}")]
    Unavailable {
        /// What is missing and what to do.
        reason: String,
    },
    /// The detector failed on a picture.
    #[error("face detection failed: {message}")]
    Failed {
        /// The inference library's message.
        message: String,
    },
}

/// Finds faces in pictures (se-vision, ONNX Runtime).
#[async_trait]
pub trait SubjectDetector: Send + Sync {
    /// The model's name, stored with results (`yunet-2026may`): results of another model are
    /// computed again.
    fn model(&self) -> &str;

    /// Whether detection can run here (loads the inference library the first time).
    async fn status(&self) -> VisionStatus;

    /// The faces of each frame, in the frames' order, most confident first, boxes normalised in
    /// the frame. CPU-heavy: runs on the blocking pool.
    ///
    /// # Errors
    /// [`VisionError::Unavailable`] when the detector cannot run, [`VisionError::Failed`] when
    /// inference fails.
    async fn detect(&self, frames: Vec<SampledFrame>) -> Result<Vec<Vec<FaceBox>>, VisionError>;
}
