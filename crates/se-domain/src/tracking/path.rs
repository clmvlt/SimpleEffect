//! A shot's path: the focus wanted at every sample, its gaps filled (the subject turned away,
//! hidden for a moment, or never seen).

use crate::spatial::Vec2;

/// One point of a path: a time in seconds (item-relative) and a focus.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct PathPoint {
    /// Seconds since the item's start.
    pub time: f64,
    /// The focus at that time.
    pub focus: Vec2,
}

/// The focus at each of `times`: `wanted` where known, interpolated linearly in time between two
/// known samples, held before the first and after the last, and `fallback` everywhere when no
/// sample is known (a shot without faces stays where it is put).
pub(super) fn filled(times: &[f64], wanted: &[Option<Vec2>], fallback: Vec2) -> Vec<PathPoint> {
    let known: Vec<(f64, Vec2)> = times
        .iter()
        .zip(wanted)
        .filter_map(|(&time, focus)| focus.map(|focus| (time, focus)))
        .collect();
    times
        .iter()
        .map(|&time| PathPoint {
            time,
            focus: value_at(&known, time).unwrap_or(fallback),
        })
        .collect()
}

/// The value of the known points at `time` (linear between neighbours, held outside).
fn value_at(known: &[(f64, Vec2)], time: f64) -> Option<Vec2> {
    let after = known.partition_point(|(at, _)| *at < time);
    match (
        after.checked_sub(1).and_then(|index| known.get(index)),
        known.get(after),
    ) {
        (Some(&(t0, a)), Some(&(t1, b))) => {
            let span = t1 - t0;
            let share = if span > 0.0 { (time - t0) / span } else { 0.0 };
            Some(Vec2::new(
                (b.x - a.x).mul_add(share, a.x),
                (b.y - a.y).mul_add(share, a.y),
            ))
        }
        (Some(&(_, value)), None) | (None, Some(&(_, value))) => Some(value),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gaps_are_interpolated_and_edges_held() {
        let times = [0.0, 0.2, 0.4, 0.6, 0.8];
        let wanted = [
            None,
            Some(Vec2::new(0.2, 0.5)),
            None,
            Some(Vec2::new(0.4, 0.5)),
            None,
        ];
        let path = filled(&times, &wanted, Vec2::new(0.5, 0.5));
        let xs: Vec<f64> = path.iter().map(|point| point.focus.x).collect();
        assert_eq!(xs.len(), 5);
        for (x, expected) in xs.iter().zip([0.2, 0.2, 0.3, 0.4, 0.4]) {
            assert!((x - expected).abs() < 1e-12, "{xs:?}");
        }
    }

    #[test]
    fn nothing_known_gives_the_fallback() {
        let path = filled(&[0.0, 0.2], &[None, None], Vec2::new(0.5, 0.5));
        assert!(path.iter().all(|point| point.focus == Vec2::new(0.5, 0.5)));
    }
}
