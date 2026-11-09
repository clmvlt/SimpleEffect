//! Smoothing of a shot's path with the One-Euro filter (Casiez, Roussel, Vogel, CHI 2012): a
//! low-pass filter whose cutoff rises with the speed, so slow drifts are smoothed hard and real
//! moves are followed. The whole path is known, so it runs forward and backward and the two are
//! averaged: their lags cancel and the frame starts moving with the subject, not after it.

use super::{as_float, path::PathPoint};
use crate::spatial::Vec2;

/// Cutoff (Hz) of the least smoothed setting (`smoothing` 0): close to the raw samples.
const MAX_MIN_CUTOFF: f64 = 3.0;
/// Cutoff (Hz) of the most smoothed setting (`smoothing` 1): only slow, steady moves.
const MIN_MIN_CUTOFF: f64 = 0.05;
/// Speed coefficient of the least smoothed setting (cutoff gained per unit of focus per second).
const MAX_BETA: f64 = 0.6;
/// Cutoff (Hz) of the speed estimate.
const DERIVATIVE_CUTOFF: f64 = 1.0;
/// Samples averaged to start each pass (a raw first sample would keep its noise at the shot's
/// edges).
const SEED_SAMPLES: usize = 5;

/// Parameters of the One-Euro filter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct OneEuroParams {
    /// Cutoff frequency at rest, in hertz.
    pub min_cutoff: f64,
    /// Cutoff gained per unit of speed.
    pub beta: f64,
}

impl OneEuroParams {
    /// The parameters of a smoothing setting in `[0, 1]`: the cutoff falls geometrically from
    /// 3 Hz to 0.05 Hz, the speed coefficient linearly to 0.
    #[must_use]
    pub fn for_smoothing(smoothing: f64) -> Self {
        let share = if smoothing.is_finite() {
            smoothing.clamp(0.0, 1.0)
        } else {
            0.5
        };
        Self {
            min_cutoff: MAX_MIN_CUTOFF * (MIN_MIN_CUTOFF / MAX_MIN_CUTOFF).powf(share),
            beta: MAX_BETA * (1.0 - share),
        }
    }
}

/// `path` smoothed on both axes: two rounds of forward and backward passes averaged (one round
/// turns a step into two exponentials meeting at a corner; two give an S-shaped pan that few
/// curved keyframes follow).
pub(super) fn smoothed(path: &[PathPoint], params: OneEuroParams) -> Vec<PathPoint> {
    let once = round_trip(path, params);
    round_trip(&once, params)
}

/// One forward and one backward pass, averaged.
fn round_trip(path: &[PathPoint], params: OneEuroParams) -> Vec<PathPoint> {
    let forward = filter_axes(path.iter(), params);
    let mut backward = filter_axes(path.iter().rev(), params);
    backward.reverse();
    path.iter()
        .zip(forward.iter().zip(&backward))
        .map(|(point, (a, b))| PathPoint {
            time: point.time,
            focus: Vec2::new(f64::midpoint(a.x, b.x), f64::midpoint(a.y, b.y)),
        })
        .collect()
}

/// One pass over `points` in their order (time measured as the distance from the first), started
/// from the mean of the first [`SEED_SAMPLES`].
fn filter_axes<'a>(
    points: impl Iterator<Item = &'a PathPoint> + Clone,
    params: OneEuroParams,
) -> Vec<Vec2> {
    let seed: Vec<Vec2> = points
        .clone()
        .take(SEED_SAMPLES)
        .map(|point| point.focus)
        .collect();
    let count = as_float(seed.len().max(1));
    let mean = |axis: fn(&Vec2) -> f64| seed.iter().map(axis).sum::<f64>() / count;
    let mut x = OneEuro::new(params, mean(|focus| focus.x));
    let mut y = OneEuro::new(params, mean(|focus| focus.y));
    let mut origin = None;
    points
        .map(|point| {
            let start = *origin.get_or_insert(point.time);
            let elapsed = (point.time - start).abs();
            Vec2::new(
                x.filter(elapsed, point.focus.x),
                y.filter(elapsed, point.focus.y),
            )
        })
        .collect()
}

