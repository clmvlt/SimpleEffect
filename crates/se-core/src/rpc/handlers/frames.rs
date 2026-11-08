//! Frames drawn by the renderer for AI agents (ADR-0028): the renderer reports here when it
//! cannot draw a `renderFramesRequested` (frames themselves arrive by `PUT /media/renders`).

use se_protocol::{CommandResult, ProtocolError};

use super::HandlerContext;

/// `reportFrameRenderFailure`: the agent waiting for `request_id` gets `message`.
///
/// # Errors
/// `RENDER_REQUEST_NOT_FOUND` when nothing waits for it any more.
pub fn report_failure(
    context: &HandlerContext,
    request_id: &str,
    message: &str,
) -> Result<CommandResult, ProtocolError> {
    context.renders.fail(request_id, message)?;
    Ok(CommandResult::ReportFrameRenderFailure)
}
