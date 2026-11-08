//! Sets of time ranges and instants spread over a range. Source time ↔ sequence time of clips is
//! [`se_domain::ClipWindow`] (re-exported here): analyses are stored per asset in source time;
//! tools bring them back to the sequence when asked about a range of it.

pub use se_domain::ClipWindow;
use se_domain::{Flicks, TimeRange};

/// `ranges` sorted and merged where they touch or overlap.
#[must_use]
pub fn merge(mut ranges: Vec<TimeRange>) -> Vec<TimeRange> {
    ranges.retain(|range| !range.is_empty());
    ranges.sort_by_key(|range| range.start());
    let mut merged: Vec<TimeRange> = Vec::with_capacity(ranges.len());
    for range in ranges {
        match merged.last_mut() {
            Some(last) if range.start() <= last.end() => {
                if range.end() > last.end()
                    && let Ok(joined) = TimeRange::from_start_end(last.start(), range.end())
                {
                    *last = joined;
                }
            }
            _ => merged.push(range),
        }
    }
    merged
}

/// The parts of `within` covered by none of `ranges`.
#[must_use]
pub fn complement(ranges: Vec<TimeRange>, within: TimeRange) -> Vec<TimeRange> {
    let mut gaps = Vec::new();
    let mut cursor = within.start();
    for range in merge(ranges) {
        let Some(range) = range.intersection(within) else {
            continue;
        };
        if range.start() > cursor
            && let Ok(gap) = TimeRange::from_start_end(cursor, range.start())
        {
            gaps.push(gap);
        }
        cursor = cursor.max(range.end());
    }
    if cursor < within.end()
        && let Ok(gap) = TimeRange::from_start_end(cursor, within.end())
    {
        gaps.push(gap);
    }
    gaps
}

/// `count` instants spread over `range`: the middle of each of `count` equal slices (never the
/// excluded end).
#[must_use]
pub fn spread(range: TimeRange, count: usize) -> Vec<Flicks> {
    let Ok(count_i64) = i64::try_from(count) else {
        return Vec::new();
    };
    if count_i64 == 0 {
        return Vec::new();
    }
    let duration = i128::from(range.duration().get());
    (0..count_i64)
        .filter_map(|index| {
            let offset = duration * i128::from(2 * index + 1) / i128::from(2 * count_i64);
            let offset = Flicks::new(i64::try_from(offset).ok()?).ok()?;
            range.start().checked_add(offset)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use se_domain::test_support::millis;

    use super::*;

    fn range(start_ms: i64, end_ms: i64) -> TimeRange {
        TimeRange::from_start_end(millis(start_ms), millis(end_ms)).unwrap()
    }

    #[test]
    fn complements_fill_the_gaps_between_merged_ranges() {
        let within = range(0, 10_000);
        let covered = vec![
            range(6_000, 8_000),
            range(1_000, 3_000),
            range(2_500, 4_000),
        ];
        assert_eq!(
            merge(covered.clone()),
            [range(1_000, 4_000), range(6_000, 8_000)]
        );
        assert_eq!(
            complement(covered, within),
            [range(0, 1_000), range(4_000, 6_000), range(8_000, 10_000)]
        );
        assert_eq!(complement(Vec::new(), within), [within]);
        assert_eq!(
            complement(vec![range(0, 12_000)], within),
            Vec::<TimeRange>::new()
        );
    }

    #[test]
    fn spread_instants_sit_in_the_middle_of_equal_slices() {
        assert_eq!(
            spread(range(0, 10_000), 5),
            [
                millis(1_000),
                millis(3_000),
                millis(5_000),
                millis(7_000),
                millis(9_000)
            ]
        );
        assert_eq!(spread(range(2_000, 4_000), 1), [millis(3_000)]);
        assert_eq!(spread(range(0, 1_000), 0), Vec::<Flicks>::new());
    }
}
