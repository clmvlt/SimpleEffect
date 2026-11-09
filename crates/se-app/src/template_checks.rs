//! [`TemplateCheckService`]: motion template modules checked by the renderer (prompt 24,
//! ADR-0035), then read into a template definition.
//!
//! Only the renderer runs template code (its sandboxed Workers). A check goes out through the
//! [`RendererLink`] (the `templateCheckRequested` event); the renderer answers with
//! `reportTemplateCheck` ([`TemplateCheckService::deliver`]). The caller waits at most
//! [`CHECK_TIMEOUT`]. A passed check is then read by the domain ([`checked_definition`]): what
//! `meta` and `params` declare must hold too, or the agent gets those problems.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
    time::Duration,
};

use se_domain::{Format, ProjectId, TemplateDefinition, parse_definition_json};
use se_protocol::{
    TemplateCheckFrame, TemplateCheckOutcome, TemplateCheckRequest, TemplateCheckStage,
    TemplateProblem,
};
use tokio::sync::oneshot;

use crate::{
    AppError,
    frame_renders::frame_size,
    ports::{IdGenerator, RendererLink},
};

/// Longest wait for the renderer's verdict (compilation, fonts, a dozen renders).
pub const CHECK_TIMEOUT: Duration = Duration::from_secs(45);

/// Width of the pictures a check returns, at most.
pub const CHECK_PICTURE_WIDTH: u32 = 640;

/// A module that passed the renderer's check and the domain's reading.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckedTemplate {
    /// What `meta` and `params` declare.
    pub definition: TemplateDefinition,
    /// Entrance, middle and exit, rendered with the default values.
    pub frames: Vec<TemplateCheckFrame>,
    /// Longest render of one frame during the check, in milliseconds.
    pub slowest_frame_ms: f64,
}

type PendingMap = HashMap<String, oneshot::Sender<TemplateCheckOutcome>>;

/// Sends template checks to the renderer and collects its verdicts.
pub struct TemplateCheckService {
    link: Arc<dyn RendererLink>,
    ids: Arc<dyn IdGenerator>,
    timeout: Duration,
    pending: Arc<Mutex<PendingMap>>,
}

impl TemplateCheckService {
    /// A service waiting at most [`CHECK_TIMEOUT`] per check.
    #[must_use]
    pub fn new(link: Arc<dyn RendererLink>, ids: Arc<dyn IdGenerator>) -> Self {
        Self::with_timeout(link, ids, CHECK_TIMEOUT)
    }

    /// A service waiting at most `timeout` per check (tests).
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

    /// Checks `source` for a sequence in `format`: compiled and run by the renderer, then its
    /// declarations read by the domain.
    ///
    /// # Errors
    /// [`AppError::TemplateInvalid`] with every problem found;
    /// [`AppError::RendererUnavailable`] when no editor window is connected or none answered in
    /// time.
    pub async fn check(
        &self,
        project_id: ProjectId,
        format: &Format,
        source: &str,
    ) -> Result<CheckedTemplate, AppError> {
        match self.outcome(project_id, format, source).await? {
            TemplateCheckOutcome::Passed {
                meta_json,
                params_json,
                frames,
                slowest_frame_ms,
            } => Ok(CheckedTemplate {
                definition: checked_definition(&meta_json, &params_json)?,
                frames,
                slowest_frame_ms,
            }),
            TemplateCheckOutcome::Failed { problems } => {
                Err(AppError::TemplateInvalid { problems })
            }
        }
    }

    /// The renderer's raw verdict on `source`.
    async fn outcome(
        &self,
        project_id: ProjectId,
        format: &Format,
        source: &str,
    ) -> Result<TemplateCheckOutcome, AppError> {
        if !self.link.is_connected() {
            return Err(AppError::RendererUnavailable {
                reason: "no SimpleEffect window is open to run the template: open the project in \
                         the editor"
                    .to_owned(),
            });
        }
        let request_id = self.ids.new_uuid_v7().to_string();
        let (sender, receiver) = oneshot::channel();
        lock(&self.pending).insert(request_id.clone(), sender);
        let _forget = ForgetOnDrop {
            pending: Arc::clone(&self.pending),
            request_id: request_id.clone(),
        };
        let (width, height) = design_frame(format);
        let (picture_width, picture_height) = frame_size(format, CHECK_PICTURE_WIDTH);
        self.link.request_template_check(TemplateCheckRequest {
            request_id,
            project_id,
            source: source.to_owned(),
            width,
            height,
            picture_width,
            picture_height,
        });
        match tokio::time::timeout(self.timeout, receiver).await {
            Ok(Ok(outcome)) => Ok(outcome),
            Ok(Err(_)) => Err(AppError::Internal(
                "a template check was dropped without an answer".to_owned(),
            )),
            Err(_) => Err(AppError::RendererUnavailable {
                reason: format!(
                    "the editor window did not check the template within {} s: make sure the \
                     project is open in SimpleEffect, then try again",
                    self.timeout.as_secs()
                ),
            }),
        }
    }

