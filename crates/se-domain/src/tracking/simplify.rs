//! Simplification of a smoothed path into few keyframes (Ramer–Douglas–Peucker over curved
//! segments): a point is kept when the segment between its neighbours kept would move the frame
//! by more than a tolerance. Segments are cubic: each keyframe's easing is a Bézier curve with
//! `x1 = 1/3`, `x2 = 2/3` — then the curve's progress is a cubic Hermite polynomial of time — whose
//! end slopes follow the smoothed path, so a smooth pan needs two or three keyframes instead of
//! a dozen straight pieces. Slopes are limited to `[0, 3]` (Fritsch–Carlson): a segment never
//! overshoots its two values, so the focus stays inside the picture.

use super::{path::PathPoint, window::CropWindow};
use crate::{document::Easing, spatial::Vec2};

/// Largest move of the frame allowed between the smoothed path and its keyframes, as a share of
/// the frame: 0.8 % is about 9 pixels of a 1080-pixel-wide frame.
pub(super) const TOLERANCE: f64 = 0.008;

/// Below this move (a share of the frame), a segment is straight: it barely moves.
const STILL: f64 = 0.002;

/// Largest end slope of a segment, relative to its average slope (monotonic cubic).
const MAX_SLOPE: f64 = 3.0;

/// A point kept as a keyframe, with the easing of the segment that leaves it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct KeptPoint {
    /// Seconds since the item's start.
    pub time: f64,
    /// The focus.
    pub focus: Vec2,
    /// Interpolation towards the next kept point.
    pub easing: Easing,
}

/// The points of `path` to keep (its first and last always), each with the easing towards the
/// next: the segments between them never differ from a dropped point by more than `tolerance`
/// ([`CropWindow::distance`]).
pub(super) fn simplified(path: &[PathPoint], window: CropWindow, tolerance: f64) -> Vec<KeptPoint> {
    let Some(last) = path.len().checked_sub(1) else {
        return Vec::new();
    };
    let slopes = slopes(path);
    let mut keep = vec![false; path.len()];
    if let Some(first) = keep.first_mut() {
        *first = true;
    }
    if let Some(end) = keep.last_mut() {
        *end = true;
    }
    let mut spans = vec![(0, last)];
    while let Some((start, end)) = spans.pop() {
        if end <= start + 1 {
            continue;
        }
        let segment = Segment::between(path, &slopes, start, end, window);
        let farthest = (start + 1..end)
            .filter_map(|index| {
                let point = path.get(index)?;
                Some((index, window.distance(point.focus, segment.at(point.time))))
            })
            .max_by(|x, y| x.1.total_cmp(&y.1));
        if let Some((index, distance)) = farthest
            && distance > tolerance
        {
            if let Some(flag) = keep.get_mut(index) {
                *flag = true;
            }
            spans.push((start, index));
            spans.push((index, end));
        }
    }
    let kept: Vec<usize> = (0..path.len()).filter(|&index| keep[index]).collect();
    kept.iter()
        .enumerate()
        .filter_map(|(position, &index)| {
            let point = path.get(index)?;
            let easing = kept.get(position + 1).map_or(Easing::Linear, |&next| {
                Segment::between(path, &slopes, index, next, window).easing()
            });
            Some(KeptPoint {
                time: point.time,
                focus: point.focus,
                easing,
            })
        })
        .collect()
}

/// The speed of the path at each point (focus units per second), by central differences.
fn slopes(path: &[PathPoint]) -> Vec<Vec2> {
    (0..path.len())
        .map(|index| {
            let before = path.get(index.saturating_sub(1));
            let after = path.get(index + 1).or_else(|| path.get(index));
            match (before, after) {
                (Some(a), Some(b)) if b.time > a.time => {
                    let span = b.time - a.time;
                    Vec2::new(
                        (b.focus.x - a.focus.x) / span,
                        (b.focus.y - a.focus.y) / span,
                    )
                }
                _ => Vec2::default(),
            }
        })
        .collect()
}

/// One segment between two kept points: the straight line, or the cubic of its easing.
struct Segment {
    start: PathPoint,
    end: PathPoint,
    /// `(y1, y2)` of the Bézier easing, `None` for a straight segment.
    curve: Option<(f64, f64)>,
}

impl Segment {
    fn between(
        path: &[PathPoint],
        slopes: &[Vec2],
        start: usize,
        end: usize,
        window: CropWindow,
    ) -> Self {
        let (Some(&a), Some(&b)) = (path.get(start), path.get(end)) else {
            return Self {
                start: PathPoint {
                    time: 0.0,
                    focus: Vec2::default(),
                },
                end: PathPoint {
                    time: 0.0,
                    focus: Vec2::default(),
                },
                curve: None,
            };
        };
        let span = b.time - a.time;
        let moved = window.distance(a.focus, b.focus);
        if span <= 0.0 || moved < STILL {
            return Self {
                start: a,
                end: b,
                curve: None,
            };
        }
        // The axis that moves most (relative to the window) drives the shared easing.
        let horizontal = ((b.focus.x - a.focus.x) / window.width).abs()
            >= ((b.focus.y - a.focus.y) / window.height).abs();
        let pick = |vector: Vec2| if horizontal { vector.x } else { vector.y };
        let delta = pick(b.focus) - pick(a.focus);
        let normalised = |slope: Vec2| (pick(slope) * span / delta).clamp(0.0, MAX_SLOPE);
        let start_slope = slopes.get(start).copied().map_or(1.0, normalised);
        let end_slope = slopes.get(end).copied().map_or(1.0, normalised);
        Self {
            start: a,
            end: b,
            curve: Some((start_slope / 3.0, 1.0 - end_slope / 3.0)),
        }
    }

