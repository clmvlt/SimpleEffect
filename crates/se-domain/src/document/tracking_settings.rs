//! [`TrackingSettings`]: how automatic reframing follows the subject of a clip (prompt 26,
//! ADR-0037). They are kept in [`crate::Reframe::Tracked`] so that « Relancer » recomputes the
//! trajectory with the same choices and the inspector shows them.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The settings of a tracked reframe. Every value has a range ([`TrackingSettings::problems`]);
/// the trajectory itself is computed by [`crate::tracking`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TrackingSettings {
    /// How smooth the frame's movement is, from 0 (follows the subject closely) to 1 (slow,
    /// steady moves).
    pub smoothing: f64,
    /// Where the subject's eyes sit in the frame, measured from its top in `[0.2, 0.8]` (0.33:
    /// the upper third line). It only matters when the frame keeps less than the picture's
    /// height (a portrait picture in a landscape frame, a zoom).
    pub eye_line: f64,
    /// Frame every face when they fit in the frame together; otherwise follow the main subject.
    pub group_framing: bool,
    /// Share of the frame, in `[0, 0.5]`, the subject may move across before the frame follows
    /// it (a camera operator's « dead zone »; 0 follows every move).
    pub dead_zone: f64,
}

impl TrackingSettings {
    /// Lowest and highest eye line.
    pub const EYE_LINE_RANGE: (f64, f64) = (0.2, 0.8);
    /// Largest dead zone.
    pub const MAX_DEAD_ZONE: f64 = 0.5;

    /// Every value outside its range, as `(field, rule)` pairs (JSON field names).
    #[must_use]
    pub fn problems(&self) -> Vec<(&'static str, &'static str)> {
        let mut problems = Vec::new();
        let within =
            |value: f64, low: f64, high: f64| value.is_finite() && (low..=high).contains(&value);
        if !within(self.smoothing, 0.0, 1.0) {
            problems.push(("smoothing", "must be in [0, 1]"));
        }
        let (low, high) = Self::EYE_LINE_RANGE;
        if !within(self.eye_line, low, high) {
            problems.push(("eyeLine", "must be in [0.2, 0.8]"));
        }
        if !within(self.dead_zone, 0.0, Self::MAX_DEAD_ZONE) {
            problems.push(("deadZone", "must be in [0, 0.5]"));
        }
        problems
    }
}

impl Default for TrackingSettings {
    /// Medium smoothing, eyes on the upper third line, groups framed together, a dead zone of
    /// 15 % of the frame (a seated speaker's sway leaves the frame still).
    fn default() -> Self {
        Self {
            smoothing: 0.5,
            eye_line: 0.33,
            group_framing: true,
            dead_zone: 0.15,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        assert_eq!(TrackingSettings::default().problems(), []);
    }

    #[test]
    fn values_outside_their_range_are_reported() {
        let settings = TrackingSettings {
            smoothing: 1.5,
            eye_line: 0.1,
            group_framing: false,
            dead_zone: f64::NAN,
        };
        assert_eq!(
            settings
                .problems()
                .into_iter()
                .map(|(field, _)| field)
                .collect::<Vec<_>>(),
            ["smoothing", "eyeLine", "deadZone"]
        );
    }

    #[test]
    fn settings_use_camel_case() {
        assert_eq!(
            serde_json::to_string(&TrackingSettings::default()).unwrap(),
            r#"{"smoothing":0.5,"eyeLine":0.33,"groupFraming":true,"deadZone":0.15}"#
        );
    }
}
