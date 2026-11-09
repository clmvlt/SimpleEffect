//! [`YuNetDetector`]: the `SubjectDetector` port over ONNX Runtime. One session of the embedded
//! YuNet model, created the first time it is needed and reused; inference runs on the blocking
//! pool, one picture at a time (the model takes any size).

use std::sync::{Arc, Mutex, PoisonError};

use async_trait::async_trait;
use ort::{
    session::{Session, builder::GraphOptimizationLevel},
    value::Tensor,
};
use se_app::ports::{SampledFrame, SubjectDetector, VisionError};
use se_domain::FaceBox;
use se_protocol::VisionStatus;

use crate::{
    platform::{accelerator, execution_providers},
    runtime::{RuntimeLocator, load},
    yunet::{MODEL_BYTES, MODEL_NAME, STRIDES, StrideOutputs, decode, input_tensor},
};

/// The face detector of automatic reframing (ADR-0037). Cheap to clone.
#[derive(Clone)]
pub struct YuNetDetector {
    inner: Arc<Inner>,
}

struct Inner {
    locator: RuntimeLocator,
    threads: usize,
    session: Mutex<Option<Session>>,
}

impl YuNetDetector {
    /// A detector loading ONNX Runtime from `locator`'s places, inferring on `threads` threads.
    #[must_use]
    pub fn new(locator: RuntimeLocator, threads: usize) -> Self {
        Self {
            inner: Arc::new(Inner {
                locator,
                threads: threads.max(1),
                session: Mutex::new(None),
            }),
        }
    }
}

impl Inner {
    /// Runs `work` with the session, creating it first if needed.
    fn with_session<T>(
        &self,
        work: impl FnOnce(&mut Session) -> Result<T, VisionError>,
    ) -> Result<T, VisionError> {
        let mut slot = self.session.lock().unwrap_or_else(PoisonError::into_inner);
        if slot.is_none() {
            *slot = Some(self.create_session()?);
        }
        match slot.as_mut() {
            Some(session) => work(session),
            None => Err(unavailable("the detection session could not be created")),
        }
    }

    fn create_session(&self) -> Result<Session, VisionError> {
        let runtime = load(&self.locator).map_err(|reason| VisionError::Unavailable { reason })?;
        let failed = |error: ort::Error| VisionError::Failed {
            message: format!("the face model could not be prepared: {error}"),
        };
        Session::builder(&runtime.environment)
            .map_err(failed)?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|error| failed(error.into()))?
            .with_intra_threads(self.threads)
            .map_err(|error| failed(error.into()))?
            .with_execution_providers(execution_providers())
            .map_err(|error| failed(error.into()))?
            .commit_from_memory(MODEL_BYTES)
            .map_err(failed)
    }

    fn status(&self) -> VisionStatus {
        let ready = self.with_session(|_| Ok(()));
        let runtime = load(&self.locator).ok();
        VisionStatus {
            available: ready.is_ok(),
            model: MODEL_NAME.to_owned(),
            runtime: runtime.as_ref().map(|runtime| runtime.description.clone()),
            accelerator: runtime.map(|_| accelerator().to_owned()),
            reason: ready.err().map(|error| match error {
                VisionError::Unavailable { reason } => reason,
                VisionError::Failed { message } => message,
            }),
        }
    }

    fn detect(&self, frames: &[SampledFrame]) -> Result<Vec<Vec<FaceBox>>, VisionError> {
        self.with_session(|session| {
            frames
                .iter()
                .map(|frame| detect_one(session, frame))
                .collect()
        })
    }
}

/// The faces of one picture.
fn detect_one(session: &mut Session, frame: &SampledFrame) -> Result<Vec<FaceBox>, VisionError> {
    let Some(tensor) = input_tensor(frame) else {
        return Err(VisionError::Failed {
            message: format!(
                "a sampled picture of {} × {} holds {} bytes instead of {}",
                frame.width,
                frame.height,
                frame.pixels.len(),
                u64::from(frame.width) * u64::from(frame.height) * 3
            ),
        });
    };
    let failed = |error: ort::Error| VisionError::Failed {
        message: format!(
            "the face model failed on the picture at {:?}: {error}",
            frame.time
        ),
    };
    let shape = [1_i64, 3, i64::from(tensor.height), i64::from(tensor.width)];
    let input = Tensor::from_array((shape, tensor.values)).map_err(failed)?;
    let outputs = session
        .run(ort::inputs!["input" => input])
        .map_err(failed)?;
    let mut grids = Vec::with_capacity(STRIDES.len());
    for stride in STRIDES {
        let slice = |name: String| -> Result<&[f32], VisionError> {
            let value = outputs
                .get(name.as_str())
                .ok_or_else(|| VisionError::Failed {
                    message: format!("the face model has no output {name}"),
                })?;
            value
                .try_extract_tensor::<f32>()
                .map(|(_, data)| data)
                .map_err(failed)
        };
        grids.push(StrideOutputs {
            stride,
            cls: slice(format!("cls_{stride}"))?,
            obj: slice(format!("obj_{stride}"))?,
            bbox: slice(format!("bbox_{stride}"))?,
        });
    }
    Ok(decode(&grids, tensor.width, frame.width, frame.height))
}

fn unavailable(reason: &str) -> VisionError {
    VisionError::Unavailable {
        reason: reason.to_owned(),
    }
}

#[async_trait]
impl SubjectDetector for YuNetDetector {
    fn model(&self) -> &str {
        MODEL_NAME
    }

    async fn status(&self) -> VisionStatus {
        let inner = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || inner.status())
            .await
            .unwrap_or_else(|error| VisionStatus {
                available: false,
                model: MODEL_NAME.to_owned(),
                runtime: None,
                accelerator: None,
                reason: Some(format!("the detector stopped unexpectedly: {error}")),
            })
    }

    async fn detect(&self, frames: Vec<SampledFrame>) -> Result<Vec<Vec<FaceBox>>, VisionError> {
        let inner = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || inner.detect(&frames))
            .await
            .map_err(|error| VisionError::Failed {
                message: format!("the detector stopped unexpectedly: {error}"),
            })?
    }
}
