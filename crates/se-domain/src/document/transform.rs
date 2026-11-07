//! [`Transform2D`]: where and how a visual item is drawn in the frame.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::Animatable;
use crate::spatial::{DesignPixels, Vec2};

/// Placement of a visual item, every property animatable. Units: ADR-0015 (`crate::spatial`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Transform2D {
    /// Where the anchor sits, normalised in the sequence frame (`(0.5, 0.5)` is the centre).
    pub position: Animatable<Vec2>,
    /// Relative size (`(1, 1)` is the natural size; negative mirrors).
    pub scale: Animatable<Vec2>,
    /// Clockwise rotation around the anchor, in degrees.
    pub rotation: Animatable<f64>,
    /// The item's pivot, normalised in its own box (`(0.5, 0.5)` is its centre).
    pub anchor: Animatable<Vec2>,
    /// Opacity from 0 (invisible) to 1 (opaque).
    pub opacity: Animatable<f64>,
    /// Gaussian blur radius of the whole item, in design pixels (0 = sharp); >= 0.
    #[serde(default = "no_blur")]
    pub blur: Animatable<DesignPixels>,
}

fn no_blur() -> Animatable<DesignPixels> {
    Animatable::constant(DesignPixels(0.0))
}

impl Default for Transform2D {
    /// Centred, natural size, upright, opaque and sharp.
    fn default() -> Self {
        Self {
            position: Animatable::constant(Vec2::splat(0.5)),
            scale: Animatable::constant(Vec2::splat(1.0)),
            rotation: Animatable::constant(0.0),
            anchor: Animatable::constant(Vec2::splat(0.5)),
            opacity: Animatable::constant(1.0),
            blur: no_blur(),
        }
    }
}
