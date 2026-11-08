//! [`ClipWindow`]: where a clip plays its source, to convert source time ↔ sequence time.

use crate::{Flicks, Item, MediaClip, TimeRange};

/// Where a clip plays its source: its sequence range and the matching source range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipWindow {
    /// The item's range in the sequence.
    pub timeline: TimeRange,
    /// The part of the asset it plays.
    pub source: TimeRange,
    /// Source seconds per sequence second (> 0).
    pub speed: f64,
}

impl ClipWindow {
    /// The window of `clip` on `item`; `None` for an unusable speed.
    #[must_use]
    pub fn of(item: &Item, clip: &MediaClip) -> Option<Self> {
        let source = clip.source_range(item.timeline_range.duration())?;
        Some(Self {
            timeline: item.timeline_range,
            source,
            speed: clip.speed,
        })
    }

    /// The sequence time at which source time `time` plays (may lie outside the item).
    #[must_use]
    pub fn to_timeline(&self, time: Flicks) -> Option<Flicks> {
        let offset = time.checked_sub(self.source.start())?;
        let scaled = offset.checked_mul_f64(1.0 / self.speed)?;
        self.timeline.start().checked_add(scaled)
    }

    /// The source time playing at sequence time `time` (may lie outside the source window).
    #[must_use]
    pub fn to_source(&self, time: Flicks) -> Option<Flicks> {
        let offset = time.checked_sub(self.timeline.start())?;
        let scaled = offset.checked_mul_f64(self.speed)?;
        self.source.start().checked_add(scaled)
    }

    /// The part of source range `range` this clip plays, in sequence time, within `limit`.
    #[must_use]
    pub fn map_range(&self, range: TimeRange, limit: TimeRange) -> Option<TimeRange> {
        let played = range.intersection(self.source)?;
        let start = self.to_timeline(played.start())?;
        let end = self.to_timeline(played.end())?;
        let mapped = TimeRange::from_start_end(start, end.min(self.timeline.end())).ok()?;
        mapped.intersection(limit).filter(|range| !range.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::millis;

    fn range(start_ms: i64, end_ms: i64) -> TimeRange {
        TimeRange::from_start_end(millis(start_ms), millis(end_ms)).unwrap()
    }

    fn window(speed: f64) -> ClipWindow {
        // Plays source 10 s.. at sequence 2 s for 4 s.
        let timeline = range(2_000, 6_000);
        let source_end = if speed > 1.5 { 18_000 } else { 14_000 };
        ClipWindow {
            timeline,
            source: range(10_000, source_end),
            speed,
        }
    }

    #[test]
    fn source_ranges_map_into_the_sequence_through_the_speed() {
        let whole = range(0, 60_000);
        assert_eq!(
            window(1.0).map_range(range(11_000, 12_000), whole),
            Some(range(3_000, 4_000))
        );
        assert_eq!(
            window(2.0).map_range(range(12_000, 16_000), whole),
            Some(range(3_000, 5_000))
        );
        assert_eq!(
            window(1.0).map_range(range(5_000, 10_500), whole),
            Some(range(2_000, 2_500))
        );
        assert_eq!(window(1.0).map_range(range(20_000, 21_000), whole), None);
        assert_eq!(
            window(1.0).map_range(range(11_000, 13_000), range(0, 3_500)),
            Some(range(3_000, 3_500))
        );
    }

    #[test]
    fn source_and_sequence_times_convert_both_ways() {
        let fast = window(2.0);
        assert_eq!(fast.to_timeline(millis(14_000)), Some(millis(4_000)));
        assert_eq!(fast.to_source(millis(4_000)), Some(millis(14_000)));
        assert_eq!(window(1.0).to_source(millis(1_000)), Some(millis(9_000)));
    }
}
