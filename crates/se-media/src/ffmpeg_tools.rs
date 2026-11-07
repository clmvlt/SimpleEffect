//! [`FfmpegTools`]: the ffmpeg installation as a whole — where the tools are (shared
//! [`ToolLocator`]), their version, and the hardware encoders that work (cached per ffmpeg). It
//! implements the `MediaTools` port; the job runner uses it to pick encoders.

use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use se_app::ports::MediaTools;
use se_protocol::MediaToolsStatus;

use crate::{
    encoders::{EncoderCache, EncoderSupport},
    process::Sidecar,
    tools::{Tool, ToolLocator},
};

/// The tools and what they can do. Cheap to clone (shared state).
#[derive(Debug, Clone)]
pub struct FfmpegTools {
    locator: ToolLocator,
    encoders: Arc<EncoderCache>,
}

impl FfmpegTools {
    /// The tools found by `locator` (clones of it see folder changes made here).
    #[must_use]
    pub fn new(locator: ToolLocator) -> Self {
        Self {
            locator,
            encoders: Arc::new(EncoderCache::default()),
        }
    }

    /// Where the tools are looked for.
    #[must_use]
    pub fn locator(&self) -> &ToolLocator {
        &self.locator
    }

    /// The hardware encoders that work with the ffmpeg at `ffmpeg` (detected once).
    pub async fn encoder_support(&self, ffmpeg: &Path) -> EncoderSupport {
        self.encoders.support(ffmpeg).await
    }

    /// Detects the hardware encoders now (started in the background when the core starts, so the
    /// first mezzanine does not wait for it). Does nothing without ffmpeg.
    pub async fn warm_up(&self) {
        if let Ok(ffmpeg) = self.locator.locate(Tool::Ffmpeg) {
            self.encoder_support(&ffmpeg).await;
        }
    }
}

#[async_trait]
impl MediaTools for FfmpegTools {
    fn set_tools_dir(&self, dir: Option<std::path::PathBuf>) {
        self.locator.set_override_dir(dir);
    }

    async fn status(&self) -> MediaToolsStatus {
        let ffmpeg = self.locator.locate(Tool::Ffmpeg);
        let ffprobe = self.locator.locate(Tool::Ffprobe);
        let problem = [&ffmpeg, &ffprobe]
            .into_iter()
            .find_map(|found| found.as_ref().err().map(ToString::to_string));
        let (ffmpeg_version, hardware_encoders) = match &ffmpeg {
            Ok(path) => (
                version(path).await,
                self.encoder_support(path).await.names(),
            ),
            Err(_) => (None, Vec::new()),
        };
        let display = |found: &Result<std::path::PathBuf, _>| {
            found
                .as_ref()
                .ok()
                .map(|path| path.to_string_lossy().into_owned())
        };
        MediaToolsStatus {
            ffmpeg_path: display(&ffmpeg),
            ffprobe_path: display(&ffprobe),
            ffmpeg_version,
            hardware_encoders,
            problem,
        }
    }
}

/// The version ffmpeg prints (`ffmpeg version 9.0.2-essentials_build… Copyright…` → `9.0.2…`).
async fn version(ffmpeg: &Path) -> Option<String> {
    let output = Sidecar::spawn(ffmpeg, ["-hide_banner", "-version"], None)
        .ok()?
        .output()
        .await
        .ok()?;
    parse_version(&String::from_utf8_lossy(&output))
}

/// The version word of ffmpeg's first `-version` line.
fn parse_version(text: &str) -> Option<String> {
    let (_, rest) = text.lines().next()?.split_once("version ")?;
    rest.split_whitespace().next().map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_version_word() {
        let text = "ffmpeg version 9.0.2-essentials_build-www.gyan.dev Copyright (c) 2000-2026\nbuilt with gcc";
        assert_eq!(
            parse_version(text).as_deref(),
            Some("9.0.2-essentials_build-www.gyan.dev")
        );
        assert_eq!(parse_version("nothing"), None);
    }
}
