//! [`FfprobeProber`]: the `MediaProber` port over `ffprobe`.

mod ffprobe_output;
mod media_info;

use std::path::Path;

use async_trait::async_trait;
use se_app::ports::{MediaError, MediaProber, ProbedMedia};

pub use self::media_info::parse_probe_output;
use crate::{
    process::Sidecar,
    tools::{Tool, ToolLocator},
};

/// Probes media files with `ffprobe -print_format json -show_format -show_streams`.
#[derive(Debug, Clone)]
pub struct FfprobeProber {
    tools: ToolLocator,
}

impl FfprobeProber {
    /// A prober finding ffprobe with `tools`.
    #[must_use]
    pub fn new(tools: ToolLocator) -> Self {
        Self { tools }
    }
}

#[async_trait]
impl MediaProber for FfprobeProber {
    async fn probe(&self, path: &Path) -> Result<ProbedMedia, MediaError> {
        if !tokio::fs::metadata(path)
            .await
            .is_ok_and(|metadata| metadata.is_file())
        {
            return Err(MediaError::NotFound {
                path: path.to_path_buf(),
            });
        }
        let ffprobe = self.tools.locate(Tool::Ffprobe)?;
        let args = [
            "-v".as_ref(),
            "error".as_ref(),
            "-print_format".as_ref(),
            "json".as_ref(),
            "-show_format".as_ref(),
            "-show_streams".as_ref(),
            path.as_os_str(),
        ];
        let unsupported = |reason: String| MediaError::Unsupported {
            path: path.to_path_buf(),
            reason,
        };
        let output = Sidecar::spawn(&ffprobe, args, None)?
            .output()
            .await
            .map_err(|error| unsupported(error.to_string()))?;
        let json = String::from_utf8_lossy(&output);
        let (kind, info) = parse_probe_output(&json).map_err(unsupported)?;
        Ok(ProbedMedia { kind, info })
    }
}
