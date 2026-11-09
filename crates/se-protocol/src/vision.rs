//! Automatic reframing DTOs (prompt 26, ADR-0037): whether faces can be detected on this machine.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Whether the face detector can run here (`getVisionStatus`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct VisionStatus {
    /// Detection can run.
    pub available: bool,
    /// The detection model (`yunet-2026may`), bundled with the app.
    pub model: String,
    /// The inference library and its version (`ONNX Runtime 1.31.0`) once loaded.
    pub runtime: Option<String>,
    /// Where inference runs (`CPU`, `CoreML`) once loaded.
    pub accelerator: Option<String>,
    /// Why detection cannot run (missing library, Intel Mac…), when it cannot.
    pub reason: Option<String>,
}
