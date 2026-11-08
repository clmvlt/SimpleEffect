//! [`CaptionStyle`]: how captions look and animate (prompt 23).

use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{CaptionStyleId, Color, DesignPixels, TextCase, Vec2};

/// Characters a caption line may hold (the segmentation and the renderer's line breaking).
pub const CAPTION_LINE_CHARS: RangeInclusive<u32> = 4..=80;
/// Lines a caption may show at once.
pub const CAPTION_LINES: RangeInclusive<u32> = 1..=4;

/// A caption look shared by captions of a project (`CaptionItem::style_id`).
///
/// Sizes are design pixels (ADR-0015); `position` is the bottom centre of the caption block,
/// normalised in the frame, so blocks of one or more lines keep their baseline area.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CaptionStyle {
    /// Stable id.
    pub id: CaptionStyleId,
    /// The predefined style it was made from (`tiktokBold`, `minimal`…), kept to name it in the
    /// interface; `null` for styles made from scratch.
    pub preset: Option<String>,
    /// Font family (catalogue name, e.g. `Montserrat`).
    pub font_family: String,
    /// Font weight, 1–1000.
    pub font_weight: u16,
    /// Italic face.
    pub italic: bool,
    /// Font size in design pixels; > 0.
    pub font_size: DesignPixels,
    /// Letter case of the shown text.
    pub text_case: TextCase,
    /// Distance between baselines as a multiple of the font size, in `(0, 10]`.
    pub line_height: f64,
    /// Colour of words not spoken yet (and of every word without animation).
    pub text_color: Color,
    /// Colour of the word being spoken (and, with `highlight_spoken`, of the words spoken).
    pub active_color: Color,
    /// Colour of emphasised words (`CaptionWord::emphasis`) when they are not active.
    pub emphasis_color: Color,
    /// Outline around the glyphs.
    pub outline: CaptionOutline,
    /// Drop shadow of the glyphs.
    pub shadow: CaptionShadow,
    /// Plate behind the block (behind each line with [`CaptionAnimation::BoxPerLine`]).
    pub background: CaptionBox,
    /// Bottom centre of the block, normalised in the frame (both components in `[0, 1]`).
    pub position: Vec2,
    /// How words animate as they are spoken.
    pub animation: CaptionAnimation,
    /// Words already spoken keep the active colour (karaoke fill).
    pub highlight_spoken: bool,
    /// Longest line in characters ([`CAPTION_LINE_CHARS`]); longer words get a line of their own.
    pub max_chars_per_line: u32,
    /// Lines shown at once ([`CAPTION_LINES`]); a longer caption shows the lines holding the
    /// active word.
    pub max_lines: u32,
}

/// An outline around caption glyphs, all of it outside the glyphs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CaptionOutline {
    /// Whether it is drawn.
    pub enabled: bool,
    /// Outline colour.
    pub color: Color,
    /// Visible width in design pixels; ≥ 0.
    pub width: DesignPixels,
}

/// A drop shadow of caption glyphs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CaptionShadow {
    /// Whether it is drawn.
    pub enabled: bool,
    /// Shadow colour.
    pub color: Color,
    /// Blur radius in design pixels; ≥ 0.
    pub blur: DesignPixels,
    /// Offset in design pixels (rightward, downward).
    pub offset: Vec2,
}

/// A plate behind caption text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CaptionBox {
    /// Whether it is drawn.
    pub enabled: bool,
    /// Plate colour.
    pub color: Color,
    /// Space around the text in design pixels (x left and right, y above and below); ≥ 0.
    pub padding: Vec2,
    /// Corner radius in design pixels; ≥ 0.
    pub corner_radius: DesignPixels,
}

/// How caption words animate while spoken. The active word is the last one whose start has
/// passed; it stays active until the next word starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum CaptionAnimation {
    /// Every word in the text colour.
    #[default]
    None,
    /// The active word in the active colour.
    WordHighlight,
    /// The active word in the active colour, each word popping (a short scale bump) as it starts.
    PopPerWord,
    /// Each line on its own plate, the active word in the active colour.
    BoxPerLine,
    /// Words appear as they are spoken (the layout never moves), the active one coloured.
    RevealPerWord,
}

impl CaptionStyle {
    /// Fields holding values outside their range, as `(JSON field path, rule)`; empty when the
    /// style is valid. Validation reports them as `VALUE_OUT_OF_RANGE`.
    #[must_use]
    pub fn problems(&self) -> Vec<(&'static str, &'static str)> {
        let mut problems = Vec::new();
        let mut check = |ok: bool, field: &'static str, rule: &'static str| {
            if !ok {
                problems.push((field, rule));
            }
        };
        check(
            !self.font_family.trim().is_empty(),
            "fontFamily",
            "must not be empty",
        );
        check(
            (1..=1000).contains(&self.font_weight),
            "fontWeight",
            "must be between 1 and 1000",
        );
        check(
            positive(self.font_size.0),
            "fontSize",
            "must be a number > 0",
        );
        check(
            positive(self.line_height) && self.line_height <= 10.0,
            "lineHeight",
            "must be in (0, 10]",
        );
        check(
            non_negative(self.outline.width.0),
            "outline.width",
            "must be a number ≥ 0",
        );
        check(
            non_negative(self.shadow.blur.0),
            "shadow.blur",
            "must be a number ≥ 0",
        );
        check(
            self.shadow.offset.x.is_finite() && self.shadow.offset.y.is_finite(),
            "shadow.offset",
            "must be finite numbers",
        );
        check(
            non_negative(self.background.padding.x) && non_negative(self.background.padding.y),
            "background.padding",
            "must be numbers ≥ 0",
        );
        check(
            non_negative(self.background.corner_radius.0),
            "background.cornerRadius",
            "must be a number ≥ 0",
        );
        check(
            unit(self.position.x) && unit(self.position.y),
            "position",
            "both components must be in [0, 1]",
        );
        check(
            CAPTION_LINE_CHARS.contains(&self.max_chars_per_line),
            "maxCharsPerLine",
            "must be between 4 and 80",
        );
        check(
            CAPTION_LINES.contains(&self.max_lines),
            "maxLines",
            "must be between 1 and 4",
        );
        problems
    }
}

fn positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn non_negative(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

fn unit(value: f64) -> bool {
    (0.0..=1.0).contains(&value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caption_style_presets;

    #[test]
    fn presets_are_valid_styles() {
        let presets = caption_style_presets().unwrap();
        assert!(presets.len() >= 4);
        for preset in presets {
            assert_eq!(preset.style.problems(), [], "preset {}", preset.id);
            assert_eq!(preset.style.preset.as_deref(), Some(preset.id.as_str()));
        }
    }

    #[test]
    fn problems_name_each_faulty_field() {
        let mut style = caption_style_presets().unwrap()[0].style.clone();
        style.font_size = DesignPixels(0.0);
        style.position = Vec2::new(1.5, 0.5);
        style.max_lines = 9;
        let fields: Vec<_> = style
            .problems()
            .into_iter()
            .map(|(field, _)| field)
            .collect();
        assert_eq!(fields, ["fontSize", "position", "maxLines"]);
    }
}
