//! [`FrameRate`]: a rational number of frames per second.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};
use ts_rs::TS;

use super::{Flicks, TimeError};

/// Frames per second as a reduced fraction, e.g. `30000/1001` for 29.97 fps.
///
/// Only rates whose frame lasts a whole number of flicks are representable (every standard rate
/// does), so frame positions convert to flicks exactly. The fraction is stored reduced: `60/2`
/// becomes `30/1`, and equal rates compare equal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FrameRate {
    numerator: u32,
    denominator: u32,
}

impl FrameRate {
    /// 23.976 fps (NTSC film).
    pub const FPS_23_976: Self = Self::exact(24_000, 1001);
    /// 24 fps (film).
    pub const FPS_24: Self = Self::exact(24, 1);
    /// 25 fps (PAL).
    pub const FPS_25: Self = Self::exact(25, 1);
    /// 29.97 fps (NTSC video).
    pub const FPS_29_97: Self = Self::exact(30_000, 1001);
    /// 30 fps.
    pub const FPS_30: Self = Self::exact(30, 1);
    /// 50 fps (PAL high frame rate).
    pub const FPS_50: Self = Self::exact(50, 1);
    /// 59.94 fps (NTSC high frame rate).
    pub const FPS_59_94: Self = Self::exact(60_000, 1001);
    /// 60 fps.
    pub const FPS_60: Self = Self::exact(60, 1);
    /// 120 fps (slow motion sources).
    pub const FPS_120: Self = Self::exact(120, 1);
    /// Every standard rate, slowest first.
    pub const STANDARD: [Self; 9] = [
        Self::FPS_23_976,
        Self::FPS_24,
        Self::FPS_25,
        Self::FPS_29_97,
        Self::FPS_30,
        Self::FPS_50,
        Self::FPS_59_94,
        Self::FPS_60,
        Self::FPS_120,
    ];

    /// Slowest supported rate, in frames per second.
    pub const MIN_FPS: u32 = 1;
    /// Fastest supported rate, in frames per second.
    pub const MAX_FPS: u32 = 1000;

    /// Builds `numerator/denominator` frames per second, reduced.
    ///
    /// # Errors
    /// [`TimeError::ZeroFrameRate`] if either part is 0; [`TimeError::FrameRateOutOfRange`]
    /// outside [`MIN_FPS`](Self::MIN_FPS)..=[`MAX_FPS`](Self::MAX_FPS) frames per second;
    /// [`TimeError::InexactFrameRate`] if one frame does not last a whole number of flicks
    /// (e.g. `47/2`).
    pub const fn new(numerator: u32, denominator: u32) -> Result<Self, TimeError> {
        if numerator == 0 || denominator == 0 {
            return Err(TimeError::ZeroFrameRate {
                numerator,
                denominator,
            });
        }
        let divisor = greatest_common_divisor(numerator, denominator);
        let reduced = Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        };
        let (frames, seconds) = (widen(reduced.numerator), widen(reduced.denominator));
        if frames < widen(Self::MIN_FPS) * seconds || frames > widen(Self::MAX_FPS) * seconds {
            return Err(TimeError::FrameRateOutOfRange {
                numerator,
                denominator,
            });
        }
        if Flicks::PER_SECOND * seconds % frames != 0 {
            return Err(TimeError::InexactFrameRate {
                numerator,
                denominator,
            });
        }
        Ok(reduced)
    }

    /// Frames counted per `denominator` seconds.
    #[must_use]
    pub const fn numerator(self) -> u32 {
        self.numerator
    }

    /// Seconds over which `numerator` frames are counted.
    #[must_use]
    pub const fn denominator(self) -> u32 {
        self.denominator
    }

    /// Exact duration of one frame (e.g. 23,543,520 flicks at 30000/1001).
    #[must_use]
    pub const fn frame_duration(self) -> Flicks {
        // Exact by construction (`new` checks divisibility) and between 1 ms and 1 s because the
        // rate is between 1 and 1000 fps, so `Flicks::new` cannot fail.
        let flicks = Flicks::PER_SECOND * widen(self.denominator) / widen(self.numerator);
        match Flicks::new(flicks) {
            Ok(duration) => duration,
            Err(_) => Flicks::one_second(),
        }
    }

    /// Whether this is an NTSC rate (x/1001: 23.976, 29.97, 59.94…).
    #[must_use]
    pub const fn is_ntsc(self) -> bool {
        self.denominator == 1001
    }

    /// Frames per timecode second: the rate rounded up (30 for 29.97, 24 for 23.976).
    ///
    /// SMPTE timecodes count frames at this nominal rate (see `parse_timecode`).
    #[must_use]
    pub const fn nominal_fps(self) -> u32 {
        self.numerator.div_ceil(self.denominator)
    }

    /// The rate as a float, for display only.
    #[must_use]
    pub fn as_f64(self) -> f64 {
        f64::from(self.numerator) / f64::from(self.denominator)
    }

    /// A rate known to be valid and reduced (used for the constants above).
    const fn exact(numerator: u32, denominator: u32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

impl fmt::Display for FrameRate {
    /// Formats the rate for messages: `25`, `29.97`, `23.976`.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denominator == 1 {
            return write!(formatter, "{}", self.numerator);
        }
        let text = format!("{:.3}", self.as_f64());
        formatter.write_str(text.trim_end_matches('0').trim_end_matches('.'))
    }
}

