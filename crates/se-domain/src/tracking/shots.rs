//! The clip's source span cut into [`Shot`]s at shot changes (prompt 09's scene detection): the
//! frame is computed for each shot on its own, so it never drifts from one shot's subject to the
//! next one's.

use crate::{
    analysis::{SubjectDetection, SubjectFrame},
    time::Flicks,
};

/// One shot of a clip: a span of source time and the samples taken in it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Shot<'a> {
    /// First source instant of the shot (the clip's source start, or a shot change).
    pub start: Flicks,
    /// Source instant where the shot ends (excluded): the next shot change or the clip's end.
    pub end: Flicks,
    /// The samples of the shot, in time order.
    pub frames: &'a [SubjectFrame],
}

/// The shots of the source span `[start, end)`. `changes` are shot changes in source time (any
/// order; those outside the span are ignored). Samples closer than half an interval to a shot
/// change are left out: their picture may belong to the neighbouring shot (sample times are
/// rounded to the nearest frame). A shot left without samples keeps the ones it holds. The last
/// shot keeps a sample at exactly `end`.
pub(super) fn split_into_shots<'a>(
    detection: &'a SubjectDetection,
    changes: &[Flicks],
    start: Flicks,
    end: Flicks,
) -> Vec<Shot<'a>> {
    let mut cuts: Vec<Flicks> = changes
        .iter()
        .copied()
        .filter(|&change| change > start && change < end)
        .collect();
    cuts.sort_unstable();
    cuts.dedup();
    let guard = Flicks::new(detection.interval.get() / 2).unwrap_or(Flicks::ZERO);
    let mut bounds = Vec::with_capacity(cuts.len() + 2);
    bounds.push(start);
    bounds.extend(cuts);
    bounds.push(end);
    let last = bounds.len() - 2;
    let mut shots = Vec::with_capacity(last + 1);
    for (index, pair) in bounds.windows(2).enumerate() {
        let &[shot_start, shot_end] = pair else {
            continue;
        };
        let first = if index == 0 {
            shot_start
        } else {
            shot_start.checked_add(guard).unwrap_or(shot_start)
        };
        let latest = if index == last {
            shot_end
        } else {
            shot_end.checked_sub(guard).unwrap_or(shot_end)
        };
        let mut frames = detection.frames_between(first, latest);
        if frames.is_empty() {
            frames = without_end(detection.frames_between(shot_start, shot_end), shot_end);
        }
        shots.push(Shot {
            start: shot_start,
            end: shot_end,
            frames,
        });
    }
    shots
}

/// `frames` without a sample at exactly `end` (the first instant of the next shot).
fn without_end(frames: &[SubjectFrame], end: Flicks) -> &[SubjectFrame] {
    match frames.split_last() {
        Some((last, rest)) if last.time >= end => rest,
        _ => frames,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detection(count: u32) -> SubjectDetection {
        SubjectDetection {
            model: "test".to_owned(),
            interval: Flicks::from_millis(200),
            frames: (0..count)
                .map(|index| SubjectFrame {
                    time: Flicks::from_millis(index * 200),
                    faces: Vec::new(),
                })
                .collect(),
        }
    }

    fn millis(frames: &[SubjectFrame]) -> Vec<i64> {
        frames
            .iter()
            .map(|frame| frame.time.get() / 705_600)
            .collect()
    }

    #[test]
    fn one_shot_without_changes() {
        let detection = detection(20);
        let shots = split_into_shots(
            &detection,
            &[],
            Flicks::from_millis(1000),
            Flicks::from_millis(2000),
        );
        assert_eq!(shots.len(), 1);
        assert_eq!(
            millis(shots[0].frames),
            [1000, 1200, 1400, 1600, 1800, 2000]
        );
    }

    #[test]
    fn changes_split_the_span_and_samples_near_a_cut_are_left_out() {
        let detection = detection(20);
        let changes = [
            Flicks::from_millis(1500),
            Flicks::from_millis(500),
            Flicks::from_millis(3000),
        ];
        let shots = split_into_shots(
            &detection,
            &changes,
            Flicks::from_millis(1000),
            Flicks::from_millis(2600),
        );
        assert_eq!(shots.len(), 2);
        assert_eq!(shots[0].end, Flicks::from_millis(1500));
        // Half an interval (100 ms) from the cut at 1500 is far enough; closer is not.
        assert_eq!(millis(shots[0].frames), [1000, 1200, 1400]);
        assert_eq!(shots[1].start, Flicks::from_millis(1500));
        assert_eq!(
            millis(shots[1].frames),
            [1600, 1800, 2000, 2200, 2400, 2600]
        );
        let near = split_into_shots(
            &detection,
            &[Flicks::from_millis(1450)],
            Flicks::from_millis(1000),
            Flicks::from_millis(2000),
        );
        assert_eq!(millis(near[0].frames), [1000, 1200]);
        assert_eq!(millis(near[1].frames), [1600, 1800, 2000]);
    }

    #[test]
    fn a_short_shot_keeps_its_own_samples() {
        let detection = detection(20);
        let changes = [Flicks::from_millis(1100), Flicks::from_millis(1300)];
        let shots = split_into_shots(
            &detection,
            &changes,
            Flicks::from_millis(1000),
            Flicks::from_millis(2000),
        );
        assert_eq!(shots.len(), 3);
        assert_eq!(millis(shots[1].frames), [1200]);
        assert_eq!(millis(shots[0].frames), [1000]);
    }
}
