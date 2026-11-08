//! [`plan_cuts`]: candidate cut ranges (words or silences) turned into the ranges actually
//! removed — padded, kept inside the clips and the requested range, merged, snapped to frames,
//! free of slivers — sorted and apart, as `RippleDeleteRanges` needs them.

use se_domain::{Flicks, FrameRate, RoundingMode, TimeRange};

/// How cut edges move to the frame grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameSnap {
    /// To the closest frame (word cuts: their margins absorb half a frame).
    Nearest,
    /// Inwards: the start up, the end down (silence cuts never eat into sound).
    Inward,
}

/// The rules of [`plan_cuts`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CutParams {
    /// Kept at both ends of each candidate (silence padding; zero for words).
    pub padding: Flicks,
    /// Cuts shorter than this (after snapping) are dropped.
    pub min_cut: Flicks,
    /// Less than this between two cuts, or between a cut and the edge of a clip it lies in, is
    /// removed too: such slivers sound like glitches.
    pub min_keep: Flicks,
    /// The sequence's frame rate (cut edges sit on its frames).
    pub frame_rate: FrameRate,
    /// How edges reach the frame grid.
    pub snap: FrameSnap,
}

/// The cuts of `candidates` (sequence time, any order): each shrunk by `padding` on both sides,
/// clipped to `limit` and to the `clips` (the windows of the clips whose content is cut: a cut
/// never runs into time no clip covers), merged when less than `min_keep` apart, extended to a
/// clip edge closer than `min_keep`, snapped to frames, and dropped when shorter than `min_cut`
/// (before or after snapping).
/// The result is sorted, every range positive and apart from the next.
#[must_use]
pub fn plan_cuts(
    candidates: &[TimeRange],
    params: &CutParams,
    clips: &[TimeRange],
    limit: TimeRange,
) -> Vec<TimeRange> {
    let clipped: Vec<TimeRange> = candidates
        .iter()
        .filter_map(|range| padded(*range, params.padding))
        .filter_map(|range| range.intersection(limit))
        .flat_map(|range| inside_clips(range, clips))
        .collect();
    let joined = merged(clipped, params.min_keep);
    let extended: Vec<TimeRange> = joined
        .into_iter()
        .map(|range| to_near_edges(range, clips, params.min_keep, limit))
        .collect();
    let snapped = extended
        .into_iter()
        .filter(|range| range.duration() >= params.min_cut)
        .filter_map(|range| snapped(range, params, limit))
        .filter(|range| range.duration() >= params.min_cut)
        .collect();
    merged(snapped, Flicks::ZERO)
        .into_iter()
        .filter(|range| !range.is_empty())
        .collect()
}

/// `range` shrunk by `padding` at both ends; `None` when nothing is left.
fn padded(range: TimeRange, padding: Flicks) -> Option<TimeRange> {
    let start = range.start().checked_add(padding)?;
    let end = range.end().checked_sub(padding)?;
    TimeRange::from_start_end(start, end)
        .ok()
        .filter(|range| !range.is_empty())
}

/// The parts of `range` covered by `clips`.
fn inside_clips(range: TimeRange, clips: &[TimeRange]) -> Vec<TimeRange> {
    let covered = merged(clips.to_vec(), Flicks::ZERO);
    covered
        .iter()
        .filter_map(|clip| clip.intersection(range))
        .filter(|part| !part.is_empty())
        .collect()
}

/// `ranges` sorted, those overlapping or closer than `gap` joined.
fn merged(mut ranges: Vec<TimeRange>, gap: Flicks) -> Vec<TimeRange> {
    ranges.sort_by_key(|range| range.start());
    let mut joined: Vec<TimeRange> = Vec::with_capacity(ranges.len());
    for range in ranges {
        if let Some(last) = joined.last_mut() {
            let reach = last.end().checked_add(gap).unwrap_or(Flicks::MAX);
            if range.start() <= reach {
                if range.end() > last.end()
                    && let Ok(longer) = TimeRange::from_start_end(last.start(), range.end())
                {
                    *last = longer;
                }
                continue;
            }
        }
        joined.push(range);
    }
    joined
}

/// `range` reaching the edges of the clips it lies in when less than `min_keep` would be left
/// between them (inside `limit`).
fn to_near_edges(
    range: TimeRange,
    clips: &[TimeRange],
    min_keep: Flicks,
    limit: TimeRange,
) -> TimeRange {
    let (mut start, mut end) = (range.start(), range.end());
    for clip in clips.iter().filter(|clip| clip.intersects(range)) {
        let before = start.checked_sub(clip.start()).unwrap_or(Flicks::ZERO);
        if before > Flicks::ZERO && before < min_keep && clip.start() >= limit.start() {
            start = start.min(clip.start());
        }
        let after = clip.end().checked_sub(end).unwrap_or(Flicks::ZERO);
        if after > Flicks::ZERO && after < min_keep && clip.end() <= limit.end() {
            end = end.max(clip.end());
        }
    }
    TimeRange::from_start_end(start, end).unwrap_or(range)
}

