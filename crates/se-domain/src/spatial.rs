//! Spatial units of the document (ADR-0015).
//!
//! - **Positions** are normalised in the sequence frame: `(0, 0)` is the top-left corner,
//!   `(1, 1)` the bottom-right one, whatever the format. Values outside `[0, 1]` are off-screen.
//! - **Anchors** are normalised in the item's own box: `(0.5, 0.5)` is its centre.
//! - **Scales** are relative: `(1, 1)` is the natural size; negative values mirror.
//! - **Rotations** are degrees, clockwise (the y axis points down).
//! - **Sizes** (font size, stroke width, corner radius, shape extents…) are [`DesignPixels`]:
//!   pixels of a frame whose shorter side is [`DESIGN_REFERENCE_SIDE`] (1080), so one design pixel
//!   is one real pixel in every 1080-class format (16:9, 9:16, 1:1, 4:5) and two in 4K.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Shorter side, in real pixels, of the frame in which a design pixel is one real pixel.
pub const DESIGN_REFERENCE_SIDE: u32 = 1080;

/// A pair of numbers: a normalised position or anchor, or a relative scale.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, TS)]
pub struct Vec2 {
    /// Horizontal component (rightward).
    pub x: f64,
    /// Vertical component (downward).
    pub y: f64,
}

impl Vec2 {
    /// Both components set to the same value.
    #[must_use]
    pub const fn splat(value: f64) -> Self {
        Self { x: value, y: value }
    }

    /// Builds a vector from its components.
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// A length in design pixels (see the module documentation and ADR-0015).
///
/// Serialized as a plain number.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize, TS)]
pub struct DesignPixels(pub f64);

impl DesignPixels {
    /// The length in real pixels of a `width` × `height` frame.
    #[must_use]
    pub fn to_output_pixels(self, width: u32, height: u32) -> f64 {
        self.0 * design_scale(width, height)
    }
}

/// Real pixels per design pixel in a `width` × `height` frame: shorter side / 1080.
#[must_use]
pub fn design_scale(width: u32, height: u32) -> f64 {
    f64::from(width.min(height)) / f64::from(DESIGN_REFERENCE_SIDE)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
    }

    #[test]
    fn design_pixels_follow_the_shorter_side() {
        let size = DesignPixels(48.0);
        assert_close(size.to_output_pixels(1920, 1080), 48.0);
        assert_close(size.to_output_pixels(1080, 1920), 48.0);
        assert_close(size.to_output_pixels(1080, 1350), 48.0);
        assert_close(size.to_output_pixels(3840, 2160), 96.0);
        assert_close(size.to_output_pixels(1280, 720), 32.0);
    }

    #[test]
    fn serializes_as_plain_numbers() {
        assert_eq!(serde_json::to_string(&DesignPixels(12.5)).unwrap(), "12.5");
        assert_eq!(
            serde_json::to_string(&Vec2::new(0.5, 1.0)).unwrap(),
            r#"{"x":0.5,"y":1.0}"#
        );
    }
}
