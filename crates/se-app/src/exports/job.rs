//! [`ExportJob`]: the job of one export. It opens the output, asks the renderer to draw the
//! snapshot, writes what the renderer uploads, then completes and checks the file.
//!
//! The job owns the output: uploads reach it through an inbox (one writer, writes in order). If
//! the job is cancelled, its future is dropped with the output, whose drop removes the partial
//! files and stops ffmpeg; a guard marks the export cancelled and tells the renderer.

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use se_domain::ExportId;
use se_protocol::{CoreEvent, ExportRenderRequest, ExportState};
use tokio::{
    sync::{mpsc, oneshot},
    time::{Instant, MissedTickBehavior},
};

use super::{check, registry::ExportRegistry};
use crate::{
    jobs::{JobAttempt, JobWork},
    ports::{
        EventSink, ExportOutput, ExportOutputPlan, ExportOutputs, ExportStream, MediaError,
        MediaProber, ProgressSink, RendererLink,
    },
};

/// Uploads waiting for the job (each holds up to a chunk or a frame in memory).
const INBOX_CAPACITY: usize = 4;

/// What the renderer sends to a running export.
pub enum ExportMessage {
    /// Bytes of a stream at a position.
    Write {
        /// Which stream.
        stream: ExportStream,
        /// Where in it.
        position: u64,
        /// The bytes.
        bytes: Vec<u8>,
        /// Told once written.
        reply: oneshot::Sender<Result<(), MediaError>>,
    },
    /// The next RGBA frame for ffmpeg.
    Frame {
        /// `width × height × 4` bytes.
        rgba: Vec<u8>,
        /// Told once ffmpeg took it.
        reply: oneshot::Sender<Result<(), MediaError>>,
    },
    /// How far the render is.
    Progress {
        /// Units done (frames, or sound blocks without picture).
        done: u64,
        /// Units in all.
        total: u64,
    },
    /// Everything was uploaded.
    Finished,
    /// The renderer could not render.
    Failed {
        /// Why.
        message: String,
    },
}

/// How long the job waits for the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportTiming {
    /// Longest silence of the renderer (no upload, no progress) before the export fails.
    pub idle_limit: Duration,
    /// How often the job checks that an editor window is still connected.
    pub connection_check: Duration,
}

impl Default for ExportTiming {
    fn default() -> Self {
        Self {
            idle_limit: Duration::from_secs(90),
            connection_check: Duration::from_secs(2),
        }
    }
}

/// The work of one export job (never retried: a second render would start from scratch).
pub struct ExportJob {
    pub(super) export_id: ExportId,
    pub(super) request: ExportRenderRequest,
    pub(super) plan: ExportOutputPlan,
    pub(super) registry: Arc<ExportRegistry>,
    pub(super) outputs: Arc<dyn ExportOutputs>,
    pub(super) prober: Arc<dyn MediaProber>,
    pub(super) renderer: Arc<dyn RendererLink>,
    pub(super) events: Arc<dyn EventSink>,
    pub(super) timing: ExportTiming,
}

#[async_trait]
impl JobWork for ExportJob {
    fn retries(&self) -> bool {
        false
    }

    async fn run(&self, progress: &dyn ProgressSink, _attempt: JobAttempt) -> Result<(), String> {
        let mut guard = CancelOnDrop {
            registry: Arc::clone(&self.registry),
            export_id: self.export_id,
            armed: true,
        };
        let outcome = self.render(progress).await;
        guard.armed = false;
        self.registry.close_inbox(self.export_id);
        match outcome {
            Ok(completed) => {
                self.registry.set_state(self.export_id, completed);
                Ok(())
            }
            Err(message) => {
                self.registry.set_state(
                    self.export_id,
                    ExportState::Failed {
                        message: message.clone(),
                    },
                );
                self.events.publish(CoreEvent::ExportRenderCancelled {
                    export_id: self.export_id,
                });
                Err(message)
            }
        }
    }
}

impl ExportJob {
    /// Renders and completes the file; the completed state, or why it failed.
    async fn render(&self, progress: &dyn ProgressSink) -> Result<ExportState, String> {
        if self.registry.is_over(self.export_id) {
            return Err("the export was cancelled before it started".to_owned());
        }
        if !self.renderer.is_connected() {
            return Err(
                "no SimpleEffect window is open to draw the export: open the editor and export again"
                    .to_owned(),
            );
        }
        let started = Instant::now();
        self.registry.set_state(
            self.export_id,
            ExportState::Rendering {
                progress: 0.0,
                eta_seconds: None,
            },
        );
        let mut output = self
            .outputs
            .open(self.plan.clone())
            .await
            .map_err(|error| format!("the export file could not be prepared: {error}"))?;
        let (sender, inbox) = mpsc::channel(INBOX_CAPACITY);
        self.registry.open_inbox(self.export_id, sender);
        self.events
            .publish(CoreEvent::ExportRenderRequested(self.request.clone()));
        self.receive(output.as_mut(), inbox, progress, started)
            .await?;
        self.registry.close_inbox(self.export_id);
        self.registry
            .set_state(self.export_id, ExportState::Finalizing);
        let size_bytes = output
            .finish()
            .await
            .map_err(|error| format!("the export file could not be completed: {error}"))?;
        Ok(ExportState::Completed {
            size_bytes,
            elapsed_seconds: started.elapsed().as_secs_f64(),
            check: self.file_check().await,
        })
    }

