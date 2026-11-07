//! Decoding the first sound stream to interleaved `f32` PCM on ffmpeg's stdout (at most 2
//! channels, native sample rate), read in chunks: the input of waveform peaks and audio analyses.

use std::path::Path;

use se_app::ports::MediaError;
use se_domain::MediaInfo;
use tokio::{io::AsyncReadExt, process::ChildStdout};

use crate::process::Sidecar;

/// Most channels kept (more are downmixed by ffmpeg): enough to draw and measure.
pub const MAX_CHANNELS: u16 = 2;
/// Bytes read from ffmpeg at a time.
const CHUNK_BYTES: usize = 64 * 1024;

/// The format of the PCM a media's first sound stream decodes to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PcmFormat {
    /// Interleaved channels (1 or 2).
    pub channels: u16,
    /// Samples per second per channel.
    pub sample_rate: u32,
    /// Frames expected from the media's duration, when known (for progress).
    pub expected_frames: Option<f64>,
}

impl PcmFormat {
    /// The format for `info`'s first sound stream.
    ///
    /// # Errors
    /// [`MediaError::Failed`] when the media has no sound.
    pub fn of(info: &MediaInfo, purpose: &str) -> Result<Self, MediaError> {
        let stream = info
            .audio_streams
            .first()
            .ok_or_else(|| MediaError::Failed(format!("the media has no sound to {purpose}")))?;
        Ok(Self {
            channels: stream.channels.clamp(1, MAX_CHANNELS),
            sample_rate: stream.sample_rate,
            expected_frames: info
                .duration
                .map(|duration| duration.to_seconds_f64() * f64::from(stream.sample_rate))
                .filter(|frames| *frames > 0.0),
        })
    }

    /// The fraction done after `frames` frames, below 1 (the end is reported by the job).
    #[must_use]
    pub fn progress(self, frames: u64) -> Option<f64> {
        // Frame counts stay far below 2^52: the conversion is exact.
        #[allow(clippy::cast_precision_loss)]
        self.expected_frames
            .map(|expected| (frames as f64 / expected).min(0.99))
    }
}

/// Starts ffmpeg decoding `input`'s first sound stream to `format` on stdout.
///
/// # Errors
/// [`MediaError::Failed`] when ffmpeg cannot start.
pub fn spawn_decoder(
    ffmpeg: &Path,
    input: &Path,
    format: PcmFormat,
) -> Result<Sidecar, MediaError> {
    let args = [
        "-hide_banner".to_owned(),
        "-nostdin".to_owned(),
        "-v".to_owned(),
        "error".to_owned(),
        "-i".to_owned(),
        input.to_string_lossy().into_owned(),
        "-map".to_owned(),
        "0:a:0".to_owned(),
        "-vn".to_owned(),
        "-sn".to_owned(),
        "-dn".to_owned(),
        "-ac".to_owned(),
        format.channels.to_string(),
        "-c:a".to_owned(),
        "pcm_f32le".to_owned(),
        "-f".to_owned(),
        "f32le".to_owned(),
        "pipe:1".to_owned(),
    ];
    Sidecar::spawn(ffmpeg, &args, None)
}

/// Reads ffmpeg's stdout as `f32` samples, chunk by chunk.
pub struct PcmReader {
    stdout: Option<ChildStdout>,
    buffer: Vec<u8>,
    pending: Vec<u8>,
}

impl PcmReader {
    /// A reader of `process`'s stdout (taken from it).
    pub fn new(process: &mut Sidecar) -> Self {
        Self {
            stdout: process.take_stdout(),
            buffer: vec![0; CHUNK_BYTES],
            pending: Vec::with_capacity(CHUNK_BYTES + 4),
        }
    }

    /// The next samples (interleaved), or `None` at the end of the stream.
    ///
    /// # Errors
    /// [`MediaError::Failed`] when reading the pipe fails.
    pub async fn next_samples(&mut self) -> Result<Option<Vec<f32>>, MediaError> {
        let Some(stdout) = self.stdout.as_mut() else {
            return Ok(None);
        };
        let read = stdout.read(&mut self.buffer).await.map_err(|error| {
            MediaError::Failed(format!("reading ffmpeg's sound failed: {error}"))
        })?;
        if read == 0 {
            return Ok(None);
        }
        self.pending.extend_from_slice(&self.buffer[..read]);
        let whole = self.pending.len() - self.pending.len() % 4;
        let samples = self.pending[..whole]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|bytes| f32::from_le_bytes(*bytes))
            .collect();
        self.pending.drain(..whole);
        Ok(Some(samples))
    }
}
