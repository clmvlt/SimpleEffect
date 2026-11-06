//! [`TimeRange`]: a half-open interval of time.

use serde::{Deserialize, Deserializer, Serialize};
use ts_rs::TS;

use super::{Flicks, TimeError};

/// The half-open interval `[start, start + duration)`.
///
/// Invariants, enforced at construction and deserialization: `duration ≥ 0` and the end stays
/// within the supported range. An empty range (`duration == 0`) contains and intersects nothing.
/// `start` may be negative (relative offsets); documents reject negative positions in validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TimeRange {
    start: Flicks,
    duration: Flicks,
}

impl TimeRange {
    /// Builds `[start, start + duration)`.
    ///
    /// # Errors
    /// [`TimeError::NegativeDuration`] if `duration < 0`; [`TimeError::OutOfRange`] if the end
    /// leaves the supported range.
    pub const fn new(start: Flicks, duration: Flicks) -> Result<Self, TimeError> {
        if duration.is_negative() {
            return Err(TimeError::NegativeDuration {
                duration: duration.get(),
            });
        }
        match start.checked_add(duration) {
            Some(_) => Ok(Self { start, duration }),
            None => Err(TimeError::OutOfRange),
        }
    }

    /// Builds `[start, end)`.
    ///
    /// # Errors
    /// [`TimeError::NegativeDuration`] if `end < start`; [`TimeError::OutOfRange`] if the
    /// duration leaves the supported range.
    pub const fn from_start_end(start: Flicks, end: Flicks) -> Result<Self, TimeError> {
        match end.checked_sub(start) {
            Some(duration) => Self::new(start, duration),
            None => Err(TimeError::OutOfRange),
        }
    }

    /// First instant of the range.
    #[must_use]
    pub const fn start(self) -> Flicks {
        self.start
    }

    /// Length of the range (≥ 0).
    #[must_use]
    pub const fn duration(self) -> Flicks {
        self.duration
    }

    /// First instant after the range (excluded from it).
    #[must_use]
    pub const fn end(self) -> Flicks {
        match self.start.checked_add(self.duration) {
            Some(end) => end,
            // Unreachable: construction checked that the end is in range.
            None => Flicks::MAX,
        }
    }

    /// Whether the range has no duration.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.duration.get() == 0
    }

    /// Whether `time` lies in `[start, end)`.
    #[must_use]
    pub const fn contains(self, time: Flicks) -> bool {
        self.start.get() <= time.get() && time.get() < self.end().get()
    }

    /// Whether `other` lies entirely inside this range (an empty `other` must start inside it).
    #[must_use]
    pub const fn contains_range(self, other: Self) -> bool {
        self.start.get() <= other.start.get() && other.end().get() <= self.end().get()
    }

    /// Whether both ranges share at least one instant. Ranges that only touch do not intersect.
    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        !self.is_empty()
            && !other.is_empty()
            && self.start.get() < other.end().get()
            && other.start.get() < self.end().get()
    }

    /// The instants shared by both ranges, or `None` when they do not intersect.
    #[must_use]
    pub fn intersection(self, other: Self) -> Option<Self> {
        if !self.intersects(other) {
            return None;
        }
        let start = self.start.max(other.start);
        let end = self.end().min(other.end());
        Self::from_start_end(start, end).ok()
    }

    /// The same range moved by `delta`.
    ///
    /// # Errors
    /// [`TimeError::OutOfRange`] if the moved range leaves the supported range.
    pub const fn shifted(self, delta: Flicks) -> Result<Self, TimeError> {
        match self.start.checked_add(delta) {
            Some(start) => Self::new(start, self.duration),
            None => Err(TimeError::OutOfRange),
        }
    }
}

impl<'de> Deserialize<'de> for TimeRange {
    /// Reads `{ "start", "duration" }` and applies the checks of [`TimeRange::new`].
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct RawTimeRange {
            start: Flicks,
            duration: Flicks,
        }
        let raw = RawTimeRange::deserialize(deserializer)?;
        Self::new(raw.start, raw.duration).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(start: i64, duration: i64) -> TimeRange {
        TimeRange::new(Flicks::new(start).unwrap(), Flicks::new(duration).unwrap()).unwrap()
    }

    fn at(value: i64) -> Flicks {
        Flicks::new(value).unwrap()
    }

    #[test]
    fn rejects_negative_durations_and_unrepresentable_ends() {
        assert_eq!(
            TimeRange::new(at(0), at(-1)),
            Err(TimeError::NegativeDuration { duration: -1 })
        );
        assert_eq!(
            TimeRange::new(Flicks::MAX, at(1)),
            Err(TimeError::OutOfRange)
        );
        assert_eq!(
            TimeRange::from_start_end(at(10), at(4)),
            Err(TimeError::NegativeDuration { duration: -6 })
        );
    }

    #[test]
    fn is_half_open() {
        let range = range(10, 5);
        assert_eq!(range.end(), at(15));
        assert!(range.contains(at(10)));
        assert!(range.contains(at(14)));
        assert!(!range.contains(at(15)));
        assert!(!range.contains(at(9)));
    }

    #[test]
    fn touching_ranges_do_not_intersect() {
        assert!(!range(0, 10).intersects(range(10, 5)));
        assert_eq!(range(0, 10).intersection(range(10, 5)), None);
        assert_eq!(range(0, 10).intersection(range(5, 10)), Some(range(5, 5)));
    }

    #[test]
    fn empty_ranges_intersect_nothing() {
        assert!(!range(5, 0).intersects(range(0, 10)));
        assert!(!range(5, 0).contains(at(5)));
        assert!(range(0, 10).contains_range(range(5, 0)));
    }

    #[test]
    fn shifting_keeps_the_duration() {
        assert_eq!(range(10, 5).shifted(at(-20)), Ok(range(-10, 5)));
        assert_eq!(
            range(10, 5).shifted(Flicks::MAX),
            Err(TimeError::OutOfRange)
        );
    }

    #[test]
    fn deserialization_enforces_the_invariants() {
        let parsed: TimeRange = serde_json::from_str(r#"{"start":10,"duration":5}"#).unwrap();
        assert_eq!(parsed, range(10, 5));
        let error = serde_json::from_str::<TimeRange>(r#"{"start":10,"duration":-5}"#).unwrap_err();
        assert!(error.to_string().contains("negative duration"), "{error}");
    }
}
