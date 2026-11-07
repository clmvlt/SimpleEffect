//! [`ImageItem`]: a still image asset placed on a video track.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::Transform2D;
use crate::ids::AssetId;

/// Shows an image asset (a logo, a photo, a sticker) for the item's duration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ImageItem {
    /// The image asset (its kind must be `image`).
    pub asset_id: AssetId,
    /// How the image's natural box relates to the frame.
    #[serde(default)]
    pub fit: ImageFit,
    /// Placement of the image.
    pub transform: Transform2D,
}

impl ImageItem {
    /// A new image item showing `asset_id` fitted into the frame, centred.
    #[must_use]
    pub fn new(asset_id: AssetId) -> Self {
        Self {
            asset_id,
            fit: ImageFit::default(),
            transform: Transform2D::default(),
        }
    }
}

/// The natural box of an image (its size at scale 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum ImageFit {
    /// As large as possible inside the frame, keeping its proportions.
    #[default]
    Fit,
    /// Covers the whole frame, keeping its proportions (edges may be cropped).
    Fill,
    /// One image pixel per design pixel (a 400 px logo is 400 px wide in a 1080p frame).
    Natural,
}