    /// Writes uploads until the renderer finishes, fails, falls silent or disconnects.
    async fn receive(
        &self,
        output: &mut dyn ExportOutput,
        mut inbox: mpsc::Receiver<ExportMessage>,
        progress: &dyn ProgressSink,
        started: Instant,
    ) -> Result<(), String> {
        let mut tracker = ProgressTracker::default();
        let mut connection = tokio::time::interval(self.timing.connection_check);
        connection.set_missed_tick_behavior(MissedTickBehavior::Delay);
        let mut deadline = Instant::now() + self.timing.idle_limit;
        loop {
            tokio::select! {
                message = inbox.recv() => {
                    deadline = Instant::now() + self.timing.idle_limit;
                    match message {
                        Some(ExportMessage::Finished) => return Ok(()),
                        Some(message) => {
                            self.handle(output, message, &mut tracker, progress, started).await?;
                        }
                        None => return Err("the export stopped receiving data".to_owned()),
                    }
                }
                () = tokio::time::sleep_until(deadline) => {
                    return Err(format!(
                        "the editor window sent nothing for {} s: it may have frozen; export again",
                        self.timing.idle_limit.as_secs()
                    ));
                }
                _ = connection.tick() => {
                    if !self.renderer.is_connected() {
                        return Err("the editor window closed during the export".to_owned());
                    }
                }
            }
        }
    }

    async fn handle(
        &self,
        output: &mut dyn ExportOutput,
        message: ExportMessage,
        tracker: &mut ProgressTracker,
        progress: &dyn ProgressSink,
        started: Instant,
    ) -> Result<(), String> {
        match message {
            ExportMessage::Write {
                stream,
                position,
                bytes,
                reply,
            } => {
                let written = output.write(stream, position, &bytes).await;
                answer(reply, written)
            }
            ExportMessage::Frame { rgba, reply } => answer(reply, output.write_frame(&rgba).await),
            ExportMessage::Progress { done, total } => {
                let fraction = fraction(done, total);
                progress.report(fraction);
                if let Some(state) = tracker.next(fraction, started.elapsed().as_secs_f64()) {
                    self.registry.set_state(self.export_id, state);
                }
                Ok(())
            }
            ExportMessage::Failed { message } => {
                Err(format!("the editor could not render the export: {message}"))
            }
            ExportMessage::Finished => Ok(()),
        }
    }

    /// What ffprobe finds in the finished file (`None` when it cannot read it).
    async fn file_check(&self) -> Option<se_protocol::ExportCheck> {
        let settings = self.request.settings;
        let duration = super::span_duration(self.request.start, self.request.end);
        match self.prober.probe(&self.plan.path).await {
            Ok(probed) => Some(check::file_check(&probed.info, &settings, duration)),
            Err(error) => {
                tracing::warn!(export_id = %self.export_id, %error, "checking an export failed");
                None
            }
        }
    }
}

/// Passes a write's outcome to the uploader; a failed write fails the export.
fn answer(
    reply: oneshot::Sender<Result<(), MediaError>>,
    outcome: Result<(), MediaError>,
) -> Result<(), String> {
    let failure = outcome
        .as_ref()
        .err()
        .map(|error| format!("writing the export failed: {error}"));
    // The uploader may have given up waiting: the outcome still decides the export.
    let _ = reply.send(outcome);
    failure.map_or(Ok(()), Err)
}

fn fraction(done: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    // Counts stay far below 2^52: the conversions are exact.
    #[allow(clippy::cast_precision_loss)]
    let fraction = done.min(total) as f64 / total as f64;
    fraction
}

/// Publishes progress at most once per percent (and when the time left first appears).
#[derive(Default)]
struct ProgressTracker {
    last_percent: Option<u32>,
    had_eta: bool,
}

impl ProgressTracker {
    fn next(&mut self, fraction: f64, elapsed_seconds: f64) -> Option<ExportState> {
        let eta_seconds = check::eta_seconds(elapsed_seconds, fraction);
        // `fraction` lies in [0, 1]: the percent fits.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let percent = (fraction * 100.0).floor() as u32;
        let first_eta = eta_seconds.is_some() && !self.had_eta;
        if self.last_percent == Some(percent) && !first_eta {
            return None;
        }
        self.last_percent = Some(percent);
        self.had_eta |= eta_seconds.is_some();
        Some(ExportState::Rendering {
            progress: fraction,
            eta_seconds,
        })
    }
}

/// Marks the export cancelled when the job's future is dropped before it ends (the job was
/// cancelled: the output, dropped too, removes the partial files).
struct CancelOnDrop {
    registry: Arc<ExportRegistry>,
    export_id: ExportId,
    armed: bool,
}

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        if self.armed {
            self.registry.mark_cancelled(self.export_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractions_are_bounded() {
        assert!((fraction(0, 0) - 0.0).abs() < f64::EPSILON);
        assert!((fraction(5, 10) - 0.5).abs() < f64::EPSILON);
        assert!((fraction(12, 10) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn progress_is_published_once_per_percent() {
        let mut tracker = ProgressTracker::default();
        assert!(tracker.next(0.001, 0.1).is_some());
        assert!(tracker.next(0.004, 0.2).is_none());
        assert!(tracker.next(0.011, 0.3).is_some());
        // The time left appears within the same percent: published once.
        assert!(tracker.next(0.015, 5.0).is_none());
        assert!(matches!(
            tracker.next(0.021, 5.0),
            Some(ExportState::Rendering {
                eta_seconds: Some(_),
                ..
            })
        ));
    }
}
