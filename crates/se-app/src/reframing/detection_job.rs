//! The subject detection job (`JobKind::SubjectDetection`): pictures sampled by ffmpeg stream
//! through a small channel to the face detector, batch by batch, while ffmpeg keeps decoding;
//! the faces are stored once per content in the project database (`AnalysisKind::
//! SubjectDetection`). The asset keeps no summary: only automatic reframing reads them.

use std::sync::Arc;

use async_trait::async_trait;
use se_domain::{AnalysisResult, AssetId, Flicks, SubjectDetection, SubjectFrame};
use tokio::sync::mpsc;

use crate::{
    jobs::{JobAttempt, JobWork},
    ports::{
        Clock, FrameSampleRequest, FrameSampler, ProgressSink, SampledFrame, StoredAnalysis,
        SubjectDetector,
    },
    project_actor::ProjectActorHandle,
};

/// Pictures waiting between ffmpeg and the detector (a few megabytes at most).
const CHANNEL_FRAMES: usize = 16;

/// Pictures handed to the detector at once.
const BATCH_FRAMES: usize = 8;

/// What a subject detection job needs, decided when it was queued.
pub(super) struct SubjectDetectionJob {
    pub sampler: Arc<dyn FrameSampler>,
    pub detector: Arc<dyn SubjectDetector>,
    pub clock: Arc<dyn Clock>,
    pub handle: ProjectActorHandle,
    pub asset_id: AssetId,
    pub content_key: String,
    /// The sampling (its source is the original or a ready proxy).
    pub request: FrameSampleRequest,
    /// The media's duration, for progress.
    pub duration: Option<Flicks>,
}

#[async_trait]
impl JobWork for SubjectDetectionJob {
    async fn run(&self, progress: &dyn ProgressSink, attempt: JobAttempt) -> Result<(), String> {
        // A second attempt decodes in software (the hardware may be what failed).
        let request = FrameSampleRequest {
            hardware_decoding: self.request.hardware_decoding && attempt.number <= 1,
            ..self.request.clone()
        };
        let (sender, receiver) = mpsc::channel(CHANNEL_FRAMES);
        let sampling = self.sampler.sample_frames(&request, sender);
        let detecting = self.detect_all(receiver, progress);
        let (sampled, detected) = tokio::join!(sampling, detecting);
        let frames = detected?;
        sampled.map_err(|error| error.to_string())?;
        if frames.is_empty() {
            return Err("ffmpeg gave no picture of this video: check that it plays".to_owned());
        }
        let stored = StoredAnalysis {
            asset_id: self.asset_id,
            content_key: self.content_key.clone(),
            created_at: self.clock.now_unix_millis(),
            result: AnalysisResult::SubjectDetection(SubjectDetection {
                model: self.detector.model().to_owned(),
                interval: request.interval,
                frames,
            }),
        };
        self.handle
            .analyses()
            .save_analysis(&stored)
            .await
            .map_err(|error| error.to_string())?;
        progress.report(1.0);
        Ok(())
    }
}

impl SubjectDetectionJob {
    /// Reads every picture from `receiver` and detects its faces, in batches. On a detector
    /// error the receiver is dropped, which stops ffmpeg.
    async fn detect_all(
        &self,
        mut receiver: mpsc::Receiver<SampledFrame>,
        progress: &dyn ProgressSink,
    ) -> Result<Vec<SubjectFrame>, String> {
        let expected = self.duration.map(|duration| {
            duration.to_seconds_f64() / self.request.interval.to_seconds_f64().max(1e-3)
        });
        let mut frames: Vec<SubjectFrame> = Vec::new();
        let mut batch: Vec<SampledFrame> = Vec::with_capacity(BATCH_FRAMES);
        loop {
            let next = receiver.recv().await;
            let finished = next.is_none();
            batch.extend(next);
            if batch.len() < BATCH_FRAMES && !finished {
                continue;
            }
            if !batch.is_empty() {
                let pictures = std::mem::take(&mut batch);
                let times: Vec<Flicks> = pictures.iter().map(|frame| frame.time).collect();
                let faces = self
                    .detector
                    .detect(pictures)
                    .await
                    .map_err(|error| error.to_string())?;
                frames.extend(
                    times
                        .into_iter()
                        .zip(faces)
                        .map(|(time, faces)| SubjectFrame { time, faces }),
                );
                if let Some(expected) = expected.filter(|expected| *expected > 0.0) {
                    let done = f64::from(u32::try_from(frames.len()).unwrap_or(u32::MAX));
                    progress.report((done / expected).min(0.99));
                }
            }
            if finished {
                return Ok(frames);
            }
        }
    }
}
