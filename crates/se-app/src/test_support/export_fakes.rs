//! [`FakeExportOutputs`]: export outputs in memory (prompt 27). Written bytes and frames are
//! kept per output; finished files are recorded by path, unfinished outputs that were dropped
//! (cancellations, failures) are counted, as the real adapter would remove their partial files.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;

use super::lock;
use crate::ports::{ExportOutput, ExportOutputPlan, ExportOutputs, ExportStream, MediaError};

/// A file an export completed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishedExportFile {
    /// Where it was written.
    pub path: PathBuf,
    /// The muxed bytes (`data` stream).
    pub data: Vec<u8>,
    /// The WAV bytes (`audio` stream).
    pub audio: Vec<u8>,
    /// Frames given to « ffmpeg ».
    pub frames: usize,
}

#[derive(Default)]
struct OutputsState {
    opened: Vec<ExportOutputPlan>,
    finished: Vec<FinishedExportFile>,
    discarded: Vec<PathBuf>,
    fail_open: Option<String>,
    fail_finish: Option<String>,
}

/// Export outputs in memory. Clones share state.
#[derive(Clone, Default)]
pub struct FakeExportOutputs {
    state: Arc<Mutex<OutputsState>>,
}

impl FakeExportOutputs {
    /// The plans of every output opened, in order.
    #[must_use]
    pub fn opened(&self) -> Vec<ExportOutputPlan> {
        lock(&self.state).opened.clone()
    }

    /// The files completed, in order.
    #[must_use]
    pub fn finished(&self) -> Vec<FinishedExportFile> {
        lock(&self.state).finished.clone()
    }

    /// The final paths of outputs dropped before they finished (partial files removed).
    #[must_use]
    pub fn discarded(&self) -> Vec<PathBuf> {
        lock(&self.state).discarded.clone()
    }

    /// Makes the next `open` fail with `message`.
    pub fn fail_next_open(&self, message: &str) {
        lock(&self.state).fail_open = Some(message.to_owned());
    }

    /// Makes the next `finish` fail with `message`.
    pub fn fail_next_finish(&self, message: &str) {
        lock(&self.state).fail_finish = Some(message.to_owned());
    }
}

#[async_trait]
impl ExportOutputs for FakeExportOutputs {
    async fn open(&self, plan: ExportOutputPlan) -> Result<Box<dyn ExportOutput>, MediaError> {
        let mut state = lock(&self.state);
        if let Some(message) = state.fail_open.take() {
            return Err(MediaError::Failed(message));
        }
        state.opened.push(plan.clone());
        Ok(Box::new(FakeExportOutput {
            state: Arc::clone(&self.state),
            file: FinishedExportFile {
                path: plan.path.clone(),
                data: Vec::new(),
                audio: Vec::new(),
                frames: 0,
            },
            frame_bytes: plan
                .video
                .map(|video| video.width as usize * video.height as usize * 4),
            finished: false,
        }))
    }
}

struct FakeExportOutput {
    state: Arc<Mutex<OutputsState>>,
    file: FinishedExportFile,
    frame_bytes: Option<usize>,
    finished: bool,
}

#[async_trait]
impl ExportOutput for FakeExportOutput {
    async fn write(
        &mut self,
        stream: ExportStream,
        position: u64,
        bytes: &[u8],
    ) -> Result<(), MediaError> {
        let target = match stream {
            ExportStream::Data => &mut self.file.data,
            ExportStream::Audio => &mut self.file.audio,
        };
        let start = usize::try_from(position)
            .map_err(|_| MediaError::Failed("position out of range".to_owned()))?;
        let end = start + bytes.len();
        if target.len() < end {
            target.resize(end, 0);
        }
        target[start..end].copy_from_slice(bytes);
        Ok(())
    }

    async fn write_frame(&mut self, rgba: &[u8]) -> Result<(), MediaError> {
        if self.frame_bytes != Some(rgba.len()) {
            return Err(MediaError::Failed(format!(
                "a frame of {} bytes does not match the export's size",
                rgba.len()
            )));
        }
        self.file.frames += 1;
        Ok(())
    }

    async fn finish(mut self: Box<Self>) -> Result<u64, MediaError> {
        let mut state = lock(&self.state);
        if let Some(message) = state.fail_finish.take() {
            return Err(MediaError::Failed(message));
        }
        self.finished = true;
        let size = self.file.data.len().max(self.file.audio.len()) + self.file.frames;
        state.finished.push(self.file.clone());
        Ok(size as u64)
    }
}

impl Drop for FakeExportOutput {
    fn drop(&mut self) {
        if !self.finished {
            lock(&self.state).discarded.push(self.file.path.clone());
        }
    }
}
