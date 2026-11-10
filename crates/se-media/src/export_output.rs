//! [`FsExportOutputs`]: the files of exports (prompt 27, ADR-0038), implementing se-app's
//! `ExportOutputs`.
//!
//! Everything is written next to the final file under partial names (`<name>.part`,
//! `<name>.video.part`, `<name>.audio.part`) and the final file appears by a rename once
//! complete. Muxed exports are the renderer's bytes; masters are RGBA frames encoded by ffmpeg
//! as they arrive (the WAV of their sound is complete before the first frame); AAC exports join
//! the renderer's picture to its WAV encoded by ffmpeg at the end. Dropping an unfinished output
//! stops ffmpeg and removes every partial file.

mod args;
mod part_file;

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use async_trait::async_trait;
use se_app::ports::{ExportOutput, ExportOutputPlan, ExportOutputs, ExportStream, MediaError};
use se_protocol::ExportPipeline;
use tokio::{io::AsyncWriteExt, process::ChildStdin};

pub use self::args::{aac_remux_args, frames_args, master_codec_args, muxer_of};
use self::part_file::{PartFile, io_error};
use crate::{
    process::Sidecar,
    tools::{Tool, ToolLocator},
};

/// Attempts at removing a partial file still held by a stopping process (Windows).
const REMOVE_ATTEMPTS: u32 = 10;
/// Pause between those attempts.
const REMOVE_PAUSE: Duration = Duration::from_millis(100);

/// Opens export outputs on disk, finding ffmpeg with `locator` when a pipeline needs it.
#[derive(Clone)]
pub struct FsExportOutputs {
    locator: ToolLocator,
}

impl FsExportOutputs {
    /// Outputs using the ffmpeg `locator` finds.
    #[must_use]
    pub const fn new(locator: ToolLocator) -> Self {
        Self { locator }
    }
}

/// `<file name><suffix>` next to `path`.
fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    path.with_file_name(name)
}

/// The partial files of an export at `path`.
struct PartPaths {
    /// What becomes the final file.
    output: PathBuf,
    /// The muxed picture of AAC exports.
    video: PathBuf,
    /// The WAV for ffmpeg.
    audio: PathBuf,
}

impl PartPaths {
    fn of(path: &Path) -> Self {
        Self {
            output: sibling(path, ".part"),
            video: sibling(path, ".video.part"),
            audio: sibling(path, ".audio.part"),
        }
    }

    fn all(&self) -> [PathBuf; 3] {
        [self.output.clone(), self.video.clone(), self.audio.clone()]
    }
}

#[async_trait]
impl ExportOutputs for FsExportOutputs {
    async fn open(&self, plan: ExportOutputPlan) -> Result<Box<dyn ExportOutput>, MediaError> {
        let folder = plan.path.parent().unwrap_or_else(|| Path::new("."));
        if !tokio::fs::metadata(folder)
            .await
            .is_ok_and(|metadata| metadata.is_dir())
        {
            return Err(MediaError::Failed(format!(
                "the folder {} does not exist: choose another folder",
                folder.display()
            )));
        }
        let parts = PartPaths::of(&plan.path);
        let (data, audio, ffmpeg) = match plan.pipeline {
            ExportPipeline::Muxed => (Some(parts.output.clone()), None, None),
            ExportPipeline::FfmpegFrames { with_audio } => (
                None,
                with_audio.then(|| parts.audio.clone()),
                Some(self.locator.locate(Tool::Ffmpeg)?),
            ),
            ExportPipeline::AacRemux { with_video } => (
                with_video.then(|| parts.video.clone()),
                Some(parts.audio.clone()),
                Some(self.locator.locate(Tool::Ffmpeg)?),
            ),
        };
        let mut output = FsExportOutput {
            frame_bytes: plan
                .video
                .map_or(0, |video| video.width as usize * video.height as usize * 4),
            plan,
            parts,
            ffmpeg,
            data: None,
            audio: None,
            encoder: None,
            frames: 0,
            finished: false,
        };
        if let Some(path) = data {
            output.data = Some(PartFile::create(path).await?);
        }
        if let Some(path) = audio {
            output.audio = Some(PartFile::create(path).await?);
        }
        Ok(Box::new(output))
    }
}

/// ffmpeg encoding frames from its stdin.
struct FrameEncoder {
    process: Sidecar,
    stdin: ChildStdin,
}

/// The files of one export being written.
struct FsExportOutput {
    plan: ExportOutputPlan,
    parts: PartPaths,
    ffmpeg: Option<PathBuf>,
    data: Option<PartFile>,
    audio: Option<PartFile>,
    encoder: Option<FrameEncoder>,
    frame_bytes: usize,
    frames: u64,
    finished: bool,
}

#[async_trait]
impl ExportOutput for FsExportOutput {
    async fn write(
        &mut self,
        stream: ExportStream,
        position: u64,
        bytes: &[u8],
    ) -> Result<(), MediaError> {
        let file = match stream {
            ExportStream::Data => self.data.as_mut(),
            ExportStream::Audio => self.audio.as_mut(),
        };
        let file = file.ok_or_else(|| {
            MediaError::Failed(format!(
                "this export takes no {stream:?} stream (or it was already complete)"
            ))
        })?;
        file.write_at(position, bytes).await
    }

    async fn write_frame(&mut self, rgba: &[u8]) -> Result<(), MediaError> {
        if self.frame_bytes == 0 || rgba.len() != self.frame_bytes {
            return Err(MediaError::Failed(format!(
                "a frame of {} bytes does not fit this export ({} bytes expected)",
                rgba.len(),
                self.frame_bytes
            )));
        }
        if self.encoder.is_none() {
            self.encoder = Some(self.start_encoder().await?);
        }
        let Some(encoder) = self.encoder.as_mut() else {
            return Err(MediaError::Failed("ffmpeg is not running".to_owned()));
        };
        if let Err(error) = encoder.stdin.write_all(rgba).await {
            return Err(self.encoder_failure(&error).await);
        }
        self.frames += 1;
        Ok(())
    }

