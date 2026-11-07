//! [`Animatable`] properties and their [`Keyframe`]s.

use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned};
use ts_rs::TS;

use super::Easing;
use crate::time::Flicks;

/// A property that is either constant or animated by keyframes.
///
/// Interpolation between keyframes lives in `@se/engine` (prompt 12), so preview and export share
/// it; the domain only stores and validates the keyframes. A keyframed property has at least one
/// keyframe, sorted by strictly increasing time, all within the item's duration.
///
/// Reading also accepts a bare value (`72` for `{"kind": "static", "value": 72}`): properties that
/// became animatable (prompt 15: font size, text colour, shape fill) keep reading journals written
/// before, without a journal migration. Values are always written tagged.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
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

    /// Whether the property has keyframes.
    pub const fn is_keyframed(&self) -> bool {
        matches!(self, Self::Keyframed { .. })
    }

    /// Every value it holds: the static value, or each keyframe's value in time order.
    pub fn values(&self) -> Vec<&T> {
        match self {
            Self::Static { value } => vec![value],
            Self::Keyframed { keyframes } => {
                keyframes.iter().map(|keyframe| &keyframe.value).collect()
            }
        }
    }
}

impl<T: Clone> Animatable<T> {
    /// The value held at `time` (item-relative) when keyframes are read as holds: the static value,
    /// else the value of the last keyframe at or before `time`, else of the first keyframe. `None`
    /// only for a keyframed property without keyframes (invalid in a document).
    ///
    /// The domain never interpolates (the engine does): animation presets use this to pick the
    /// value they animate to or from.
    #[must_use]
    pub fn held_value_at(&self, time: Flicks) -> Option<T> {
        match self {
            Self::Static { value } => Some(value.clone()),
            Self::Keyframed { keyframes } => keyframes
                .iter()
                .rev()
                .find(|keyframe| keyframe.time <= time)
                .or_else(|| keyframes.first())
                .map(|keyframe| keyframe.value.clone()),
        }
    }
}

/// The tagged form, as written.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum Tagged<T> {
    Static { value: T },
    Keyframed { keyframes: Vec<Keyframe<T>> },
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for Animatable<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Animated values (numbers, `Vec2`, colours) never have a `kind` field: an object with
        // one is the tagged form, anything else a bare value.
        let raw = serde_json::Value::deserialize(deserializer)?;
        let tagged = raw
            .as_object()
            .is_some_and(|fields| fields.contains_key("kind"));
        if tagged {
            let parsed: Tagged<T> =
                serde_json::from_value(raw).map_err(serde::de::Error::custom)?;
            Ok(match parsed {
                Tagged::Static { value } => Self::Static { value },
                Tagged::Keyframed { keyframes } => Self::Keyframed { keyframes },
            })
        } else {
            serde_json::from_value(raw)
                .map(Self::constant)
                .map_err(serde::de::Error::custom)
        }
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
    use crate::{Color, Vec2, test_support::millis};

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

    #[test]
    fn reads_bare_values_as_static_ones() {
        assert_eq!(
            serde_json::from_str::<Animatable<f64>>("72").unwrap(),
            Animatable::constant(72.0)
        );
        assert_eq!(
            serde_json::from_str::<Animatable<Color>>(r##""#ffffff""##).unwrap(),
            Animatable::constant(Color::WHITE)
        );
        assert_eq!(
            serde_json::from_str::<Animatable<Vec2>>(r#"{"x":1,"y":2}"#).unwrap(),
            Animatable::constant(Vec2::new(1.0, 2.0))
        );
    }

    #[test]
    fn errors_name_the_problem() {
        let error = serde_json::from_str::<Animatable<Color>>(r#"{"kind":"static","value":"red"}"#)
            .unwrap_err();
        assert!(
            error.to_string().contains("'red' is not a colour"),
            "{error}"
        );
        let error = serde_json::from_str::<Animatable<f64>>(r#"{"kind":"bouncy"}"#).unwrap_err();
        assert!(
            error.to_string().contains("unknown variant `bouncy`"),
            "{error}"
        );
    }

    #[test]
    fn held_values_read_keyframes_as_steps() {
        let property = Animatable::Keyframed {
            keyframes: vec![
                Keyframe {
                    time: millis(500),
                    value: 1.0,
                    easing: Easing::Linear,
                },
                Keyframe {
                    time: millis(1000),
                    value: 2.0,
                    easing: Easing::Linear,
                },
            ],
        };
        assert_eq!(property.held_value_at(millis(0)), Some(1.0));
        assert_eq!(property.held_value_at(millis(999)), Some(1.0));
        assert_eq!(property.held_value_at(millis(1000)), Some(2.0));
        assert_eq!(
            Animatable::constant(3.0).held_value_at(millis(9)),
            Some(3.0)
        );
    }
}
