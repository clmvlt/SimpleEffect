//! Input pieces shared by write tools: positions, colours, outlines, animations, keyframe values
//! and easings, written the way agents think (named places, `#rrggbb`, seconds) and converted to
//! the document's types.

use schemars::JsonSchema;
use se_app::agent_edits::PresetSpec;
use se_domain::{AnimationPhase, Color, Easing, EasingPreset, TimeRange, Vec2, format_clock};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    output::{ToolError, codes},
    time_io::TimeInput,
    writes::WriteScene,
};

/// A point of the frame, normalised: `x` from 0 (left) to 1 (right), `y` from 0 (top) to 1
/// (bottom).
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PointInput {
    /// From 0 (left edge) to 1 (right edge).
    pub x: f64,
    /// From 0 (top edge) to 1 (bottom edge).
    pub y: f64,
}

/// A named place of the frame (inside the usual safe areas of vertical and horizontal videos).
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NamedPosition {
    /// The centre (0.5, 0.5).
    Center,
    /// Upper part, centred (0.5, 0.15).
    Top,
    /// Lower part, centred (0.5, 0.8).
    Bottom,
    /// Left part, centred vertically (0.2, 0.5).
    Left,
    /// Right part, centred vertically (0.8, 0.5).
    Right,
    /// Lower third, left (0.3, 0.75): names and titles.
    LowerThird,
}

/// Where an item's centre (its anchor) sits.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum PositionInput {
    /// A named place.
    Named(NamedPosition),
    /// A normalised point.
    Point(PointInput),
}

impl PositionInput {
    /// The normalised position.
    #[must_use]
    pub const fn to_vec2(self) -> Vec2 {
        match self {
            Self::Point(point) => Vec2::new(point.x, point.y),
            Self::Named(NamedPosition::Center) => Vec2::new(0.5, 0.5),
            Self::Named(NamedPosition::Top) => Vec2::new(0.5, 0.15),
            Self::Named(NamedPosition::Bottom) => Vec2::new(0.5, 0.8),
            Self::Named(NamedPosition::Left) => Vec2::new(0.2, 0.5),
            Self::Named(NamedPosition::Right) => Vec2::new(0.8, 0.5),
            Self::Named(NamedPosition::LowerThird) => Vec2::new(0.3, 0.75),
        }
    }
}

/// An outline around text glyphs or a shape.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StrokeInput {
    /// `#rrggbb` or `#rrggbbaa`; default black.
    pub color: Option<String>,
    /// Width in design pixels (1080 px = the frame's shorter side); default 4.
    pub width: Option<f64>,
}

/// An animation applied to a new item.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnimationInput {
    /// A preset id from `list_animation_presets` (`fade`, `pop`, `slideFromLeft`…).
    pub preset: String,
    /// Its length (seconds or timecode); default: the preset's.
    pub duration: Option<TimeInput>,
}

impl AnimationInput {
    /// The preset as a se-app spec for `phase`.
    ///
    /// # Errors
    /// [`codes::INVALID_TIME`] for an unreadable or non-positive duration.
    pub fn spec(
        &self,
        scene: &WriteScene<'_>,
        phase: AnimationPhase,
    ) -> Result<PresetSpec, ToolError> {
        Ok(PresetSpec {
            preset: self.preset.clone(),
            phase,
            duration: self
                .duration
                .as_ref()
                .map(|duration| scene.duration(duration, "the animation"))
                .transpose()?,
        })
    }
}

/// The entrance and exit presets of a new item, in order.
///
/// # Errors
/// [`codes::INVALID_TIME`].
pub fn presets(
    scene: &WriteScene<'_>,
    entrance: Option<&AnimationInput>,
    exit: Option<&AnimationInput>,
) -> Result<Vec<PresetSpec>, ToolError> {
    let mut specs = Vec::new();
    if let Some(entrance) = entrance {
        specs.push(entrance.spec(scene, AnimationPhase::In)?);
    }
    if let Some(exit) = exit {
        specs.push(exit.spec(scene, AnimationPhase::Out)?);
    }
    Ok(specs)
}

