//! [`Animatable`] properties and their [`Keyframe`]s.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::Easing;
use crate::time::Flicks;

/// A property that is either constant or animated by keyframes.
///
/// Interpolation between keyframes lives in `@se/engine` (prompt 12), so preview and export share
/// it; the domain only stores and validates the keyframes. A keyframed property has at least one
/// keyframe, sorted by strictly increasing time, all within the item's duration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Animatable<T> {
    /// The same value for the whole item.
    Static {
        /// The value.
        value: T,
    },
    /// Values at given times, interpolated in between.
    Keyframed {
        /// Keyframes sorted by strictly increasing `time`.
        keyframes: Vec<Keyframe<T>>,
    },
}

impl<T> Animatable<T> {
    /// A constant property.
    pub const fn constant(value: T) -> Self {
        Self::Static { value }
    }
}

/// The value of a property at an instant, and how to reach the next keyframe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Keyframe<T> {
    /// Time relative to the item's start (`0` is its first instant, its duration its end).
    pub time: Flicks,
    /// The value at that time.
    pub value: T,
    /// Interpolation from this keyframe to the next one (ignored on the last keyframe).
    pub easing: Easing,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_tagged_by_kind() {
        let constant = Animatable::constant(0.5);
        assert_eq!(
            serde_json::to_string(&constant).unwrap(),
            r#"{"kind":"static","value":0.5}"#
        );
        let animated = Animatable::Keyframed {
            keyframes: vec![Keyframe {
                time: Flicks::ZERO,
                value: 1.0,
                easing: Easing::Linear,
            }],
        };
        let json = serde_json::to_string(&animated).unwrap();
        assert_eq!(
            json,
            r#"{"kind":"keyframed","keyframes":[{"time":0,"value":1.0,"easing":{"kind":"linear"}}]}"#
        );
        assert_eq!(
            serde_json::from_str::<Animatable<f64>>(&json).unwrap(),
            animated
        );
    }
}
