//! [`FrameRenderService`]: composited frames of a sequence, drawn by the renderer for AI agents
//! (`get_frames`, ADR-0028).
//!
//! Only the renderer can composite (the engine lives there, golden rule 7: what agents see is
//! what the viewer and the export show). A request goes out through the [`RendererLink`] (the
//! `renderFramesRequested` event); the renderer uploads one JPEG per time
//! ([`FrameRenderService::deliver`], route `PUT /media/renders/{id}/{index}`) or reports why it
//! cannot ([`FrameRenderService::fail`]). The caller waits at most [`RENDER_TIMEOUT`].

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    time::Duration,
};

use se_domain::{Flicks, Format, ProjectId, SequenceId};
use se_protocol::FrameRenderRequest;
use tokio::sync::oneshot;

use crate::{
    AppError,
    ports::{IdGenerator, RendererLink},
};

/// Longest wait for the renderer to deliver every frame of a request.
pub const RENDER_TIMEOUT: Duration = Duration::from_secs(30);

/// Most frames one request may ask for (each costs the agent hundreds of tokens).
pub const MAX_RENDER_FRAMES: usize = 16;

/// Largest picture accepted from the renderer, in bytes.
pub const MAX_RENDERED_FRAME_BYTES: usize = 8 * 1024 * 1024;

/// Narrowest picture rendered, in pixels.
pub const MIN_RENDER_WIDTH: u32 = 64;

/// Widest picture rendered, in pixels.
pub const MAX_RENDER_WIDTH: u32 = 1920;

/// Frames of a sequence to render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameRenderSpec {
    /// The open project.
    pub project_id: ProjectId,
    /// The sequence.
    pub sequence_id: SequenceId,
    /// Sequence times, in order (at most [`MAX_RENDER_FRAMES`]).
    pub times: Vec<Flicks>,
    /// Size of the pictures (see [`frame_size`]).
    pub width: u32,
    /// Height of the pictures.
    pub height: u32,
}

/// The size of pictures of a sequence in `format` at most `max_width` wide: never wider than the
/// sequence, at least [`MIN_RENDER_WIDTH`], even dimensions (what JPEG encoders like), the
/// sequence's aspect.
#[must_use]
pub fn frame_size(format: &Format, max_width: u32) -> (u32, u32) {
    let width = max_width
        .clamp(MIN_RENDER_WIDTH, MAX_RENDER_WIDTH)
        .min(format.width.max(MIN_RENDER_WIDTH));
    let height = u64::from(width) * u64::from(format.height) / u64::from(format.width.max(1));
    let even = |value: u64| u32::try_from(value.max(2) & !1).unwrap_or(u32::MAX);
    (even(u64::from(width)), even(height))
}

/// A request waiting for its frames.
struct PendingRender {
    frames: Vec<Option<Vec<u8>>>,
    done: Option<oneshot::Sender<Result<Vec<Vec<u8>>, AppError>>>,
}

type PendingMap = HashMap<String, PendingRender>;

/// Sends frame requests to the renderer and collects the pictures it uploads.
pub struct FrameRenderService {
    link: Arc<dyn RendererLink>,
    ids: Arc<dyn IdGenerator>,
    timeout: Duration,
    pending: Arc<Mutex<PendingMap>>,
}

impl FrameRenderService {
    /// A service waiting at most [`RENDER_TIMEOUT`] per request.
    #[must_use]
    pub fn new(link: Arc<dyn RendererLink>, ids: Arc<dyn IdGenerator>) -> Self {
        Self::with_timeout(link, ids, RENDER_TIMEOUT)
    }

