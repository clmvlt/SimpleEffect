//! The audio analysis job: ffmpeg decodes the first sound stream to PCM (`pcm.rs`); the samples
//! are measured on the blocking pool ([`AudioAnalyzer`]: loudness and silences), so the tokio
//! workers only move bytes.

use std::path::Path;

use se_app::ports::{MediaError, MediaJobSpec, ProgressSink};
use se_domain::{AudioAnalysis, SilenceSettings};
use tokio::sync::mpsc;

use super::pcm::{PcmFormat, PcmReader, spawn_decoder};
use crate::analysis::AudioAnalyzer;

/// Chunks waiting for the analyzer before reading slows down.
const QUEUED_CHUNKS: usize = 16;

/// Analyses the sound of `spec` with the ffmpeg at `ffmpeg`. Dropping the future stops ffmpeg
/// and lets the analyzer thread end (its channel closes).
///
/// # Errors
/// [`MediaError`] when the media has no sound or ffmpeg fails.
pub async fn analyse(
    ffmpeg: &Path,
    spec: &MediaJobSpec,
    silence: SilenceSettings,
    progress: &dyn ProgressSink,
) -> Result<AudioAnalysis, MediaError> {
    let format = PcmFormat::of(&spec.media_info, "analyse")?;
    let mut analyzer = AudioAnalyzer::new(format.channels, format.sample_rate, silence)
        .map_err(MediaError::Failed)?;
    let (sender, mut receiver) = mpsc::channel::<Vec<f32>>(QUEUED_CHUNKS);
    let worker = tokio::task::spawn_blocking(move || {
        while let Some(samples) = receiver.blocking_recv() {
            analyzer.push(&samples)?;
        }
        Ok::<_, String>(analyzer.finish())
    });
    let mut process = spawn_decoder(ffmpeg, &spec.input, format)?;
    let mut reader = PcmReader::new(&mut process);
    let mut frames = 0_u64;
    while let Some(samples) = reader.next_samples().await? {
        frames += (samples.len() / usize::from(format.channels)) as u64;
        if sender.send(samples).await.is_err() {
            // The analyzer stopped on an error: its result says why.
            break;
        }
        if let Some(done) = format.progress(frames) {
            progress.report(done);
        }
    }
    drop(sender);
    process.finish().await?;
    let analysis = worker
        .await
        .map_err(|error| MediaError::Failed(format!("the audio analysis stopped: {error}")))?
        .map_err(MediaError::Failed)?;
    progress.report(1.0);
    Ok(analysis)
}
