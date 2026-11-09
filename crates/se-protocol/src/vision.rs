//! Automatic reframing DTOs (prompt 26, ADR-0037): whether faces can be detected on this
//! machine, and what reframing did with the clips it was given.

use se_domain::{AssetId, ItemId};
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

/// What automatic reframing did with the clips it was given (`autoReframeItems`, and versions
/// made with automatic reframing).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AutoReframeReport {
    /// Clips reframed now (their faces were already analysed), in one undoable step.
    pub applied_item_ids: Vec<ItemId>,
    /// Media whose faces are being analysed: their clips are reframed when the analysis ends
    /// (`autoReframeFinished`), in one more undoable step.
    pub waiting_asset_ids: Vec<AssetId>,
    /// Clips left as they are, and why.
    pub skipped: Vec<SkippedReframe>,
}

/// A clip automatic reframing cannot follow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SkippedReframe {
    /// The item.
    pub item_id: ItemId,
    /// Why it was left.
    pub reason: ReframeSkipReason,
}

/// Why a clip was not reframed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum ReframeSkipReason {
    /// Not a media clip on a video track (texts, shapes, sound).
    NotAVideoClip,
    /// Its media has no picture, or a picture of unknown size.
    NoPicture,
    /// Its original file is missing.
    MediaMissing,
    /// It lies on a locked track.
    LockedTrack,
}

/// A media whose faces could not be analysed (`autoReframeFinished`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ReframeFailure {
    /// The media.
    pub asset_id: AssetId,
    /// Why (actionable).
    pub message: String,
}
