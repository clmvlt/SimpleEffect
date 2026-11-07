//! Expanding a preset into keyframes of an item ([`expand_preset`]).
//!
//! The preset's span `[start, start + duration]` sits at the start of the item (`in`), at its
//! end (`out`) or in its middle (`emphasis`) unless a start is given, and is clamped into the
//! item. For each property the preset animates:
//! - the **base** is the property's value at the end of an entrance, at the start of an exit or
//!   an emphasis, read as a hold of the existing keyframes ([`Animatable::held_value_at`]: the
//!   domain never interpolates);
//! - the property's keyframes inside the span are replaced by the preset's, those outside it are
//!   kept (a static property becomes keyframed; before and after the span it holds its first and
//!   last keyframes, so an entrance waits invisible and an exit stays gone).
//!
//! The exit plays the entrance backwards: positions `1 − at`, keyframes in reverse order and each
//! segment's easing mirrored (`ease-in` ↔ `ease-out`, Bézier control points rotated by 180°).

use super::{AnimationPhase, AnimationPreset, PresetKeyframe, PresetNumber, PresetValue};
use crate::{
    Animatable, AnimatableProperty, Color, DesignPixels, DomainError, Easing, EasingPreset, Flicks,
    ItemId, ItemKind, Keyframe, PropertyAnimation, PropertyRef, Vec2,
};

/// Where and how long a preset plays in an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresetPlacement {
    /// The phase it is applied as.
    pub phase: AnimationPhase,
    /// Its requested duration (shortened to the item's).
    pub duration: Flicks,
    /// Its start, item-relative; `None` for the phase's default place.
    pub start: Option<Flicks>,
}

/// Why a preset cannot be expanded on an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetRefusal {
    /// The preset has no such phase (an emphasis applied as an entrance...).
    PhaseNotSupported,
    /// The duration (or the item) is not positive.
    EmptyDuration,
    /// The item has no such property (a typewriter on a shape).
    MissingProperty(AnimatableProperty),
}

impl PresetRefusal {
    /// The domain error for this refusal of `preset` applied as `phase` on `item` (a
    /// `item_kind`).
    #[must_use]
    pub fn into_error(
        self,
        preset: &AnimationPreset,
        phase: AnimationPhase,
        item: ItemId,
        item_kind: &'static str,
    ) -> DomainError {
        match self {
            Self::PhaseNotSupported => DomainError::PresetPhaseNotSupported {
                preset: preset.id.clone(),
                phase,
                supported: preset
                    .phases
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
            },
            Self::EmptyDuration => DomainError::EmptyDuration {
                operation: "applyAnimationPreset",
            },
            Self::MissingProperty(property) => DomainError::PresetNotApplicable {
                item,
                item_kind,
                preset: preset.id.clone(),
                property,
            },
        }
    }
}

/// The new value of every property `preset` animates on an item of `kind` lasting
/// `item_duration`.
///
/// # Errors
/// A [`PresetRefusal`] (see [`PresetRefusal::into_error`] for the domain error).
pub fn expand_preset(
    preset: &AnimationPreset,
    placement: PresetPlacement,
    kind: &ItemKind,
    item_duration: Flicks,
) -> Result<Vec<PropertyAnimation>, PresetRefusal> {
    if !preset.supports(placement.phase) {
        return Err(PresetRefusal::PhaseNotSupported);
    }
    if placement.duration.get() <= 0 || item_duration.get() <= 0 {
        return Err(PresetRefusal::EmptyDuration);
    }
    let span = span_of(placement, item_duration);
    preset
        .tracks
        .iter()
        .map(|track| {
            let slot = kind
                .property(track.property)
                .ok_or(PresetRefusal::MissingProperty(track.property))?;
            let keyframes = phase_keyframes(&track.keyframes, placement.phase);
            Ok(animate(
                track.property,
                slot,
                &keyframes,
                span,
                placement.phase,
            ))
        })
        .collect()
}

/// The span of the animation, item-relative.
#[derive(Debug, Clone, Copy)]
struct Span {
    start: i64,
    duration: i64,
}

impl Span {
    fn end(self) -> i64 {
        self.start + self.duration
    }

    /// The flick of position `at` (0 to 1) in the span.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        reason = "spans are far below 2^53 flicks; positions are rounded to the nearest flick"
    )]
    fn time_at(self, at: f64) -> i64 {
        self.start + (at * self.duration as f64).round() as i64
    }
}

fn span_of(placement: PresetPlacement, item_duration: Flicks) -> Span {
    let total = item_duration.get();
    let duration = placement.duration.get().min(total);
    let latest = total - duration;
    let default = match placement.phase {
        AnimationPhase::In => 0,
        AnimationPhase::Out => latest,
        AnimationPhase::Emphasis => latest / 2,
    };
    let start = placement
        .start
        .map_or(default, Flicks::get)
        .clamp(0, latest);
    Span { start, duration }
}

/// The preset's keyframes for `phase`: as written, or played backwards for an exit.
fn phase_keyframes(keyframes: &[PresetKeyframe], phase: AnimationPhase) -> Vec<PresetKeyframe> {
    if phase != AnimationPhase::Out {
        return keyframes.to_vec();
    }
    let count = keyframes.len();
    (0..count)
        .map(|index| {
            let source = &keyframes[count - 1 - index];
            // The segment leaving this keyframe is the one that arrived at it, mirrored.
            let easing = if index + 1 < count {
                mirrored(keyframes[count - 2 - index].easing)
            } else {
                Easing::Linear
            };
            PresetKeyframe {
                at: 1.0 - source.at,
                value: source.value.clone(),
                easing,
            }
        })
        .collect()
}

