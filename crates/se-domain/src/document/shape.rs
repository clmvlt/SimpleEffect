//! [`ShapeItem`]: a vector shape drawn by the engine.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Animatable, Stroke, Transform2D};
use crate::{color::Color, spatial::DesignPixels};

/// A filled vector shape with an optional outline.
///
/// Dimensions are static: animate `scale` to grow a shape (animatable dimensions and gradients
/// are in `docs/backlog.md`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ShapeItem {
    /// The outline.
    pub geometry: ShapeGeometry,
    /// Fill colour.
    pub fill: Animatable<Color>,
    /// Outline drawn along the shape's edge (outside it).
    #[serde(default)]
    pub stroke: Stroke,
    /// The box is the whole sequence frame, whatever its format (a backdrop, a colour plate):
    /// the geometry's width and height are ignored (ADR-0015). Not used by lines.
    #[serde(default)]
    pub cover_frame: bool,
    /// Placement of the shape's box.
    pub transform: Transform2D,
}

impl ShapeItem {
    /// A new shape of `kind` with the default look: white, centred, sized for a 1080p frame.
    #[must_use]
    pub fn new(kind: ShapeKind) -> Self {
        Self {
            geometry: ShapeGeometry::default_of(kind),
            fill: Animatable::constant(Color::WHITE),
            stroke: Stroke::default(),
            cover_frame: false,
            transform: Transform2D::default(),
        }
    }
}

/// The kinds of shapes offered to create (a rounded rectangle is a rectangle with a radius).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum ShapeKind {
    /// Square corners.
    Rectangle,
    /// Rounded corners.
    RoundedRectangle,
    /// An ellipse.
    Ellipse,
    /// A straight line.
    Line,
}

/// The outline of a shape, sized in design pixels (ADR-0015). Every extent is > 0.
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
        /// Radius of the rounded corners (0 for square corners); ≥ 0.
        corner_radius: DesignPixels,
    },
    /// An ellipse inscribed in its box.
    Ellipse {
        /// Width of the box.
        width: DesignPixels,
        /// Height of the box.
        height: DesignPixels,
    },
    /// A horizontal line with round caps, filled with the shape's fill (rotate the item to tilt
    /// it). Its box is `length` by `thickness`.
    Line {
        /// Length from cap to cap.
        length: DesignPixels,
        /// Thickness of the line.
        thickness: DesignPixels,
    },
}

impl ShapeGeometry {
    /// The geometry a new shape of `kind` starts with.
    #[must_use]
    pub const fn default_of(kind: ShapeKind) -> Self {
        match kind {
            ShapeKind::Rectangle => Self::Rectangle {
                width: DesignPixels(600.0),
                height: DesignPixels(338.0),
                corner_radius: DesignPixels(0.0),
            },
            ShapeKind::RoundedRectangle => Self::Rectangle {
                width: DesignPixels(600.0),
                height: DesignPixels(338.0),
                corner_radius: DesignPixels(32.0),
            },
            ShapeKind::Ellipse => Self::Ellipse {
                width: DesignPixels(400.0),
                height: DesignPixels(400.0),
            },
            ShapeKind::Line => Self::Line {
                length: DesignPixels(600.0),
                thickness: DesignPixels(8.0),
            },
        }
    }

    /// The extents that must be positive, with their JSON field names.
    #[must_use]
    pub const fn extents(&self) -> [(&'static str, DesignPixels); 2] {
        match *self {
            Self::Rectangle { width, height, .. } | Self::Ellipse { width, height } => {
                [("width", width), ("height", height)]
            }
            Self::Line { length, thickness } => [("length", length), ("thickness", thickness)],
        }
    }
}