    /// A service waiting at most `timeout` per request (tests).
    #[must_use]
    pub fn with_timeout(
        link: Arc<dyn RendererLink>,
        ids: Arc<dyn IdGenerator>,
        timeout: Duration,
    ) -> Self {
        Self {
            link,
            ids,
            timeout,
            pending: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// The JPEG of each time of `spec`, in order.
    ///
    /// # Errors
    /// [`AppError::InvalidRender`] for too many frames or a bad size;
    /// [`AppError::RendererUnavailable`] when no editor window is connected or none answered in
    /// time; [`AppError::RenderFailed`] when the renderer reported a failure.
    pub async fn render(&self, spec: FrameRenderSpec) -> Result<Vec<Vec<u8>>, AppError> {
        check_spec(&spec)?;
        if spec.times.is_empty() {
            return Ok(Vec::new());
        }
        if !self.link.is_connected() {
            return Err(AppError::RendererUnavailable {
                reason: "no SimpleEffect window is open: open the project in the editor".to_owned(),
            });
        }
        let request_id = self.ids.new_uuid_v7().to_string();
        let (sender, receiver) = oneshot::channel();
        lock(&self.pending).insert(
            request_id.clone(),
            PendingRender {
                frames: vec![None; spec.times.len()],
                done: Some(sender),
            },
        );
        // Forgets the request if the caller stops waiting (agent cancelled, timeout).
        let _forget = ForgetOnDrop {
            pending: Arc::clone(&self.pending),
            request_id: request_id.clone(),
        };
        self.link.request_frames(FrameRenderRequest {
            request_id,
            project_id: spec.project_id,
            sequence_id: spec.sequence_id,
            times: spec.times,
            width: spec.width,
            height: spec.height,
        });
        match tokio::time::timeout(self.timeout, receiver).await {
            Ok(Ok(outcome)) => outcome,
            Ok(Err(_)) => Err(AppError::Internal(
                "a frame render was dropped without an answer".to_owned(),
            )),
            Err(_) => Err(AppError::RendererUnavailable {
                reason: format!(
                    "the editor window did not draw the frames within {} s: make sure the \
                     project is open in SimpleEffect, then try again",
                    self.timeout.as_secs()
                ),
            }),
        }
    }

    /// Records frame `index` of request `request_id`; the request is answered once every frame
    /// arrived.
    ///
    /// # Errors
    /// [`AppError::RenderRequestNotFound`] when nothing waits for that request (finished, failed
    /// or timed out); [`AppError::InvalidRender`] for an index out of range or bytes that are not
    /// a JPEG.
    pub fn deliver(&self, request_id: &str, index: usize, jpeg: Vec<u8>) -> Result<(), AppError> {
        check_jpeg(&jpeg)?;
        let mut pending = lock(&self.pending);
        let render = pending
            .get_mut(request_id)
            .ok_or_else(|| not_found(request_id))?;
        let count = render.frames.len();
        let slot = render
            .frames
            .get_mut(index)
            .ok_or_else(|| AppError::InvalidRender {
                reason: format!("frame {index} is out of range: the request has {count} frames"),
            })?;
        *slot = Some(jpeg);
        if render.frames.iter().all(Option::is_some)
            && let Some(mut render) = pending.remove(request_id)
        {
            let frames = render.frames.drain(..).flatten().collect();
            answer(&mut render, Ok(frames));
        }
        Ok(())
    }

    /// Answers request `request_id` with the renderer's failure `message`.
    ///
    /// # Errors
    /// [`AppError::RenderRequestNotFound`].
    pub fn fail(&self, request_id: &str, message: &str) -> Result<(), AppError> {
        let mut render = lock(&self.pending)
            .remove(request_id)
            .ok_or_else(|| not_found(request_id))?;
        answer(
            &mut render,
            Err(AppError::RenderFailed {
                reason: message.to_owned(),
            }),
        );
        Ok(())
    }

    /// How many requests wait for frames.
    #[must_use]
    pub fn pending_count(&self) -> usize {
        lock(&self.pending).len()
    }
}

/// Removes a request from the waiting ones when its caller is gone.
struct ForgetOnDrop {
    pending: Arc<Mutex<PendingMap>>,
    request_id: String,
}

impl Drop for ForgetOnDrop {
    fn drop(&mut self) {
        lock(&self.pending).remove(&self.request_id);
    }
}

fn answer(render: &mut PendingRender, outcome: Result<Vec<Vec<u8>>, AppError>) {
    if let Some(done) = render.done.take() {
        // The caller may have stopped waiting: nobody to tell then.
        let _ = done.send(outcome);
    }
}

fn check_spec(spec: &FrameRenderSpec) -> Result<(), AppError> {
    if spec.times.len() > MAX_RENDER_FRAMES {
        return Err(AppError::InvalidRender {
            reason: format!(
                "{} frames asked; ask for at most {MAX_RENDER_FRAMES} at once",
                spec.times.len()
            ),
        });
    }
    if spec.width < 2 || spec.height < 2 || spec.width > MAX_RENDER_WIDTH * 2 {
        return Err(AppError::InvalidRender {
            reason: format!("{}×{} is not a picture size", spec.width, spec.height),
        });
    }
    if let Some(time) = spec.times.iter().find(|time| time.is_negative()) {
        return Err(AppError::InvalidRender {
            reason: format!("time {} s is before the start", time.to_seconds_f64()),
        });
    }
    Ok(())
}

fn check_jpeg(bytes: &[u8]) -> Result<(), AppError> {
    if bytes.len() > MAX_RENDERED_FRAME_BYTES {
        return Err(AppError::InvalidRender {
            reason: format!(
                "the picture weighs {} bytes; at most {MAX_RENDERED_FRAME_BYTES} are accepted",
                bytes.len()
            ),
        });
    }
    if !bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Err(AppError::InvalidRender {
            reason: "the uploaded picture is not a JPEG".to_owned(),
        });
    }
    Ok(())
}

fn not_found(request_id: &str) -> AppError {
    AppError::RenderRequestNotFound {
        request_id: request_id.to_owned(),
    }
}

fn lock(pending: &Mutex<PendingMap>) -> MutexGuard<'_, PendingMap> {
    // Entries are inserted and removed whole.
    pending.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use se_domain::FrameRate;

    use super::*;

    fn format(width: u32, height: u32) -> Format {
        Format {
            width,
            height,
            frame_rate: FrameRate::FPS_25,
            sample_rate: 48_000,
        }
    }

    #[test]
    fn sizes_keep_the_aspect_with_even_sides() {
        assert_eq!(frame_size(&format(1920, 1080), 768), (768, 432));
        assert_eq!(frame_size(&format(1080, 1920), 768), (768, 1364));
        assert_eq!(frame_size(&format(1280, 720), 4000), (1280, 720));
        assert_eq!(frame_size(&format(1920, 1080), 10), (64, 36));
        assert_eq!(frame_size(&format(1001, 1001), 1001), (1000, 1000));
    }

    #[test]
    fn jpeg_checks_refuse_other_bytes() {
        assert!(check_jpeg(&[0xFF, 0xD8, 0xFF, 0xE0]).is_ok());
        assert!(matches!(
            check_jpeg(b"\x89PNG"),
            Err(AppError::InvalidRender { .. })
        ));
    }
}
