//! [`FfmpegJobRunner`]: the `MediaJobRunner` and `FrameExtractor` ports over ffmpeg
//! (thumbnails, waveform peaks, mezzanines and proxies, audio analyses, scene detection, frames).

mod audio_analysis;
mod ffmpeg_progress;
mod pcm;
mod scene_detection;
pub mod thumbnail_plan;
mod thumbnails;
mod transcode;
pub mod transcode_args;
mod waveform;

use std::path::Path;

use async_trait::async_trait;
use se_app::ports::{
    FrameExtractor, FrameRequest, MediaError, MediaJobOutput, MediaJobRunner, MediaJobSpec,
    MediaTask, ProgressSink,
};
use se_domain::AnalysisResult;
use se_protocol::JobKind;

pub use self::{scene_detection::scene_filter, thumbnails::parse_tile_times};
use crate::{cache_layout, ffmpeg_tools::FfmpegTools, frames, tools::Tool};

/// Runs media jobs with the ffmpeg of its [`FfmpegTools`].
#[derive(Debug, Clone)]
pub struct FfmpegJobRunner {
    tools: FfmpegTools,
}

impl FfmpegJobRunner {
    /// A runner using `tools` (shared with the settings: a new tools folder applies at once).
    #[must_use]
    pub fn new(tools: FfmpegTools) -> Self {
        Self { tools }
    }
}

#[async_trait]
impl MediaJobRunner for FfmpegJobRunner {
    async fn outputs_ready(&self, kind: JobKind, output_dir: &Path) -> bool {
        cache_layout::outputs_ready(kind, output_dir).await
    }

    async fn run(
        &self,
        spec: &MediaJobSpec,
        progress: &dyn ProgressSink,
    ) -> Result<MediaJobOutput, MediaError> {
        let ffmpeg = self.tools.locator().locate(Tool::Ffmpeg)?;
        tokio::fs::create_dir_all(&spec.output_dir)
            .await
            .map_err(|error| MediaError::Io {
                path: spec.output_dir.clone(),
                message: error.to_string(),
            })?;
        Ok(match &spec.task {
            MediaTask::Thumbnails => {
                thumbnails::generate(&ffmpeg, spec, progress).await?;
                MediaJobOutput::Files
            }
            MediaTask::WaveformPeaks => {
                waveform::generate(&ffmpeg, spec, progress).await?;
                MediaJobOutput::Files
            }
            MediaTask::Transcode(plan) => {
                let support = self.tools.encoder_support(&ffmpeg).await;
                transcode::generate(&ffmpeg, support, spec, plan, progress).await?;
                MediaJobOutput::Files
            }
            MediaTask::AudioAnalysis { silence } => {
                let analysis = audio_analysis::analyse(&ffmpeg, spec, *silence, progress).await?;
                MediaJobOutput::Analysis(AnalysisResult::AudioAnalysis(analysis))
            }
            MediaTask::SceneDetection { threshold } => {
                let detection =
                    scene_detection::detect(&ffmpeg, spec, *threshold, progress).await?;
                MediaJobOutput::Analysis(AnalysisResult::SceneDetection(detection))
            }
        })
    }
}

#[async_trait]
impl FrameExtractor for FfmpegJobRunner {
    async fn extract_frame(&self, request: &FrameRequest) -> Result<(), MediaError> {
        let ffmpeg = self.tools.locator().locate(Tool::Ffmpeg)?;
        frames::extract(&ffmpeg, request).await
    }
}