impl<'de> Deserialize<'de> for FrameRate {
    /// Reads `{ "numerator", "denominator" }` and applies the checks of [`FrameRate::new`].
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct RawFrameRate {
            numerator: u32,
            denominator: u32,
        }
        let raw = RawFrameRate::deserialize(deserializer)?;
        Self::new(raw.numerator, raw.denominator).map_err(serde::de::Error::custom)
    }
}

/// `u32` → `i64` in const context, where `i64::from` is not callable.
#[allow(
    clippy::cast_lossless,
    reason = "lossless widening; `From` is not const"
)]
const fn widen(value: u32) -> i64 {
    value as i64
}

const fn greatest_common_divisor(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_constants_are_valid_and_reduced() {
        for rate in FrameRate::STANDARD {
            assert_eq!(
                FrameRate::new(rate.numerator(), rate.denominator()),
                Ok(rate)
            );
        }
    }

    #[test]
    fn frame_durations_are_exact() {
        assert_eq!(FrameRate::FPS_29_97.frame_duration().get(), 23_543_520);
        assert_eq!(FrameRate::FPS_23_976.frame_duration().get(), 29_429_400);
        assert_eq!(FrameRate::FPS_25.frame_duration().get(), 28_224_000);
        assert_eq!(FrameRate::FPS_120.frame_duration().get(), 5_880_000);
    }

    #[test]
    fn fractions_are_reduced() {
        assert_eq!(FrameRate::new(60, 2), Ok(FrameRate::FPS_30));
        assert_eq!(FrameRate::new(48_000, 2002), Ok(FrameRate::FPS_23_976));
    }

    #[test]
    fn rejects_zero_and_inexact_rates() {
        assert!(matches!(
            FrameRate::new(0, 1),
            Err(TimeError::ZeroFrameRate { .. })
        ));
        assert!(matches!(
            FrameRate::new(30, 0),
            Err(TimeError::ZeroFrameRate { .. })
        ));
        assert!(matches!(
            FrameRate::new(47, 2),
            Err(TimeError::InexactFrameRate { .. })
        ));
        assert!(matches!(
            FrameRate::new(1, 2),
            Err(TimeError::FrameRateOutOfRange { .. })
        ));
        assert!(matches!(
            FrameRate::new(90_000, 1),
            Err(TimeError::FrameRateOutOfRange { .. })
        ));
    }

    #[test]
    fn ntsc_and_nominal_rates() {
        assert!(FrameRate::FPS_59_94.is_ntsc());
        assert!(!FrameRate::FPS_60.is_ntsc());
        assert_eq!(FrameRate::FPS_29_97.nominal_fps(), 30);
        assert_eq!(FrameRate::FPS_23_976.nominal_fps(), 24);
        assert_eq!(FrameRate::FPS_25.nominal_fps(), 25);
    }

    #[test]
    fn displays_like_a_human_would_write_it() {
        assert_eq!(FrameRate::FPS_25.to_string(), "25");
        assert_eq!(FrameRate::FPS_29_97.to_string(), "29.97");
        assert_eq!(FrameRate::FPS_23_976.to_string(), "23.976");
    }

    #[test]
    fn deserialization_validates_and_reduces() {
        let rate: FrameRate = serde_json::from_str(r#"{"numerator":60,"denominator":2}"#).unwrap();
        assert_eq!(rate, FrameRate::FPS_30);
        assert_eq!(
            serde_json::to_string(&rate).unwrap(),
            r#"{"numerator":30,"denominator":1}"#
        );
        assert!(serde_json::from_str::<FrameRate>(r#"{"numerator":47,"denominator":2}"#).is_err());
    }
}