/// `text` as a colour.
///
/// # Errors
/// [`codes::INVALID_ARGUMENTS`] naming `field`.
pub fn color(field: &str, text: &str) -> Result<Color, ToolError> {
    Color::parse(text).map_err(|error| {
        ToolError::invalid_arguments(format!(
            "{field}: {error}; write colours as \"#rrggbb\" or \"#rrggbbaa\""
        ))
    })
}

/// The range from `start` to `end`, or of length `duration` (exactly one of the two).
///
/// # Errors
/// [`codes::INVALID_ARGUMENTS`] when both or neither are given; [`codes::INVALID_TIME`].
pub fn span(
    scene: &WriteScene<'_>,
    start: &TimeInput,
    end: Option<&TimeInput>,
    duration: Option<&TimeInput>,
) -> Result<TimeRange, ToolError> {
    let start = scene.time(start)?;
    let end = match (end, duration) {
        (Some(end), None) => scene.time(end)?,
        (None, Some(duration)) => start
            .checked_add(scene.duration(duration, "duration")?)
            .ok_or_else(|| ToolError::new(codes::INVALID_TIME, "the end is too far"))?,
        _ => {
            return Err(ToolError::invalid_arguments(
                "give either `end` or `duration`, not both",
            ));
        }
    };
    TimeRange::from_start_end(start, end)
        .ok()
        .filter(|range| !range.is_empty() && !start.is_negative())
        .ok_or_else(|| {
            ToolError::new(
                codes::INVALID_TIME,
                format!(
                    "the item must start at 0 or later and end after it starts (start {}, end {})",
                    format_clock(start),
                    format_clock(end)
                ),
            )
        })
}

/// A keyframe value: a number (opacity, rotation, sizes, dB), a pair (position, scale, anchor,
/// focus, shadow offset) or a colour.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ValueInput {
    /// A number.
    Number(f64),
    /// A pair `[x, y]`.
    Pair([f64; 2]),
    /// A pair `{"x", "y"}`.
    Point(PointInput),
    /// A colour `#rrggbb[aa]`.
    Text(String),
}

impl ValueInput {
    /// The value as the document writes it (pairs as `{"x", "y"}`).
    #[must_use]
    pub fn to_json(&self) -> Value {
        match self {
            Self::Number(number) => json!(number),
            Self::Pair([x, y]) => json!({ "x": x, "y": y }),
            Self::Point(point) => json!({ "x": point.x, "y": point.y }),
            Self::Text(text) => json!(text),
        }
    }
}

/// A named easing.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EasingName {
    /// Constant speed.
    Linear,
    /// Jumps to the next value at the next keyframe.
    Hold,
    /// Starts slowly.
    EaseIn,
    /// Ends slowly (natural for entrances).
    EaseOut,
    /// Starts and ends slowly.
    EaseInOut,
}

/// How a value travels to the next keyframe.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum EasingInput {
    /// A named easing.
    Named(EasingName),
    /// A cubic Bézier `[x1, y1, x2, y2]` (CSS-like).
    Bezier([f64; 4]),
}

impl EasingInput {
    /// The document's easing.
    #[must_use]
    pub const fn to_easing(self) -> Easing {
        match self {
            Self::Named(EasingName::Linear) => Easing::Linear,
            Self::Named(EasingName::Hold) => Easing::Hold,
            Self::Named(EasingName::EaseIn) => Easing::Preset {
                name: EasingPreset::EaseIn,
            },
            Self::Named(EasingName::EaseOut) => Easing::Preset {
                name: EasingPreset::EaseOut,
            },
            Self::Named(EasingName::EaseInOut) => Easing::Preset {
                name: EasingPreset::EaseInOut,
            },
            Self::Bezier([x1, y1, x2, y2]) => Easing::CubicBezier { x1, y1, x2, y2 },
        }
    }
}