    async fn finish(mut self: Box<Self>) -> Result<u64, MediaError> {
        match self.plan.pipeline {
            ExportPipeline::Muxed => {
                if let Some(data) = self.data.take() {
                    data.close().await?;
                }
            }
            ExportPipeline::FfmpegFrames { .. } => self.finish_frames().await?,
            ExportPipeline::AacRemux { .. } => self.remux_aac().await?,
        }
        tokio::fs::rename(&self.parts.output, &self.plan.path)
            .await
            .map_err(|error| io_error(&self.plan.path, &error))?;
        self.finished = true;
        for leftover in [&self.parts.video, &self.parts.audio] {
            remove_quietly(leftover).await;
        }
        let metadata = tokio::fs::metadata(&self.plan.path)
            .await
            .map_err(|error| io_error(&self.plan.path, &error))?;
        Ok(metadata.len())
    }
}

impl FsExportOutput {
    fn ffmpeg(&self) -> Result<&Path, MediaError> {
        self.ffmpeg
            .as_deref()
            .ok_or_else(|| MediaError::Failed("this export does not use ffmpeg".to_owned()))
    }

    /// Closes the WAV (complete by now) and starts ffmpeg on the first frame.
    async fn start_encoder(&mut self) -> Result<FrameEncoder, MediaError> {
        let video = self
            .plan
            .video
            .ok_or_else(|| MediaError::Failed("this export has no picture".to_owned()))?;
        let audio = match self.audio.take() {
            Some(file) => Some(file.close().await?),
            None => None,
        };
        let args = frames_args(
            &video,
            self.plan.frame_rate,
            audio.as_deref(),
            &self.parts.output,
        )
        .map_err(MediaError::Failed)?;
        let mut process = Sidecar::spawn_with_stdin(self.ffmpeg()?, args)?;
        let stdin = process
            .take_stdin()
            .ok_or_else(|| MediaError::Failed("ffmpeg has no input pipe".to_owned()))?;
        Ok(FrameEncoder { process, stdin })
    }

    /// Why ffmpeg stopped taking frames (its own message when it exited with one).
    async fn encoder_failure(&mut self, error: &std::io::Error) -> MediaError {
        let Some(FrameEncoder { process, stdin }) = self.encoder.take() else {
            return MediaError::Failed(format!("ffmpeg stopped taking frames: {error}"));
        };
        drop(stdin);
        match process.finish().await {
            Err(failure) => failure,
            Ok(()) => MediaError::Failed(format!("ffmpeg stopped taking frames: {error}")),
        }
    }

    async fn finish_frames(&mut self) -> Result<(), MediaError> {
        let FrameEncoder { process, stdin } = self
            .encoder
            .take()
            .ok_or_else(|| MediaError::Failed("the editor sent no frame to encode".to_owned()))?;
        // End of input: ffmpeg writes its last frames and the file's index, then exits.
        drop(stdin);
        process.finish().await
    }

    async fn remux_aac(&mut self) -> Result<(), MediaError> {
        let bits_per_second = self
            .plan
            .audio
            .map_or(192_000, |audio| audio.bits_per_second);
        let video = match self.data.take() {
            Some(file) => Some(file.close().await?),
            None => None,
        };
        let audio = match self.audio.take() {
            Some(file) => file.close().await?,
            None => return Err(MediaError::Failed("the editor sent no sound".to_owned())),
        };
        let args = aac_remux_args(
            video.as_deref(),
            &audio,
            bits_per_second,
            self.plan.container,
            &self.parts.output,
        );
        Sidecar::spawn(self.ffmpeg()?, args, None)?.finish().await
    }
}

impl Drop for FsExportOutput {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        let paths = self.parts.all();
        let encoder = self.encoder.take();
        // Files close when dropped; ffmpeg must exit before its output can go (Windows).
        drop((self.data.take(), self.audio.take()));
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            drop(encoder);
            for path in &paths {
                // Best effort without a runtime: nothing else can be done here.
                let _ = std::fs::remove_file(path);
            }
            return;
        };
        runtime.spawn(async move {
            if let Some(FrameEncoder { process, stdin }) = encoder {
                drop(stdin);
                process.stop().await;
            }
            for path in &paths {
                remove_patiently(path).await;
            }
        });
    }
}

/// Removes `path` if it exists, ignoring failures (a leftover only wastes space).
async fn remove_quietly(path: &Path) {
    if let Err(error) = tokio::fs::remove_file(path).await
        && error.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(%error, path = %path.display(), "removing an export's partial file failed");
    }
}

/// Removes `path`, retrying while a stopping process still holds it.
async fn remove_patiently(path: &Path) {
    for attempt in 1..=REMOVE_ATTEMPTS {
        match tokio::fs::remove_file(path).await {
            Ok(()) => return,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
            Err(error) if attempt == REMOVE_ATTEMPTS => {
                tracing::warn!(%error, path = %path.display(), "a partial export file was left");
            }
            Err(_) => tokio::time::sleep(REMOVE_PAUSE).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_files_sit_next_to_the_final_file() {
        let parts = PartPaths::of(Path::new("exports/Film.mp4"));
        assert_eq!(parts.output, Path::new("exports/Film.mp4.part"));
        assert_eq!(parts.video, Path::new("exports/Film.mp4.video.part"));
        assert_eq!(parts.audio, Path::new("exports/Film.mp4.audio.part"));
    }
}