/// The filter's state on one axis.
struct OneEuro {
    params: OneEuroParams,
    /// The last time and filtered value (time 0 and the seed at first).
    previous: (f64, f64),
    speed: f64,
}

impl OneEuro {
    const fn new(params: OneEuroParams, seed: f64) -> Self {
        Self {
            params,
            previous: (0.0, seed),
            speed: 0.0,
        }
    }

    /// The filtered value of `value` at `time` (seconds from the first sample, increasing).
    fn filter(&mut self, time: f64, value: f64) -> f64 {
        let (last_time, last_value) = self.previous;
        let elapsed = time - last_time;
        if elapsed <= 0.0 {
            return last_value;
        }
        let raw_speed = (value - last_value) / elapsed;
        self.speed = lerp(self.speed, raw_speed, alpha(DERIVATIVE_CUTOFF, elapsed));
        let cutoff = self
            .params
            .beta
            .mul_add(self.speed.abs(), self.params.min_cutoff);
        let filtered = lerp(last_value, value, alpha(cutoff, elapsed));
        self.previous = (time, filtered);
        filtered
    }
}

/// Smoothing factor of a first-order low-pass filter of `cutoff` hertz over `elapsed` seconds.
fn alpha(cutoff: f64, elapsed: f64) -> f64 {
    let tau = 1.0 / (std::f64::consts::TAU * cutoff);
    elapsed / (elapsed + tau)
}

fn lerp(from: f64, to: f64, share: f64) -> f64 {
    (to - from).mul_add(share, from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(values: impl IntoIterator<Item = f64>) -> Vec<PathPoint> {
        values
            .into_iter()
            .zip(0_u32..)
            .map(|(x, index)| PathPoint {
                time: f64::from(index) * 0.2,
                focus: Vec2::new(x, 0.5),
            })
            .collect()
    }

    fn deviation(values: &[f64]) -> f64 {
        let count = as_float(values.len());
        let mean = values.iter().sum::<f64>() / count;
        (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count).sqrt()
    }

    #[test]
    fn smoothing_settings_span_the_cutoffs() {
        let fast = OneEuroParams::for_smoothing(0.0);
        let slow = OneEuroParams::for_smoothing(1.0);
        assert!((fast.min_cutoff - 3.0).abs() < 1e-12);
        assert!((slow.min_cutoff - 0.05).abs() < 1e-12);
        assert_eq!(slow.beta, 0.0);
        assert!(OneEuroParams::for_smoothing(0.5).min_cutoff < 1.0);
    }

    #[test]
    fn jitter_is_reduced_more_with_more_smoothing() {
        let jittery: Vec<PathPoint> =
            path((0..50).map(|index| if index % 2 == 0 { 0.48 } else { 0.52 }));
        let spread = |smoothing| {
            let xs: Vec<f64> = smoothed(&jittery, OneEuroParams::for_smoothing(smoothing))
                .iter()
                .map(|point| point.focus.x)
                .collect();
            deviation(&xs)
        };
        let raw = 0.02;
        assert!(spread(0.2) < raw);
        assert!(
            spread(0.8) < spread(0.2) / 2.0,
            "{} {}",
            spread(0.8),
            spread(0.2)
        );
    }

    #[test]
    fn a_step_becomes_a_symmetric_pan_without_lag() {
        let step: Vec<PathPoint> = path((0..40).map(|index| if index < 20 { 0.3 } else { 0.7 }));
        let xs: Vec<f64> = smoothed(&step, OneEuroParams::for_smoothing(0.5))
            .iter()
            .map(|point| point.focus.x)
            .collect();
        // Halfway at the step, monotonic, still within the two positions.
        let middle = f64::midpoint(xs[19], xs[20]);
        assert!((middle - 0.5).abs() < 0.02, "{xs:?}");
        assert!(xs.windows(2).all(|pair| pair[1] >= pair[0] - 1e-12));
        assert!(xs.iter().all(|x| (0.3 - 1e-12..=0.7 + 1e-12).contains(x)));
        // The pan takes a while: neither end has moved much yet two seconds away.
        assert!(xs[9] < 0.33 && xs[30] > 0.67, "{xs:?}");
    }
}
