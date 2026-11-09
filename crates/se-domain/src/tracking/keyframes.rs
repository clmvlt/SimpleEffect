//! The focus keyframes of a tracked clip, assembled from its shots' simplified paths: times on
//! the sequence's frames, linear between keyframes, and a hold before every shot change so the
//! frame jumps to the next shot's subject exactly on the cut's first frame.

use super::simplify::KeptPoint;
use crate::{
    document::{Animatable, Easing, Keyframe},
    spatial::Vec2,
    time::{Flicks, FrameRate, RoundingMode},
};

/// One shot's simplified path and where the shot starts in the item.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct ShotPath {
    /// Item-relative start of the shot (0 for the first, a cut's first frame for the others).
    pub start: Flicks,
    /// Its kept points (item-relative seconds) with their easings, at least one.
    pub points: Vec<KeptPoint>,
}

/// Decimal places kept in focus values (a ten-thousandth of the picture: under a pixel).
const VALUE_DECIMALS: f64 = 10_000.0;

/// The focus animation of `shots` over an item of `duration` on a `rate` timeline: static when
/// every keyframe holds the same value.
pub(super) fn focus_animation(
    shots: &[ShotPath],
    duration: Flicks,
    rate: FrameRate,
) -> Animatable<Vec2> {
    let mut keyframes: Vec<Keyframe<Vec2>> = Vec::new();
    let last_shot = shots.len().saturating_sub(1);
    for (index, shot) in shots.iter().enumerate() {
        let Some(first) = shot.points.first() else {
            continue;
        };
        let start = shot.start.min(duration);
        push_keyframe(&mut keyframes, start, first.focus, first.easing);
        let end = shots
            .get(index + 1)
            .map_or(duration, |next| next.start.min(duration));
        for point in shot.points.iter().skip(1) {
            let time = snapped(point.time, rate).min(duration);
            if time < end || (index == last_shot && time <= end) {
                push_keyframe(&mut keyframes, time, point.focus, point.easing);
            }
        }
        if index == last_shot {
            let last = shot.points.last().map_or(first.focus, |point| point.focus);
            push_keyframe(&mut keyframes, duration, last, Easing::Linear);
        } else if let Some(previous) = keyframes.last_mut() {
            previous.easing = Easing::Hold;
        }
    }
    let Some(first) = keyframes.first() else {
        return Animatable::constant(Vec2::new(0.5, 0.5));
    };
    if keyframes
        .iter()
        .all(|keyframe| keyframe.value == first.value)
    {
        return Animatable::constant(first.value);
    }
    Animatable::Keyframed { keyframes }
}

/// Appends a keyframe after the last one; at the same time it replaces the last one's value
/// (keeping a hold).
fn push_keyframe(keyframes: &mut Vec<Keyframe<Vec2>>, time: Flicks, value: Vec2, easing: Easing) {
    let value = rounded(value);
    match keyframes.last_mut() {
        Some(last) if last.time >= time => {
            if last.time == time {
                last.value = value;
            }
        }
        _ => keyframes.push(Keyframe {
            time,
            value,
            easing,
        }),
    }
}

/// `seconds` on the nearest frame of `rate` (not negative).
fn snapped(seconds: f64, rate: FrameRate) -> Flicks {
    let time = Flicks::from_seconds_f64(seconds.max(0.0)).unwrap_or(Flicks::ZERO);
    on_frame(time, rate, RoundingMode::Nearest)
}

/// `time` moved onto a frame of `rate` with `rounding`.
pub(super) fn on_frame(time: Flicks, rate: FrameRate, rounding: RoundingMode) -> Flicks {
    Flicks::from_frames(time.to_frames(rate, rounding), rate).unwrap_or(time)
}

fn rounded(value: Vec2) -> Vec2 {
    Vec2::new(
        (value.x * VALUE_DECIMALS).round() / VALUE_DECIMALS,
        (value.y * VALUE_DECIMALS).round() / VALUE_DECIMALS,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::FrameRate;

    fn point(time: f64, x: f64) -> KeptPoint {
        KeptPoint {
            time,
            focus: Vec2::new(x, 0.5),
            easing: Easing::Linear,
        }
    }

    fn rate() -> FrameRate {
        FrameRate::new(25, 1).unwrap()
    }

    fn keyframes_of(animation: &Animatable<Vec2>) -> &[Keyframe<Vec2>] {
        match animation {
            Animatable::Keyframed { keyframes } => keyframes,
            Animatable::Static { .. } => &[],
        }
    }

    #[test]
    fn one_shot_runs_from_zero_to_the_end() {
        let shots = [ShotPath {
            start: Flicks::ZERO,
            points: vec![point(0.2, 0.3), point(1.0, 0.4), point(2.0, 0.6)],
        }];
        let animation = focus_animation(&shots, Flicks::from_millis(3000), rate());
        let keyframes = keyframes_of(&animation);
        let times: Vec<i64> = keyframes.iter().map(|k| k.time.get() / 705_600).collect();
        assert_eq!(times, [0, 1000, 2000, 3000]);
        assert_eq!(keyframes[0].value.x, 0.3);
        assert_eq!(keyframes[3].value.x, 0.6);
        assert!(keyframes.iter().all(|k| k.easing == Easing::Linear));
    }

    #[test]
    fn a_cut_holds_then_jumps_on_its_first_frame() {
        let cut = Flicks::from_millis(1520);
        let shots = [
            ShotPath {
                start: Flicks::ZERO,
                points: vec![point(0.0, 0.3), point(1.4, 0.35)],
            },
            ShotPath {
                start: cut,
                points: vec![point(1.6, 0.7), point(2.4, 0.7)],
            },
        ];
        let animation = focus_animation(&shots, Flicks::from_millis(2400), rate());
        let keyframes = keyframes_of(&animation);
        let summary: Vec<(i64, f64, bool)> = keyframes
            .iter()
            .map(|k| (k.time.get() / 705_600, k.value.x, k.easing == Easing::Hold))
            .collect();
        assert_eq!(
            summary,
            [
                (0, 0.3, false),
                (1400, 0.35, true),
                (1520, 0.7, false),
                (2400, 0.7, false)
            ]
        );
    }

    #[test]
    fn a_steady_focus_is_static() {
        let shots = [ShotPath {
            start: Flicks::ZERO,
            points: vec![point(0.0, 0.5), point(2.0, 0.5)],
        }];
        assert_eq!(
            focus_animation(&shots, Flicks::from_millis(2000), rate()),
            Animatable::constant(Vec2::new(0.5, 0.5))
        );
    }

    #[test]
    fn values_are_rounded_and_times_snapped() {
        let shots = [ShotPath {
            start: Flicks::ZERO,
            points: vec![point(0.0, 0.123_456), point(0.517, 0.2)],
        }];
        let animation = focus_animation(&shots, Flicks::from_millis(1000), rate());
        let keyframes = keyframes_of(&animation);
        assert_eq!(keyframes[0].value.x, 0.1235);
        assert_eq!(keyframes[1].time, Flicks::from_millis(520));
    }
}
