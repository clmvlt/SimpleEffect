//! The data of the preset library (`assets/animation-presets.json`) and its consistency checks.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{AnimatableProperty, Color, Easing, Flicks};

/// When an animation plays in its item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum AnimationPhase {
    /// An entrance, at the start of the item (or at the given start).
    In,
    /// An exit, ending with the item (or at the given start).
    Out,
    /// An emphasis inside the item (centred by default), returning to the item's values.
    Emphasis,
}

impl std::fmt::Display for AnimationPhase {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::In => "in",
            Self::Out => "out",
            Self::Emphasis => "emphasis",
        })
    }
}

/// Every preset, as read from the library file.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationPresetLibrary {
    /// Format version of the file.
    pub version: u32,
    /// The presets, in display order.
    pub presets: Vec<AnimationPreset>,
}

/// A named animation: keyframes relative to the item's own values, written for its entrance (or
/// its emphasis); the exit plays the entrance backwards.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnimationPreset {
    /// Stable id (`fade`, `slideFromLeft`…), used by operations, agents and translations.
    pub id: String,
    /// English name, for agents.
    pub name: String,
    /// What it does, for agents.
    pub description: String,
    /// Phases it can be applied as: `in` and `out` together (the exit is derived), or `emphasis`.
    pub phases: Vec<AnimationPhase>,
    /// Suggested duration in seconds.
    pub default_duration_seconds: f64,
    /// One track of keyframes per animated property.
    pub tracks: Vec<PresetTrack>,
}

/// The keyframes a preset gives one property.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PresetTrack {
    /// The property animated.
    pub property: AnimatableProperty,
    /// Keyframes sorted by strictly increasing `at`.
    pub keyframes: Vec<PresetKeyframe>,
}

/// A keyframe of a preset, placed in the animation's span.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PresetKeyframe {
    /// Position in the animation's span, from 0 (its start) to 1 (its end).
    pub at: f64,
    /// The value, relative to the item's own value.
    pub value: PresetValue,
    /// Interpolation to the next keyframe (linear when absent).
    #[serde(default)]
    pub easing: Easing,
}

/// A preset keyframe's value, relative to the property's value in the item (its *base*).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PresetValue {
    /// The item's own value.
    Base,
    /// This value, whatever the item's.
    Absolute(PresetNumber),
    /// The item's value plus this (numbers and points).
    Offset(PresetNumber),
    /// The item's value times this (numbers and points, per component).
    Factor(PresetNumber),
}

/// A number, a pair (`[x, y]`, a single number meaning both) or a colour.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum PresetNumber {
    /// A number (both components for points).
    Number(f64),
    /// A pair of numbers.
    Pair([f64; 2]),
    /// A colour (absolute colour values only).
    Color(Color),
}

impl AnimationPreset {
    /// The suggested duration in flicks.
    #[must_use]
    pub fn default_duration(&self) -> Flicks {
        Flicks::from_seconds_f64(self.default_duration_seconds)
            .unwrap_or_else(|_| Flicks::one_second())
    }

    /// Whether it can be applied as `phase`.
    #[must_use]
    pub fn supports(&self, phase: AnimationPhase) -> bool {
        self.phases.contains(&phase)
    }

    /// The properties it animates.
    #[must_use]
    pub fn properties(&self) -> Vec<AnimatableProperty> {
        self.tracks.iter().map(|track| track.property).collect()
    }
}

impl AnimationPresetLibrary {
    /// The preset with this id.
    #[must_use]
    pub fn find(&self, id: &str) -> Option<&AnimationPreset> {
        self.presets.iter().find(|preset| preset.id == id)
    }

    /// The ids of every preset, comma-separated (for messages).
    #[must_use]
    pub fn ids(&self) -> String {
        self.presets
            .iter()
            .map(|preset| preset.id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Checks what serde cannot: unique ids, phases that make sense, positive durations, sorted
    /// keyframes in `[0, 1]` and values of the property's type.
    ///
    /// # Errors
    /// The first problem found, naming the preset.
    pub fn check(&self) -> Result<(), String> {
        let mut ids = HashSet::new();
        for preset in &self.presets {
            let id = &preset.id;
            if !ids.insert(id.as_str()) {
                return Err(format!("preset '{id}' is listed twice"));
            }
            check_phases(preset)?;
            if !(preset.default_duration_seconds > 0.0 && preset.default_duration_seconds <= 60.0) {
                return Err(format!(
                    "preset '{id}' needs a default duration of 0 to 60 s"
                ));
            }
            if preset.tracks.is_empty() {
                return Err(format!("preset '{id}' animates nothing"));
            }
            for track in &preset.tracks {
                check_track(track).map_err(|problem| format!("preset '{id}': {problem}"))?;
            }
        }
        Ok(())
    }
}

fn check_phases(preset: &AnimationPreset) -> Result<(), String> {
    let phases = &preset.phases;
    let entrance = phases.len() == 2
        && phases.contains(&AnimationPhase::In)
        && phases.contains(&AnimationPhase::Out);
    let emphasis = phases == &[AnimationPhase::Emphasis];
    if entrance || emphasis {
        Ok(())
    } else {
        Err(format!(
            "preset '{}' must have the phases [in, out] or [emphasis]",
            preset.id
        ))
    }
}

fn check_track(track: &PresetTrack) -> Result<(), String> {
    let property = track.property;
    if track.keyframes.len() < 2 {
        return Err(format!("{property} needs at least two keyframes"));
    }
    let mut previous: Option<f64> = None;
    for keyframe in &track.keyframes {
        let at = keyframe.at;
        if !(0.0..=1.0).contains(&at) || previous.is_some_and(|previous| at <= previous) {
            return Err(format!(
                "{property} keyframe at {at}: positions must strictly increase within [0, 1]"
            ));
        }
        previous = Some(at);
        if !value_fits(property, &keyframe.value) {
            return Err(format!(
                "{property} keyframe at {at}: {:?} does not fit the property",
                keyframe.value
            ));
        }
    }
    Ok(())
}

/// Whether `value` makes sense for `property` (colours only absolute colours, numbers numbers,
/// points numbers or pairs).
fn value_fits(property: AnimatableProperty, value: &PresetValue) -> bool {
    use AnimatableProperty as P;
    let number = match value {
        PresetValue::Base => return true,
        PresetValue::Absolute(number)
        | PresetValue::Offset(number)
        | PresetValue::Factor(number) => *number,
    };
    match property {
        P::FillColor | P::StrokeColor | P::ShadowColor | P::BackgroundColor => {
            matches!(
                (value, number),
                (PresetValue::Absolute(_), PresetNumber::Color(_))
            )
        }
        P::Position | P::Scale | P::Anchor | P::ShadowOffset => {
            !matches!(number, PresetNumber::Color(_))
        }
        _ => matches!(number, PresetNumber::Number(_)),
    }
}