/// `range` with its edges on frames, kept on the frames inside `limit`; `None` when it vanishes.
fn snapped(range: TimeRange, params: &CutParams, limit: TimeRange) -> Option<TimeRange> {
    let (start_mode, end_mode) = match params.snap {
        FrameSnap::Nearest => (RoundingMode::Nearest, RoundingMode::Nearest),
        FrameSnap::Inward => (RoundingMode::Ceil, RoundingMode::Floor),
    };
    let rate = params.frame_rate;
    let on_frame = |time: Flicks, mode| Flicks::from_frames(time.to_frames(rate, mode), rate).ok();
    let first = on_frame(limit.start(), RoundingMode::Ceil)?;
    let last = on_frame(limit.end(), RoundingMode::Floor)?;
    let start = on_frame(range.start(), start_mode)?.max(first);
    let end = on_frame(range.end(), end_mode)?.min(last);
    TimeRange::from_start_end(start, end)
        .ok()
        .filter(|range| !range.is_empty())
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    const FPS_25: FrameRate = FrameRate::FPS_25;

    fn ms(value: i64) -> Flicks {
        Flicks::new(value * Flicks::PER_SECOND / 1000).unwrap()
    }

    fn range(start: i64, end: i64) -> TimeRange {
        TimeRange::from_start_end(ms(start), ms(end)).unwrap()
    }

    fn params(snap: FrameSnap) -> CutParams {
        CutParams {
            padding: Flicks::ZERO,
            min_cut: ms(40),
            min_keep: ms(80),
            frame_rate: FPS_25,
            snap,
        }
    }

    fn spans(ranges: &[TimeRange]) -> Vec<(i64, i64)> {
        ranges
            .iter()
            .map(|range| {
                let to_ms = |time: Flicks| time.get() * 1000 / Flicks::PER_SECOND;
                (to_ms(range.start()), to_ms(range.end()))
            })
            .collect()
    }

    #[test]
    fn cut_planning_table() {
        struct Case {
            name: &'static str,
            candidates: Vec<TimeRange>,
            params: CutParams,
            clips: Vec<TimeRange>,
            limit: TimeRange,
            expected: Vec<(i64, i64)>,
        }
        let whole = range(0, 60_000);
        let one_clip = vec![range(0, 60_000)];
        let padded = CutParams {
            padding: ms(120),
            ..params(FrameSnap::Inward)
        };
        let cases = [
            Case {
                name: "close cuts merge, far ones stay apart",
                candidates: vec![range(1000, 2000), range(2040, 3000), range(5000, 6000)],
                params: params(FrameSnap::Nearest),
                clips: one_clip.clone(),
                limit: whole,
                expected: vec![(1000, 3000), (5000, 6000)],
            },
            Case {
                name: "padding shrinks silences, short ones vanish",
                candidates: vec![range(1000, 2000), range(4000, 4200)],
                params: padded,
                clips: one_clip.clone(),
                limit: whole,
                expected: vec![(1120, 1880)],
            },
            Case {
                name: "cuts shorter than the minimum are dropped",
                candidates: vec![range(1000, 1020)],
                params: params(FrameSnap::Nearest),
                clips: one_clip.clone(),
                limit: whole,
                expected: vec![],
            },
            Case {
                name: "inward snapping never eats sound",
                candidates: vec![range(1010, 1990)],
                params: params(FrameSnap::Inward),
                clips: one_clip.clone(),
                limit: whole,
                expected: vec![(1040, 1960)],
            },
            Case {
                name: "nearest snapping",
                candidates: vec![range(1010, 1990)],
                params: params(FrameSnap::Nearest),
                clips: one_clip.clone(),
                limit: whole,
                expected: vec![(1000, 2000)],
            },
            Case {
                name: "a sliver at a clip edge is removed with the cut",
                candidates: vec![range(5040, 6000), range(9000, 9960)],
                params: params(FrameSnap::Nearest),
                clips: vec![range(5000, 10_000)],
                limit: whole,
                expected: vec![(5000, 6000), (9000, 10_000)],
            },
            Case {
                name: "cuts stay inside clips and never span empty time",
                candidates: vec![range(3000, 8000)],
                params: params(FrameSnap::Nearest),
                clips: vec![range(0, 4000), range(6000, 10_000)],
                limit: whole,
                expected: vec![(3000, 4000), (6000, 8000)],
            },
            Case {
                name: "the limit clips cuts and is never extended past",
                candidates: vec![range(500, 2500)],
                params: params(FrameSnap::Nearest),
                clips: vec![range(0, 3000)],
                limit: range(1000, 2000),
                expected: vec![(1000, 2000)],
            },
        ];
        for case in cases {
            let planned = plan_cuts(&case.candidates, &case.params, &case.clips, case.limit);
            assert_eq!(spans(&planned), case.expected, "{}", case.name);
        }
    }

    fn candidate() -> impl Strategy<Value = TimeRange> {
        (0_i64..50_000, 1_i64..3000).prop_map(|(start, length)| range(start, start + length))
    }

    proptest! {
        #[test]
        fn planned_cuts_are_sorted_apart_on_frames_inside_clips_and_limit(
            candidates in prop::collection::vec(candidate(), 0..30),
            nearest in any::<bool>(),
            padding in 0_i64..300,
            limit_start in 0_i64..20_000,
        ) {
            let snap = if nearest { FrameSnap::Nearest } else { FrameSnap::Inward };
            let params = CutParams { padding: ms(padding), ..params(snap) };
            let clips = vec![range(0, 20_000), range(20_000, 35_000), range(40_000, 52_000)];
            let limit = range(limit_start, 55_000);
            let planned = plan_cuts(&candidates, &params, &clips, limit);
            for pair in planned.windows(2) {
                prop_assert!(pair[0].end() < pair[1].start());
            }
            for cut in &planned {
                prop_assert!(cut.duration() >= params.min_cut);
                prop_assert!(limit.contains_range(*cut));
                prop_assert!(clips.iter().any(|clip| clip.contains_range(*cut))
                    || (cut.start() >= ms(0) && cut.end() <= ms(35_000)));
                let frame = FPS_25.frame_duration().get();
                prop_assert_eq!(cut.start().get() % frame, 0);
                prop_assert_eq!(cut.end().get() % frame, 0);
            }
        }
    }
}
