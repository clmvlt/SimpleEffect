//! The waveform peaks job: ffmpeg decodes the first sound stream to interleaved `f32` PCM on
//! stdout (at most 2 channels, native sample rate); peaks are accumulated while reading and
//! written as one `SEPK` file per level of [`PEAK_LEVELS`].
//!
//! Progress comes from the samples read against the expected count (stdout carries the PCM, so
//! `-progress pipe:1` is not available).

use std::path::Path;

use se_app::ports::{MediaError, MediaJobSpec, ProgressSink};
use tokio::io::AsyncReadExt;

use crate::{
    cache_layout::{peaks_file, write_atomically},
    peaks::{PEAK_LEVELS, PeakAccumulator, all_levels},
    process::Sidecar,
};

/// Most channels kept (more are downmixed by ffmpeg): enough to draw a waveform.
const MAX_CHANNELS: u16 = 2;
/// Bytes read from ffmpeg at a time.
const CHUNK_BYTES: usize = 64 * 1024;

/// Generates the peaks of `spec` with the ffmpeg at `ffmpeg`.
///
/// # Errors
/// [`MediaError`] when the media has no sound, ffmpeg fails or the cache cannot be written.
pub async fn generate(
    ffmpeg: &Path,
    spec: &MediaJobSpec,
    progress: &dyn ProgressSink,
) -> Result<(), MediaError> {
    let stream = spec.media_info.audio_streams.first().ok_or_else(|| {
        MediaError::Failed("the media has no sound to draw a waveform from".to_owned())
    })?;
    let channels = stream.channels.clamp(1, MAX_CHANNELS);
    let expected_frames = spec
        .media_info
        .duration
        .map(|duration| duration.to_seconds_f64() * f64::from(stream.sample_rate));
    let args = [
        "-hide_banner".to_owned(),
        "-nostdin".to_owned(),
        "-v".to_owned(),
        "error".to_owned(),
        "-i".to_owned(),
        spec.input.to_string_lossy().into_owned(),
        "-map".to_owned(),
        "0:a:0".to_owned(),
        "-vn".to_owned(),
        "-sn".to_owned(),
        "-dn".to_owned(),
        "-ac".to_owned(),
        channels.to_string(),
        "-c:a".to_owned(),
        "pcm_f32le".to_owned(),
        "-f".to_owned(),
        "f32le".to_owned(),
        "pipe:1".to_owned(),
    ];
    let mut process = Sidecar::spawn(ffmpeg, &args, None)?;
    let mut accumulator = PeakAccumulator::new(channels, stream.sample_rate, PEAK_LEVELS[0]);
    if let Some(mut stdout) = process.take_stdout() {
        let mut buffer = vec![0_u8; CHUNK_BYTES];
        let mut pending: Vec<u8> = Vec::with_capacity(CHUNK_BYTES + 4);
        loop {
            let read = stdout.read(&mut buffer).await.map_err(|error| {
                MediaError::Failed(format!("reading ffmpeg's sound failed: {error}"))
            })?;
            if read == 0 {
                break;
            }
            pending.extend_from_slice(&buffer[..read]);
            let whole = pending.len() - pending.len() % 4;
            let samples: Vec<f32> = pending[..whole]
                .as_chunks::<4>()
                .0
                .iter()
                .map(|bytes| f32::from_le_bytes(*bytes))
                .collect();
            accumulator.push(&samples);
            pending.drain(..whole);
            if let Some(expected) = expected_frames.filter(|expected| *expected > 0.0) {
                // Frame counts stay far below 2^52: the conversion is exact.
                #[allow(clippy::cast_precision_loss)]
                let done = accumulator.frames() as f64 / expected;
                progress.report(done.min(0.99));
            }
        }
    }
    process.finish().await?;
    write_levels(&spec.output_dir, accumulator).await?;
    progress.report(1.0);
    Ok(())
}

/// Writes every level, coarsest first: the finest file appearing last marks completion.
async fn write_levels(dir: &Path, accumulator: PeakAccumulator) -> Result<(), MediaError> {
    let io = |error: std::io::Error| MediaError::Io {
        path: dir.to_path_buf(),
        message: error.to_string(),
    };
    tokio::fs::create_dir_all(dir).await.map_err(io)?;
    for level in all_levels(accumulator.finish()).into_iter().rev() {
        let bytes = level.encode().map_err(MediaError::Failed)?;
        write_atomically(&dir.join(peaks_file(level.samples_per_peak)), &bytes)
            .await
            .map_err(io)?;
    }
    Ok(())
}
