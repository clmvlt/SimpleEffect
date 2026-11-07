//! [`FfmpegJobRunner`]: the `MediaJobRunner` port over ffmpeg (thumbnails, waveform peaks).

mod ffmpeg_progress;
pub mod thumbnail_plan;
mod thumbnails;
mod waveform;

use std::path::Path;

use async_trait::async_trait;
use se_app::ports::{MediaError, MediaJobRunner, MediaJobSpec, ProgressSink};
use se_protocol::JobKind;

pub use self::thumbnails::parse_tile_times;
use crate::{
    cache_layout,
    tools::{Tool, ToolLocator},
};

/// Runs media jobs with the ffmpeg found by its [`ToolLocator`].
#[derive(Debug, Clone)]
pub struct FfmpegJobRunner {
    tools: ToolLocator,
}

impl FfmpegJobRunner {
    /// A runner finding ffmpeg with `tools`.
    #[must_use]
    pub fn new(tools: ToolLocator) -> Self {
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
    ) -> Result<(), MediaError> {
        let ffmpeg = self.tools.locate(Tool::Ffmpeg)?;
        tokio::fs::create_dir_all(&spec.output_dir)
            .await
            .map_err(|error| MediaError::Io {
                path: spec.output_dir.clone(),
                message: error.to_string(),
            })?;
        match spec.kind {
            JobKind::Thumbnails => thumbnails::generate(&ffmpeg, spec, progress).await,
            JobKind::WaveformPeaks => waveform::generate(&ffmpeg, spec, progress).await,
        }
    }
}
