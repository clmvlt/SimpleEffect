//! [`ShapeItem`]: a vector shape drawn by the engine.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::Transform2D;
use crate::{color::Color, spatial::DesignPixels};

/// A filled vector shape.
///
/// TODO(prompt-15): stroke, gradients, more geometries and animatable dimensions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ShapeItem {
    /// The outline.
    pub geometry: ShapeGeometry,
    /// Fill colour.
    pub fill: Color,
    /// Placement of the shape's box.
    pub transform: Transform2D,
}

/// The outline of a shape, sized in design pixels (ADR-0015).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ShapeGeometry {
    /// A rectangle, optionally with rounded corners.
    Rectangle {
        /// Width of the box.
        width: DesignPixels,
        /// Height of the box.
        height: DesignPixels,
        /// Radius of the rounded corners (0 for square corners).
        corner_radius: DesignPixels,
    },
    /// An ellipse inscribed in its box.
    Ellipse {
        /// Width of the box.
        width: DesignPixels,
        /// Height of the box.
        height: DesignPixels,
    },
}
