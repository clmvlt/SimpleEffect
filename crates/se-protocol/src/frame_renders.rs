//! Frames of a sequence rendered by the renderer for AI agents (prompt 18, ADR-0028).
//!
//! The core cannot composite (the engine lives in the renderer, golden rule 7): when an agent asks
//! for frames (`get_frames`), the core publishes [`crate::CoreEvent::RenderFramesRequested`], the
//! renderer draws each time offscreen with `@se/engine` and uploads one JPEG per frame with
//! `PUT /media/renders/{requestId}/{index}`, or reports why it cannot with
//! [`crate::Command::ReportFrameRenderFailure`].

use se_domain::{Flicks, ProjectId, SequenceId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Frames of a sequence the renderer must draw and upload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FrameRenderRequest {
    /// Identifies the request in the upload route and the failure report (a UUID).
    pub request_id: String,
    /// The project (the renderer draws only the project it has open).
    pub project_id: ProjectId,
    /// The sequence to draw.
    pub sequence_id: SequenceId,
    /// Sequence times to draw, in order; frame `i` is uploaded at index `i`.
    pub times: Vec<Flicks>,
    /// Width of the pictures, in pixels.
    pub width: u32,
    /// Height of the pictures, in pixels (the sequence's aspect).
    pub height: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_are_camel_case() {
        let request = FrameRenderRequest {
            request_id: "r1".to_owned(),
            project_id: ProjectId::from_test_index(1),
            sequence_id: SequenceId::from_test_index(2),
            times: vec![Flicks::ZERO, Flicks::one_second()],
            width: 768,
            height: 432,
        };
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["requestId"], "r1");
        assert_eq!(json["times"][1], Flicks::PER_SECOND);
        assert_eq!(json["height"], 432);
    }
}
