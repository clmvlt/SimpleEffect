//! [`Flicks`]: the integer time unit of the whole document.

use serde::{Deserialize, Deserializer, Serialize};
use ts_rs::TS;

use super::{FrameRate, RoundingMode, TimeError};

/// An instant or a duration, in flicks: 1/705,600,000 of a second (ADR-0004).
///
/// This base divides every common frame rate (NTSC x/1001 rates included) and sample rate, so
/// frame and sample positions are exact integers. Values are kept within ±[`Flicks::MAX_SAFE`],
/// which JavaScript numbers hold exactly: a `Flicks` is serialized as a plain JSON number and is a
/// branded `number` in TypeScript.
///
/// Arithmetic is checked (`checked_add`…): an overflow is a value, never a panic. Negative values
/// are allowed (offsets, deltas); the validation rejects negative positions in a document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, TS)]
#[ts(type = "number & { readonly __brand: 'Flicks' }")]
pub struct Flicks(i64);

impl Flicks {
    /// Number of flicks in one second.
    pub const PER_SECOND: i64 = 705_600_000;
    /// Largest magnitude accepted: 2^53 − 1, JavaScript's `Number.MAX_SAFE_INTEGER` (≈ 147 days).
    pub const MAX_SAFE: i64 = (1 << 53) - 1;
    /// Zero: the start of a sequence or an empty duration.
    pub const ZERO: Self = Self(0);
    /// Latest representable time.
    pub const MAX: Self = Self(Self::MAX_SAFE);
    /// Earliest representable time.
    pub const MIN: Self = Self(-Self::MAX_SAFE);

    /// Wraps a raw flick count.
    ///
    /// # Errors
    /// [`TimeError::OutOfRange`] when `|value| > MAX_SAFE`.
    pub const fn new(value: i64) -> Result<Self, TimeError> {
        if value.unsigned_abs() > Self::MAX_SAFE.unsigned_abs() {
            Err(TimeError::OutOfRange)
        } else {
            Ok(Self(value))
        }
    }

    /// One second, as a ready-made constant for callers and tests.
    #[must_use]
    pub const fn one_second() -> Self {
        Self(Self::PER_SECOND)
    }

    /// The raw flick count.
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }

    /// Whether this time is before zero.
    #[must_use]
    pub const fn is_negative(self) -> bool {
        self.0 < 0
    }

    /// The magnitude of this time; always representable since the range is symmetric.
    #[must_use]
    pub const fn abs(self) -> Self {
        Self(self.0.abs())
    }

    /// `self + rhs`, or `None` if the result leaves the supported range.
    #[must_use]
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        // Both operands are within ±2^53, so the `i64` sum cannot overflow.
        Self::within_range(self.0 + rhs.0)
    }

    /// `self - rhs`, or `None` if the result leaves the supported range.
    #[must_use]
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        Self::within_range(self.0 - rhs.0)
    }

    /// `self × factor`, or `None` if the result leaves the supported range.
    #[must_use]
    pub const fn checked_mul(self, factor: i64) -> Option<Self> {
        match self.0.checked_mul(factor) {
            Some(product) => Self::within_range(product),
            None => None,
        }
    }

    /// `self × factor` rounded to the nearest flick (halves away from zero), or `None` if
    /// `factor` is not finite or the result leaves the supported range. Used for playback speed.
    #[must_use]
    pub fn checked_mul_f64(self, factor: f64) -> Option<Self> {
        Self::from_f64_flicks(self.to_f64() * factor).ok()
    }

    /// Converts seconds to flicks, rounded to the nearest flick (halves away from zero).
    ///
    /// An `f64` has 53 bits of mantissa, so for durations of several days the input itself is
    /// only precise to about one flick. Prefer exact paths (frames, timecodes) when they exist.
    ///
    /// # Errors
    /// [`TimeError::NotFinite`] for NaN or infinities, [`TimeError::OutOfRange`] beyond
    /// ±[`Flicks::MAX_SAFE`].
    pub fn from_seconds_f64(seconds: f64) -> Result<Self, TimeError> {
        Self::from_f64_flicks(seconds * Self::PER_SECOND_F64)
    }

    /// This time in seconds. Exact as long as the result is shown with ≤ 9 decimals.
    #[must_use]
    pub fn to_seconds_f64(self) -> f64 {
        self.to_f64() / Self::PER_SECOND_F64
    }

    /// The start of frame number `frames` at `rate` (frame 0 starts at zero).
    ///
    /// # Errors
    /// [`TimeError::OutOfRange`] when the result leaves the supported range.
    pub const fn from_frames(frames: i64, rate: FrameRate) -> Result<Self, TimeError> {
        match rate.frame_duration().checked_mul(frames) {
            Some(time) => Ok(time),
            None => Err(TimeError::OutOfRange),
        }
    }

    /// The frame count at `rate` corresponding to this time, rounded with `rounding`.
    ///
    /// `RoundingMode::Floor` gives the frame displayed at this instant.
    #[must_use]
    pub const fn to_frames(self, rate: FrameRate, rounding: RoundingMode) -> i64 {
        rounding.divide(self.0, rate.frame_duration().0)
    }

    /// Flicks per second as a float; exact because it is far below 2^53.
    #[allow(
        clippy::cast_precision_loss,
        reason = "705,600,000 is exactly representable as f64"
    )]
    const PER_SECOND_F64: f64 = Self::PER_SECOND as f64;

    const fn within_range(value: i64) -> Option<Self> {
        match Self::new(value) {
            Ok(time) => Some(time),
            Err(_) => None,
        }
    }

    #[allow(
        clippy::cast_precision_loss,
        reason = "values are within ±2^53, exactly representable as f64"
    )]
    const fn to_f64(self) -> f64 {
        self.0 as f64
    }

    /// Rounds a flick count computed in floating point and checks its range.
    fn from_f64_flicks(value: f64) -> Result<Self, TimeError> {
        if !value.is_finite() {
            return Err(TimeError::NotFinite);
        }
        let rounded = value.round();
        if rounded.abs() > Self::MAX.to_f64() {
            return Err(TimeError::OutOfRange);
        }
        #[allow(
            clippy::cast_possible_truncation,
            reason = "`rounded` is an integer within ±2^53, checked above"
        )]
        Ok(Self(rounded as i64))
    }
}

