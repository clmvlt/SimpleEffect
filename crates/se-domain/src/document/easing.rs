//! [`Easing`]: how a keyframed value travels to the next keyframe.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Interpolation curve from one keyframe to the next.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Easing {
    /// Constant speed.
    #[default]
    Linear,
    /// Keep this keyframe's value until the next keyframe, then jump.
    Hold,
    /// A CSS-style cubic Bézier curve from `(0, 0)` to `(1, 1)`; `x1` and `x2` must be in
    /// `[0, 1]` so time keeps moving forward, `y1` and `y2` may overshoot.
    CubicBezier {
        /// Time of the first control point.
        x1: f64,
        /// Progress of the first control point.
        y1: f64,
        /// Time of the second control point.
        x2: f64,
        /// Progress of the second control point.
        y2: f64,
    },
    /// A named curve from the preset library.
    Preset {
        /// Which preset.
        name: EasingPreset,
    },
}

/// Named easing curves. The engine (prompt 12) owns their exact shape.
///
/// TODO(prompt-15): add the motion presets (back, elastic, bounce) with the text animations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum EasingPreset {
    /// Starts slowly, ends fast.
    EaseIn,
    /// Starts fast, ends slowly.
    EaseOut,
    /// Slow at both ends.
    EaseInOut,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_tagged_by_kind() {
        let curve = Easing::CubicBezier {
            x1: 0.25,
            y1: 0.1,
            x2: 0.25,
            y2: 1.0,
        };
        assert_eq!(
            serde_json::to_string(&curve).unwrap(),
            r#"{"kind":"cubicBezier","x1":0.25,"y1":0.1,"x2":0.25,"y2":1.0}"#
        );
        let preset = Easing::Preset {
            name: EasingPreset::EaseInOut,
        };
        assert_eq!(
            serde_json::to_string(&preset).unwrap(),
            r#"{"kind":"preset","name":"easeInOut"}"#
        );
    }
}
