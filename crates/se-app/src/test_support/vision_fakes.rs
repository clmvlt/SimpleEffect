//! Fakes of automatic reframing (prompt 26): [`FakeFrameSampler`] (frames without pixels, one
//! per interval over a chosen duration) and [`FakeSubjectDetector`] (faces given by a function of
//! the frame's time; available or not).

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, PoisonError},
};

use async_trait::async_trait;
use se_domain::{FaceBox, Flicks};
use se_protocol::VisionStatus;
use tokio::sync::mpsc;

use crate::ports::{
    FrameSampleRequest, FrameSampler, MediaError, SampledFrame, SubjectDetector, VisionError,
};

/// Seconds of video a source is sampled over unless set.
const DEFAULT_SECONDS: u32 = 10;

#[derive(Default)]
struct SamplerState {
    seconds: HashMap<PathBuf, u32>,
    requests: Vec<FrameSampleRequest>,
    failure: Option<String>,
    held: bool,
    default_seconds: Option<u32>,
}

/// A frame sampler sending one picture (without pixels) per interval over a source's duration
/// (10 s unless set). Clones share state.
#[derive(Clone, Default)]
pub struct FakeFrameSampler {
    state: Arc<Mutex<SamplerState>>,
    released: Arc<tokio::sync::Notify>,
}

impl FakeFrameSampler {
    fn state(&self) -> std::sync::MutexGuard<'_, SamplerState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Samples `source` over `seconds`.
    pub fn set_duration(&self, source: impl Into<PathBuf>, seconds: u32) {
        self.state().seconds.insert(source.into(), seconds);
    }

    /// Samples sources without a duration of their own over `seconds` (10 by default).
    pub fn set_default_duration(&self, seconds: u32) {
        self.state().default_seconds = Some(seconds);
    }

    /// Every sampling fails with `message` (or succeeds again with `None`).
    pub fn set_failure(&self, message: Option<&str>) {
        self.state().failure = message.map(str::to_owned);
    }

    /// Samplings wait (before their first picture) until [`FakeFrameSampler::release`].
    pub fn hold(&self) {
        self.state().held = true;
    }

    /// Lets held and future samplings run.
    pub fn release(&self) {
        self.state().held = false;
        self.released.notify_waiters();
    }

    /// Every request received.
    #[must_use]
    pub fn requests(&self) -> Vec<FrameSampleRequest> {
        self.state().requests.clone()
    }
}

#[async_trait]
impl FrameSampler for FakeFrameSampler {
    async fn sample_frames(
        &self,
        request: &FrameSampleRequest,
        frames: mpsc::Sender<SampledFrame>,
    ) -> Result<(), MediaError> {
        let seconds = {
            let mut state = self.state();
            state.requests.push(request.clone());
            if let Some(message) = &state.failure {
                return Err(MediaError::Failed(message.clone()));
            }
            state
                .seconds
                .get(&request.source)
                .copied()
                .or(state.default_seconds)
                .unwrap_or(DEFAULT_SECONDS)
        };
        loop {
            let released = self.released.notified();
            if !self.state().held {
                break;
            }
            released.await;
        }
        let total = Flicks::one_second().get() * i64::from(seconds);
        let step = request.interval.get().max(1);
        let mut time = 0;
        while time < total {
            let frame = SampledFrame {
                time: Flicks::new(time).unwrap_or(Flicks::ZERO),
                width: request.width,
                height: request.height,
                pixels: Vec::new(),
            };
            if frames.send(frame).await.is_err() {
                return Ok(());
            }
            time += step;
        }
        Ok(())
    }
}

/// The faces of a frame at a given time.
pub type FaceScript = Arc<dyn Fn(Flicks) -> Vec<FaceBox> + Send + Sync>;

struct DetectorState {
    available: Result<(), String>,
    faces: FaceScript,
    detected: usize,
}

/// A detector whose faces come from a script (by default one face left of centre, at
/// `x = 0.25…0.37`, in every frame). Clones share state.
#[derive(Clone)]
pub struct FakeSubjectDetector {
    state: Arc<Mutex<DetectorState>>,
}

impl Default for FakeSubjectDetector {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(DetectorState {
                available: Ok(()),
                faces: Arc::new(|_| vec![left_face()]),
                detected: 0,
            })),
        }
    }
}

/// The default face: left of centre, a medium shot.
#[must_use]
pub const fn left_face() -> FaceBox {
    FaceBox {
        x: 0.25,
        y: 0.2,
        width: 0.12,
        height: 0.16,
        score: 0.92,
    }
}

impl FakeSubjectDetector {
    fn state(&self) -> std::sync::MutexGuard<'_, DetectorState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Makes detection unavailable for `reason` (or available again with `None`).
    pub fn set_unavailable(&self, reason: Option<&str>) {
        self.state().available = reason.map_or(Ok(()), |reason| Err(reason.to_owned()));
    }

    /// Uses `script` for the faces of every frame.
    pub fn set_faces(&self, script: FaceScript) {
        self.state().faces = script;
    }

    /// Number of frames detected so far.
    #[must_use]
    pub fn detected(&self) -> usize {
        self.state().detected
    }
}

#[async_trait]
impl SubjectDetector for FakeSubjectDetector {
    fn model(&self) -> &'static str {
        "fake-faces"
    }

    async fn status(&self) -> VisionStatus {
        let reason = self.state().available.clone().err();
        VisionStatus {
            available: reason.is_none(),
            model: "fake-faces".to_owned(),
            runtime: Some("Fake runtime".to_owned()),
            accelerator: Some("CPU".to_owned()),
            reason,
        }
    }

    async fn detect(&self, frames: Vec<SampledFrame>) -> Result<Vec<Vec<FaceBox>>, VisionError> {
        let mut state = self.state();
        if let Err(reason) = &state.available {
            return Err(VisionError::Unavailable {
                reason: reason.clone(),
            });
        }
        state.detected += frames.len();
        let script = Arc::clone(&state.faces);
        drop(state);
        Ok(frames.iter().map(|frame| script(frame.time)).collect())
    }
}
