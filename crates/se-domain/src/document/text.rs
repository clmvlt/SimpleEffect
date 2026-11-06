//! [`TextItem`]: styled text drawn by the engine.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::Transform2D;
use crate::{color::Color, spatial::DesignPixels};

/// A block of text. Editable data, never a rendered picture (golden rule 6).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TextItem {
    /// The text, lines separated by `\n`.
    pub content: String,
    /// How the text looks.
    pub style: TextStyle,
    /// Placement of the text box.
    pub transform: Transform2D,
}

/// Minimal text style.
///
/// TODO(prompt-15): weight, alignment, line height, stroke, shadow, background and the text
/// animations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TextStyle {
    /// Font family name (a bundled or brand-kit font).
    pub font_family: String,
    /// Font size in design pixels (ADR-0015).
    pub font_size: DesignPixels,
    /// Fill colour of the glyphs.
    pub color: Color,
}