/// The easing of a segment played backwards.
fn mirrored(easing: Easing) -> Easing {
    match easing {
        Easing::CubicBezier { x1, y1, x2, y2 } => Easing::CubicBezier {
            x1: 1.0 - x2,
            y1: 1.0 - y2,
            x2: 1.0 - x1,
            y2: 1.0 - y1,
        },
        Easing::Preset { name } => Easing::Preset {
            name: match name {
                EasingPreset::EaseIn => EasingPreset::EaseOut,
                EasingPreset::EaseOut => EasingPreset::EaseIn,
                EasingPreset::EaseInOut => EasingPreset::EaseInOut,
            },
        },
        Easing::Linear | Easing::Hold => easing,
    }
}

/// The property's new value: its keyframes outside the span plus the preset's inside it.
fn animate(
    property: AnimatableProperty,
    slot: PropertyRef<'_>,
    keyframes: &[PresetKeyframe],
    span: Span,
    phase: AnimationPhase,
) -> PropertyAnimation {
    let base_time = Flicks::new(match phase {
        AnimationPhase::In => span.end(),
        AnimationPhase::Out | AnimationPhase::Emphasis => span.start,
    })
    .unwrap_or(Flicks::ZERO);
    // Each arm reads back a value of the slot's own type, so `read` always matches.
    let read = |slot: PropertyRef<'_>| PropertyAnimation::read(property, slot);
    let animation = match slot {
        PropertyRef::Number(current) => read(PropertyRef::Number(&merge(
            current,
            keyframes,
            span,
            base_time,
            number_value,
        ))),
        PropertyRef::Pixels(current) => read(PropertyRef::Pixels(&merge(
            current,
            keyframes,
            span,
            base_time,
            pixels_value,
        ))),
        PropertyRef::Point(current) => read(PropertyRef::Point(&merge(
            current,
            keyframes,
            span,
            base_time,
            point_value,
        ))),
        PropertyRef::Color(current) => read(PropertyRef::Color(&merge(
            current,
            keyframes,
            span,
            base_time,
            color_value,
        ))),
    };
    animation.unwrap_or(PropertyAnimation::Opacity {
        value: Animatable::constant(1.0),
    })
}

/// Keeps `current`'s keyframes outside the span and adds the preset's, resolved against the base.
fn merge<T: Clone>(
    current: &Animatable<T>,
    keyframes: &[PresetKeyframe],
    span: Span,
    base_time: Flicks,
    resolve: fn(&PresetValue, &T) -> T,
) -> Animatable<T> {
    let Some(base) = current.held_value_at(base_time) else {
        return current.clone();
    };
    let kept = match current {
        Animatable::Static { .. } => Vec::new(),
        Animatable::Keyframed { keyframes } => keyframes
            .iter()
            .filter(|keyframe| keyframe.time.get() < span.start || keyframe.time.get() > span.end())
            .cloned()
            .collect(),
    };
    let mut added: Vec<Keyframe<T>> = Vec::with_capacity(keyframes.len());
    for keyframe in keyframes {
        let Ok(time) = Flicks::new(span.time_at(keyframe.at)) else {
            continue;
        };
        // A span shorter than its keyframes' spacing would put two on the same flick.
        if added.last().is_some_and(|last| last.time >= time) {
            continue;
        }
        added.push(Keyframe {
            time,
            value: resolve(&keyframe.value, &base),
            easing: keyframe.easing,
        });
    }
    let mut all: Vec<Keyframe<T>> = kept.into_iter().chain(added).collect();
    all.sort_by_key(|keyframe| keyframe.time);
    Animatable::Keyframed { keyframes: all }
}

fn scalar(number: PresetNumber) -> f64 {
    match number {
        PresetNumber::Number(value) => value,
        PresetNumber::Pair([x, _]) => x,
        PresetNumber::Color(_) => 0.0,
    }
}

fn pair(number: PresetNumber) -> Vec2 {
    match number {
        PresetNumber::Number(value) => Vec2::splat(value),
        PresetNumber::Pair([x, y]) => Vec2::new(x, y),
        PresetNumber::Color(_) => Vec2::default(),
    }
}

fn number_value(value: &PresetValue, base: &f64) -> f64 {
    match *value {
        PresetValue::Base => *base,
        PresetValue::Absolute(number) => scalar(number),
        PresetValue::Offset(number) => base + scalar(number),
        PresetValue::Factor(number) => base * scalar(number),
    }
}

fn pixels_value(value: &PresetValue, base: &DesignPixels) -> DesignPixels {
    DesignPixels(number_value(value, &base.0))
}

fn point_value(value: &PresetValue, base: &Vec2) -> Vec2 {
    match *value {
        PresetValue::Base => *base,
        PresetValue::Absolute(number) => pair(number),
        PresetValue::Offset(number) => {
            let offset = pair(number);
            Vec2::new(base.x + offset.x, base.y + offset.y)
        }
        PresetValue::Factor(number) => {
            let factor = pair(number);
            Vec2::new(base.x * factor.x, base.y * factor.y)
        }
    }
}

fn color_value(value: &PresetValue, base: &Color) -> Color {
    match value {
        PresetValue::Absolute(PresetNumber::Color(color)) => *color,
        _ => *base,
    }
}