impl<'de> Deserialize<'de> for Flicks {
    /// Reads a JSON integer and rejects values beyond ±[`Flicks::MAX_SAFE`].
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = i64::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flicks(value: i64) -> Flicks {
        Flicks::new(value).unwrap()
    }

    #[test]
    fn new_accepts_the_safe_range_only() {
        assert!(Flicks::new(Flicks::MAX_SAFE).is_ok());
        assert!(Flicks::new(-Flicks::MAX_SAFE).is_ok());
        assert_eq!(
            Flicks::new(Flicks::MAX_SAFE + 1),
            Err(TimeError::OutOfRange)
        );
        assert_eq!(Flicks::new(i64::MIN), Err(TimeError::OutOfRange));
    }

    #[test]
    fn checked_arithmetic_refuses_to_leave_the_range() {
        assert_eq!(flicks(2).checked_add(flicks(3)), Some(flicks(5)));
        assert_eq!(flicks(2).checked_sub(flicks(3)), Some(flicks(-1)));
        assert_eq!(Flicks::MAX.checked_add(flicks(1)), None);
        assert_eq!(Flicks::MIN.checked_sub(flicks(1)), None);
        assert_eq!(flicks(4).checked_mul(-3), Some(flicks(-12)));
        assert_eq!(Flicks::MAX.checked_mul(2), None);
        assert_eq!(flicks(i64::MAX >> 11).checked_mul(i64::MAX), None);
    }

    #[test]
    fn float_scaling_rounds_to_the_nearest_flick() {
        assert_eq!(flicks(3).checked_mul_f64(0.5), Some(flicks(2)));
        assert_eq!(flicks(-3).checked_mul_f64(0.5), Some(flicks(-2)));
        assert_eq!(flicks(10).checked_mul_f64(f64::NAN), None);
        assert_eq!(Flicks::MAX.checked_mul_f64(2.0), None);
    }

    #[test]
    fn seconds_conversions() {
        assert_eq!(Flicks::from_seconds_f64(1.5), Ok(flicks(1_058_400_000)));
        assert_eq!(
            Flicks::from_seconds_f64(f64::INFINITY),
            Err(TimeError::NotFinite)
        );
        assert_eq!(Flicks::from_seconds_f64(1e9), Err(TimeError::OutOfRange));
        assert!((flicks(1_058_400_000).to_seconds_f64() - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn serializes_as_a_plain_number_and_rejects_unsafe_numbers() {
        assert_eq!(serde_json::to_string(&flicks(42)).unwrap(), "42");
        assert_eq!(serde_json::from_str::<Flicks>("-42").unwrap(), flicks(-42));
        let error = serde_json::from_str::<Flicks>("9007199254740992").unwrap_err();
        assert!(error.to_string().contains("outside the supported range"));
    }
}
