//! The waveform peaks job: ffmpeg decodes the first sound stream to interleaved `f32` PCM on
//! stdout (`pcm.rs`); peaks are accumulated while reading and written as one `SEPK` file per
//! level of [`PEAK_LEVELS`].
//!
//! Progress comes from the samples read against the expected count (stdout carries the PCM, so
//! `-progress pipe:1` is not available).

use std::path::Path;

use se_app::ports::{MediaError, MediaJobSpec, ProgressSink};

use super::pcm::{PcmFormat, PcmReader, spawn_decoder};
use crate::{
    cache_layout::{peaks_file, write_atomically},
    peaks::{PEAK_LEVELS, PeakAccumulator, all_levels},
};

/// Generates the peaks of `spec` with the ffmpeg at `ffmpeg`.
///
/// # Errors
/// [`MediaError`] when the media has no sound, ffmpeg fails or the cache cannot be written.
pub async fn generate(
    ffmpeg: &Path,
    spec: &MediaJobSpec,
    progress: &dyn ProgressSink,
) -> Result<(), MediaError> {
    let format = PcmFormat::of(&spec.media_info, "draw a waveform from")?;
    let mut process = spawn_decoder(ffmpeg, &spec.input, format)?;
    let mut accumulator = PeakAccumulator::new(format.channels, format.sample_rate, PEAK_LEVELS[0]);
    let mut reader = PcmReader::new(&mut process);
    while let Some(samples) = reader.next_samples().await? {
        accumulator.push(&samples);
        if let Some(done) = format.progress(accumulator.frames()) {
            progress.report(done);
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
