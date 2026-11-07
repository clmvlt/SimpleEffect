//! [`Stroke`]: an outline around text glyphs or a shape.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::Animatable;
use crate::{color::Color, spatial::DesignPixels};

/// An outline. Kept when disabled so turning it back on restores it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Stroke {
    /// Whether the outline is drawn.
    pub enabled: bool,
    /// Outline colour.
    pub color: Animatable<Color>,
    /// Visible width of the outline outside the glyphs or the shape's edge, in design pixels;
    /// ≥ 0.
    pub width: Animatable<DesignPixels>,
}

impl Default for Stroke {
    /// Off; a black 4 px outline when enabled.
    fn default() -> Self {
        Self {
            enabled: false,
            color: Animatable::constant(Color::BLACK),
            width: Animatable::constant(DesignPixels(4.0)),
        }
    }
}
