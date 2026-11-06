//! Property tests of the time types: exact frame round trips at every standard rate and the
//! algebra of `TimeRange`.

// Strategies live outside #[test] functions and build values that are valid by construction.
#![allow(clippy::unwrap_used)]

use proptest::prelude::*;
use se_domain::{Flicks, FrameRate, RoundingMode, TimeRange, format_timecode, parse_timecode};

/// Any standard frame rate.
fn standard_rate() -> impl Strategy<Value = FrameRate> {
    proptest::sample::select(FrameRate::STANDARD.to_vec())
}

/// A frame number whose start is representable at `rate`, with the rate.
fn frame_at_rate() -> impl Strategy<Value = (FrameRate, i64)> {
    standard_rate().prop_flat_map(|rate| {
        let limit = Flicks::MAX_SAFE / rate.frame_duration().get();
        (Just(rate), -limit..=limit)
    })
}

/// Any representable time.
fn any_time() -> impl Strategy<Value = Flicks> {
    (-Flicks::MAX_SAFE..=Flicks::MAX_SAFE).prop_map(|value| Flicks::new(value).unwrap())
}

/// Ranges within ±1 day (~6.1e13 flicks) so their ends never overflow.
fn any_range() -> impl Strategy<Value = TimeRange> {
    const DAY: i64 = 86_400 * Flicks::PER_SECOND;
    (-DAY..DAY, 0..DAY).prop_map(|(start, duration)| {
        TimeRange::new(Flicks::new(start).unwrap(), Flicks::new(duration).unwrap()).unwrap()
    })
}

proptest! {
    #[test]
    fn frames_round_trip_through_flicks_with_every_rounding((rate, frames) in frame_at_rate()) {
        let time = Flicks::from_frames(frames, rate).unwrap();
        for rounding in [RoundingMode::Floor, RoundingMode::Ceil, RoundingMode::Nearest] {
            prop_assert_eq!(time.to_frames(rate, rounding), frames);
        }
    }

    #[test]
    fn floor_finds_the_frame_containing_a_time(rate in standard_rate(), time in any_time()) {
        let frame = time.to_frames(rate, RoundingMode::Floor);
        let frame_start = Flicks::from_frames(frame, rate).unwrap().get();
        prop_assert!(frame_start <= time.get());
        prop_assert!(time.get() < frame_start + rate.frame_duration().get());
    }

    #[test]
    fn smpte_timecodes_round_trip((rate, frames) in frame_at_rate()) {
        let time = Flicks::from_frames(frames.abs(), rate).unwrap();
        prop_assert_eq!(parse_timecode(&format_timecode(time, rate), Some(rate)), Ok(time));
    }

    #[test]
    fn intersection_is_commutative(a in any_range(), b in any_range()) {
        prop_assert_eq!(a.intersection(b), b.intersection(a));
    }

    #[test]
    fn intersection_lies_in_both_ranges(a in any_range(), b in any_range()) {
        prop_assert_eq!(a.intersects(b), a.intersection(b).is_some());
        if let Some(shared) = a.intersection(b) {
            prop_assert!(!shared.is_empty());
            prop_assert!(a.contains_range(shared));
            prop_assert!(b.contains_range(shared));
        }
    }

    #[test]
    fn shifting_moves_both_ends(range in any_range(), delta in -1_000_000_000_i64..1_000_000_000) {
        let delta = Flicks::new(delta).unwrap();
        let moved = range.shifted(delta).unwrap();
        prop_assert_eq!(moved.duration(), range.duration());
        prop_assert_eq!(moved.end().get() - range.end().get(), delta.get());
    }
}