    /// The focus the segment gives at `time`.
    fn at(&self, time: f64) -> Vec2 {
        let span = self.end.time - self.start.time;
        let share = if span > 0.0 {
            ((time - self.start.time) / span).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let progress = self.curve.map_or(share, |(y1, y2)| hermite(y1, y2, share));
        let (a, b) = (self.start.focus, self.end.focus);
        Vec2::new(
            (b.x - a.x).mul_add(progress, a.x),
            (b.y - a.y).mul_add(progress, a.y),
        )
    }

    /// The easing of the segment's first keyframe.
    fn easing(&self) -> Easing {
        match self.curve {
            Some((y1, y2)) => Easing::CubicBezier {
                x1: 1.0 / 3.0,
                y1: round(y1),
                x2: 2.0 / 3.0,
                y2: round(y2),
            },
            None => Easing::Linear,
        }
    }
}

/// Progress at `share` of a cubic Bézier easing with `x1 = 1/3`, `x2 = 2/3` (then `x(t) = t`).
fn hermite(y1: f64, y2: f64, share: f64) -> f64 {
    let rest = 1.0 - share;
    3.0 * rest * share * (rest * y1 + share * y2) + share.powi(3)
}

/// Easing values kept to four decimals (tidy documents).
fn round(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: CropWindow = CropWindow {
        width: 0.3,
        height: 1.0,
    };

    fn path(xs: impl IntoIterator<Item = f64>) -> Vec<PathPoint> {
        xs.into_iter()
            .zip(0_u32..)
            .map(|(x, index)| PathPoint {
                time: f64::from(index) * 0.2,
                focus: Vec2::new(x, 0.5),
            })
            .collect()
    }

    /// The focus `kept` gives at `time` (each segment eased as its first point says).
    fn value_at(kept: &[KeptPoint], time: f64) -> Vec2 {
        let after = kept.partition_point(|point| point.time <= time);
        let (Some(a), Some(b)) = (kept.get(after.wrapping_sub(1)), kept.get(after)) else {
            return kept.last().map_or(Vec2::default(), |point| point.focus);
        };
        let share = (time - a.time) / (b.time - a.time);
        let progress = match a.easing {
            Easing::CubicBezier { y1, y2, .. } => hermite(y1, y2, share),
            _ => share,
        };
        Vec2::new(
            (b.focus.x - a.focus.x).mul_add(progress, a.focus.x),
            (b.focus.y - a.focus.y).mul_add(progress, a.focus.y),
        )
    }

    #[test]
    fn the_cubic_matches_the_engine_bezier_with_thirds() {
        // x(t) = t when x1 = 1/3 and x2 = 2/3, so y(t) is the curve's progress at time share t.
        let (y1, y2) = (0.2, 0.9);
        for step in 0..=10 {
            let t = f64::from(step) / 10.0;
            let x =
                3.0 * (1.0 - t).powi(2) * t / 3.0 + 3.0 * (1.0 - t) * t * t * 2.0 / 3.0 + t.powi(3);
            assert!((x - t).abs() < 1e-12);
        }
        assert_eq!(hermite(y1, y2, 0.0), 0.0);
        assert!((hermite(y1, y2, 1.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn a_straight_move_keeps_its_ends() {
        let kept = simplified(
            &path((0..30).map(|i| 0.3 + 0.01 * f64::from(i))),
            WINDOW,
            TOLERANCE,
        );
        assert_eq!(kept.len(), 2);
    }

    #[test]
    fn a_smooth_pan_needs_few_keyframes() {
        // An S-shaped pan of 0.2 over 4 s, still before and after.
        let xs = (0..40).map(|i| {
            let share = ((f64::from(i) - 10.0) / 20.0).clamp(0.0, 1.0);
            0.3 + 0.2 * (3.0 - 2.0 * share) * share * share
        });
        let input = path(xs);
        let kept = simplified(&input, WINDOW, TOLERANCE);
        assert!(kept.len() <= 6, "{kept:?}");
        for point in &input {
            let deviation = WINDOW.distance(point.focus, value_at(&kept, point.time));
            assert!(
                deviation <= TOLERANCE + 1e-9,
                "{deviation} at {}",
                point.time
            );
        }
    }

    #[test]
    fn every_dropped_point_stays_within_the_tolerance() {
        let input = path((0..60).map(|i| 0.5 + 0.1 * (f64::from(i) / 6.0).sin()));
        let kept = simplified(&input, WINDOW, TOLERANCE);
        assert!(kept.len() < input.len() / 3, "{}", kept.len());
        for point in &input {
            let deviation = WINDOW.distance(point.focus, value_at(&kept, point.time));
            assert!(
                deviation <= TOLERANCE + 1e-9,
                "{deviation} at {}",
                point.time
            );
        }
    }

    #[test]
    fn segments_never_overshoot_their_ends() {
        let input = path((0..40).map(|i| 0.5 + 0.12 * (f64::from(i) / 4.0).sin()));
        let kept = simplified(&input, WINDOW, TOLERANCE);
        for pair in kept.windows(2) {
            let (low, high) = (
                pair[0].focus.x.min(pair[1].focus.x),
                pair[0].focus.x.max(pair[1].focus.x),
            );
            for step in 0..=20 {
                let time = pair[0].time + (pair[1].time - pair[0].time) * f64::from(step) / 20.0;
                let x = value_at(&kept, time).x;
                assert!(
                    (low - 1e-9..=high + 1e-9).contains(&x),
                    "{x} outside {low}..{high}"
                );
            }
        }
    }

    #[test]
    fn short_paths_are_kept() {
        assert_eq!(simplified(&[], WINDOW, TOLERANCE), []);
        assert_eq!(simplified(&path([0.5]), WINDOW, TOLERANCE).len(), 1);
    }
}
