//! [`TextItem`]: styled text drawn by the engine.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Animatable, Stroke, Transform2D};
use crate::{color::Color, spatial::DesignPixels, spatial::Vec2};

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
    /// Share of the characters shown, from 0 (none) to 1 (all), in reading order: animating it
    /// from 0 to 1 types the text out (the « machine à écrire » preset). The layout is always
    /// that of the whole text, so revealed characters never move.
    #[serde(default = "full_reveal")]
    pub reveal: Animatable<f64>,
}

fn full_reveal() -> Animatable<f64> {
    Animatable::constant(1.0)
}

impl TextItem {
    /// Default font of new texts (bundled with the app, so always available).
    pub const DEFAULT_FONT_FAMILY: &'static str = "Inter";

    /// A new text with the default style: white Inter, 96 px, semi-bold, centred in the frame.
    #[must_use]
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            style: TextStyle::default(),
            transform: Transform2D::default(),
            reveal: full_reveal(),
        }
    }
}

/// How a text looks. Sizes are design pixels (ADR-0015); the properties that change over time
/// (size, colours, spacing, stroke, shadow, background colour) are [`Animatable`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TextStyle {
    /// Font family name, as listed by the font catalogue (bundled or system fonts). A missing
    /// family is drawn with the default font and reported.
    pub font_family: String,
    /// Font weight, 1 to 1000 (400 regular, 700 bold).
    #[serde(default = "regular_weight")]
    pub font_weight: u16,
    /// Italic (or oblique) face.
    #[serde(default)]
    pub italic: bool,
    /// Font size (the em size) in design pixels; > 0.
    pub font_size: Animatable<DesignPixels>,
    /// Colour of the glyphs.
    pub color: Animatable<Color>,
    /// Outline around the glyphs.
    #[serde(default)]
    pub stroke: Stroke,
    /// Drop shadow under the glyphs (and their outline).
    #[serde(default)]
    pub shadow: TextShadow,
    /// Plate drawn behind the text block.
    #[serde(default)]
    pub background: TextBackground,
    /// Extra space between characters, in design pixels (negative tightens).
    #[serde(default = "no_spacing")]
    pub letter_spacing: Animatable<DesignPixels>,
    /// Distance between baselines as a multiple of the font size (1.2 by default); > 0.
    #[serde(default = "default_line_height")]
    pub line_height: f64,
    /// Alignment of the lines in the text block.
    #[serde(default)]
    pub align: TextAlign,
    /// Width at which lines wrap, in design pixels; `None` keeps each line on one line.
    #[serde(default)]
    pub max_width: Option<DesignPixels>,
    /// Letter case shown (the content keeps its own case).
    #[serde(default)]
    pub text_case: TextCase,
}

const fn regular_weight() -> u16 {
    400
}

fn no_spacing() -> Animatable<DesignPixels> {
    Animatable::constant(DesignPixels(0.0))
}

const fn default_line_height() -> f64 {
    1.2
}

impl TextStyle {
    /// Lightest and heaviest weights (CSS `font-weight`).
    pub const WEIGHT_RANGE: std::ops::RangeInclusive<u16> = 1..=1000;
}

impl Default for TextStyle {
    /// White Inter, 96 px, semi-bold, centred, no outline, shadow or plate.
    fn default() -> Self {
        Self {
            font_family: TextItem::DEFAULT_FONT_FAMILY.to_owned(),
            font_weight: 600,
            italic: false,
            font_size: Animatable::constant(DesignPixels(96.0)),
            color: Animatable::constant(Color::WHITE),
            stroke: Stroke::default(),
            shadow: TextShadow::default(),
            background: TextBackground::default(),
            letter_spacing: no_spacing(),
            line_height: default_line_height(),
            align: TextAlign::default(),
            max_width: None,
            text_case: TextCase::default(),
        }
    }
}

/// A drop shadow. Kept when disabled so turning it back on restores it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TextShadow {
    /// Whether the shadow is drawn.
    pub enabled: bool,
    /// Shadow colour (its alpha is the shadow's opacity).
    pub color: Animatable<Color>,
    /// Blur radius in design pixels; ≥ 0.
    pub blur: Animatable<DesignPixels>,
    /// Shift of the shadow in design pixels (x rightward, y downward).
    pub offset: Animatable<Vec2>,
}

impl Default for TextShadow {
    /// Off; a soft black shadow 4 px below when enabled.
    fn default() -> Self {
        Self {
            enabled: false,
            color: Animatable::constant(Color {
                alpha: 0xb3,
                ..Color::BLACK
            }),
            blur: Animatable::constant(DesignPixels(8.0)),
            offset: Animatable::constant(Vec2::new(0.0, 4.0)),
        }
    }
}

/// A plate behind the text block (lower thirds, captions). Kept when disabled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TextBackground {
    /// Whether the plate is drawn. When drawn, the item's box includes it.
    pub enabled: bool,
    /// Plate colour.
    pub color: Animatable<Color>,
    /// Space between the text and the plate's edges, in design pixels (x on the left and right,
    /// y above and below); ≥ 0.
    pub padding: Vec2,
    /// Radius of the plate's corners in design pixels; ≥ 0.
    pub corner_radius: DesignPixels,
}

impl Default for TextBackground {
    /// Off; a translucent black plate with 24 × 12 px of padding when enabled.
    fn default() -> Self {
        Self {
            enabled: false,
            color: Animatable::constant(Color {
                alpha: 0xb3,
                ..Color::BLACK
            }),
            padding: Vec2::new(24.0, 12.0),
            corner_radius: DesignPixels(8.0),
        }
    }
}

/// Horizontal alignment of the lines of a text block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum TextAlign {
    /// Lines start on the left edge.
    Left,
    /// Lines are centred.
    #[default]
    Center,
    /// Lines end on the right edge.
    Right,
}

/// Letter case in which a text is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum TextCase {
    /// As typed.
    #[default]
    Normal,
    /// Every letter in capitals.
    Uppercase,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_version_4_style() {
        let style: TextStyle =
            serde_json::from_str(r##"{"fontFamily":"Inter","fontSize":72.0,"color":"#ffffff"}"##)
                .unwrap();
        assert_eq!(style.font_size, Animatable::constant(DesignPixels(72.0)));
        assert_eq!(style.color, Animatable::constant(Color::WHITE));
        assert_eq!(style.font_weight, 400);
        assert!((style.line_height - 1.2).abs() < f64::EPSILON);
        assert!(!style.stroke.enabled);
        assert_eq!(style.align, TextAlign::Center);
    }

    #[test]
    fn new_texts_use_the_bundled_font_fully_revealed() {
        let text = TextItem::new("Titre");
        assert_eq!(text.style.font_family, "Inter");
        assert_eq!(text.reveal, Animatable::constant(1.0));
        assert_eq!(text.transform, Transform2D::default());
    }
}