    /// Records the renderer's verdict on check `request_id`.
    ///
    /// # Errors
    /// [`AppError::TemplateCheckNotFound`] when nothing waits for it (finished or timed out).
    pub fn deliver(&self, request_id: &str, outcome: TemplateCheckOutcome) -> Result<(), AppError> {
        let sender = lock(&self.pending).remove(request_id).ok_or_else(|| {
            AppError::TemplateCheckNotFound {
                request_id: request_id.to_owned(),
            }
        })?;
        // The caller may have stopped waiting: nobody to tell then.
        let _ = sender.send(outcome);
        Ok(())
    }

    /// How many checks wait for a verdict.
    #[must_use]
    pub fn pending_count(&self) -> usize {
        lock(&self.pending).len()
    }
}

/// The frame a template draws in for a sequence in `format`: design pixels, shorter side 1080
/// (ADR-0015), the sequence's aspect, even sides.
#[must_use]
pub fn design_frame(format: &Format) -> (u32, u32) {
    let shorter = f64::from(format.width.min(format.height).max(1));
    let scale = f64::from(se_domain::DESIGN_REFERENCE_SIDE) / shorter;
    let side = |value: u32| {
        let scaled = (f64::from(value) * scale / 2.0).round() * 2.0;
        // Formats are at most 16384 pixels a side: the scaled side always fits.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let whole = scaled.clamp(2.0, 65_536.0) as u32;
        whole
    };
    (side(format.width), side(format.height))
}

/// What a passed module declares, read by the domain.
///
/// # Errors
/// [`AppError::TemplateInvalid`] with the problems of `meta` and `params` (stage `definition`).
pub fn checked_definition(
    meta_json: &str,
    params_json: &str,
) -> Result<TemplateDefinition, AppError> {
    parse_definition_json(meta_json, params_json).map_err(|problems| AppError::TemplateInvalid {
        problems: problems
            .into_iter()
            .map(|problem| definition_problem(format!("{}: {}", problem.path, problem.message)))
            .collect(),
    })
}

/// A problem of what the module declares.
#[must_use]
pub fn definition_problem(message: String) -> TemplateProblem {
    TemplateProblem {
        stage: TemplateCheckStage::Definition,
        message,
        line: None,
        column: None,
        time: None,
    }
}

/// Removes a check from the waiting ones when its caller is gone.
struct ForgetOnDrop {
    pending: Arc<Mutex<PendingMap>>,
    request_id: String,
}

impl Drop for ForgetOnDrop {
    fn drop(&mut self) {
        lock(&self.pending).remove(&self.request_id);
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
    fn design_frames_have_a_shorter_side_of_1080() {
        assert_eq!(design_frame(&format(3840, 2160)), (1920, 1080));
        assert_eq!(design_frame(&format(720, 1280)), (1080, 1920));
        assert_eq!(design_frame(&format(1080, 1080)), (1080, 1080));
        assert_eq!(design_frame(&format(1000, 750)), (1440, 1080));
    }

    #[test]
    fn declarations_are_read_by_the_domain() {
        let meta = r#"{"name":"Title","defaultDuration":4}"#;
        let params = r#"[["title",{"type":"string","default":"Hi"}]]"#;
        let definition = checked_definition(meta, params).unwrap();
        assert_eq!(definition.params[0].name, "title");
        let Err(AppError::TemplateInvalid { problems }) =
            checked_definition(r#"{"name":"","defaultDuration":4}"#, "[]")
        else {
            panic!("an empty name is refused");
        };
        assert_eq!(problems[0].stage, TemplateCheckStage::Definition);
        assert!(problems[0].message.starts_with("meta.name: "));
    }
}
