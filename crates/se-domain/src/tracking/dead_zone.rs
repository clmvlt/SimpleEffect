//! The « camera operator » dead zone: the frame holds still while the subject moves inside a
//! central zone; once the subject leaves it, the frame follows the subject for as long as it
//! keeps moving, then holds again where it settled (recentred on it). Small head movements stop
//! producing micro-moves and keyframes, and a walk is followed in one continuous move rather
//! than a staircase of small jumps. The smoothing that follows turns the starts and stops into
//! eased pans.

use super::{path::PathPoint, window::CropWindow};
use crate::spatial::Vec2;

/// Samples (0.6 s at 5 per second) the subject must stay near its last position for the frame
/// to settle there…
const SETTLE_SAMPLES: usize = 3;

/// …within this share of the zone's half-size (stricter than the zone: a slow walk keeps moving).
const SETTLE_SHARE: f64 = 0.5;

/// `path` with the dead zone applied: a zone `dead_zone` × the window wide (and high).
pub(super) fn held_path(path: &[PathPoint], window: CropWindow, dead_zone: f64) -> Vec<PathPoint> {
    let Some(first) = path.first() else {
        return Vec::new();
    };
    if dead_zone <= 0.0 {
        return path.to_vec();
    }
    let half = Vec2::new(
        dead_zone * window.width / 2.0,
        dead_zone * window.height / 2.0,
    );
    let outside = |a: Vec2, b: Vec2| (a.x - b.x).abs() > half.x || (a.y - b.y).abs() > half.y;
    let near = |a: Vec2, b: Vec2| {
        (a.x - b.x).abs() <= SETTLE_SHARE * half.x && (a.y - b.y).abs() <= SETTLE_SHARE * half.y
    };
    let mut held = Some(first.focus);
    path.iter()
        .enumerate()
        .map(|(index, point)| {
            let focus = match held {
                Some(rest) if outside(point.focus, rest) => {
                    held = None;
                    point.focus
                }
                Some(rest) => rest,
                None => {
                    let recent = path.get(index.saturating_sub(SETTLE_SAMPLES - 1)..=index);
                    let settled = index + 1 >= SETTLE_SAMPLES
                        && recent.is_some_and(|recent| {
                            recent.iter().all(|other| near(other.focus, point.focus))
                        });
                    if settled {
                        held = Some(point.focus);
                    }
                    point.focus
                }
            };
            PathPoint {
                time: point.time,
                focus,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(xs: &[f64]) -> Vec<PathPoint> {
        xs.iter()
            .zip(0_u32..)
            .map(|(&x, index)| PathPoint {
                time: f64::from(index) * 0.2,
                focus: Vec2::new(x, 0.5),
            })
            .collect()
    }

    fn xs(points: &[PathPoint]) -> Vec<f64> {
        points.iter().map(|point| point.focus.x).collect()
    }

    const WINDOW: CropWindow = CropWindow {
        width: 0.3,
        height: 1.0,
    };

    #[test]
    fn small_moves_inside_the_zone_are_ignored() {
        // Zone 0.2 × 0.3 = 0.06 wide: ±0.03 around the held focus.
        let held = held_path(&path(&[0.5, 0.52, 0.48, 0.51, 0.49]), WINDOW, 0.2);
        assert_eq!(xs(&held), [0.5; 5]);
    }

    #[test]
    fn leaving_the_zone_follows_then_settles() {
        let held = held_path(
            &path(&[0.5, 0.52, 0.6, 0.61, 0.615, 0.62, 0.63, 0.61, 0.6]),
            WINDOW,
            0.2,
        );
        // Follows from 0.6 while moving; settles at 0.62 (three samples within ±0.015), then
        // holds there although the subject sways.
        assert_eq!(
            xs(&held),
            [0.5, 0.5, 0.6, 0.61, 0.615, 0.62, 0.62, 0.62, 0.62]
        );
    }

    #[test]
    fn a_continuous_walk_is_followed_without_steps() {
        let walk: Vec<f64> = (0..10).map(|index| 0.3 + 0.04 * f64::from(index)).collect();
        let held = held_path(&path(&walk), WINDOW, 0.2);
        // Held at first, then each sample followed.
        assert_eq!(xs(&held)[0], 0.3);
        assert_eq!(&xs(&held)[1..], &walk[1..]);
    }

    #[test]
    fn no_zone_follows_every_move() {
        let input = path(&[0.5, 0.52, 0.48]);
        assert_eq!(held_path(&input, WINDOW, 0.0), input);
    }
}
