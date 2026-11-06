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
    /// Placement of the image.
    pub transform: Transform2D,
}
