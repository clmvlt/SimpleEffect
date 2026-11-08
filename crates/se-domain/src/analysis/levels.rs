//! [`LevelEnvelope`]: the sound level of an asset every 20 ms, from which silences are found for
//! any threshold without decoding the sound again (smart cuts, prompt 22, ADR-0032).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::SilenceSettings;
use crate::time::{Flicks, TimeRange};

/// The RMS level of an asset's sound (all channels together) over consecutive windows of
/// `step`: value `i` describes `[i × step, (i + 1) × step)` in source time (the last window may
/// be partial), in **whole decibels below full scale** — `0` is full scale, `12` is −12 dBFS,
/// [`Self::FLOOR`] means −100 dBFS or quieter (digital silence included).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LevelEnvelope {
    /// Length of a window.
    pub step: Flicks,
    /// Decibels below full scale, one per window.
    pub values: Vec<u8>,
}

impl LevelEnvelope {
    /// The window length measured by the audio analysis: 20 ms (short enough for the gaps
    /// between words).
    pub const STEP: Flicks = Flicks::from_millis(20);

    /// The quietest value: −100 dBFS or less.
    pub const FLOOR: u8 = 100;

    /// The value of a window whose samples have the mean square `mean_square` (full scale = 1).
    #[must_use]
    pub fn value_of(mean_square: f64) -> u8 {
        if !mean_square.is_finite() || mean_square <= 0.0 {
            return Self::FLOOR;
        }
        let below = (-10.0 * mean_square.log10()).round();
        // Clamped to 0..=FLOOR first: the conversion is exact.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let value = below.clamp(0.0, f64::from(Self::FLOOR)) as u8;
        value
    }

    /// The level of a value, in dBFS.
    #[must_use]
    pub fn dbfs(value: u8) -> f64 {
        -f64::from(value)
    }

    /// The silences of `source` (source time): runs of windows quieter than
    /// `settings.threshold_dbfs` lasting at least `settings.min_duration`, clipped to `source`,
    /// in order. Windows beyond the envelope count as sound (never cut what was not measured).
    #[must_use]
    pub fn silences(&self, settings: SilenceSettings, source: TimeRange) -> Vec<TimeRange> {
        let step = self.step.get();
        if step <= 0 {
            return Vec::new();
        }
        let first = usize::try_from(source.start().get().max(0) / step).unwrap_or(usize::MAX);
        let mut silences = Vec::new();
        let mut run_start: Option<usize> = None;
        for (index, value) in self.values.iter().enumerate().skip(first) {
            if self
                .window_start(index)
                .is_none_or(|start| start >= source.end())
            {
                break;
            }
            let silent = Self::dbfs(*value) < settings.threshold_dbfs;
            match (silent, run_start) {
                (true, None) => run_start = Some(index),
                (false, Some(start)) => {
                    self.close_run(start..index, settings, source, &mut silences);
                    run_start = None;
                }
                _ => {}
            }
        }
        if let Some(start) = run_start {
            self.close_run(start..self.values.len(), settings, source, &mut silences);
        }
        silences
    }

    /// Where window `index` starts.
    fn window_start(&self, index: usize) -> Option<Flicks> {
        self.step.checked_mul(i64::try_from(index).ok()?)
    }

    /// Records the run of windows `run` when, clipped to `source`, it is long enough.
    fn close_run(
        &self,
        run: std::ops::Range<usize>,
        settings: SilenceSettings,
        source: TimeRange,
        silences: &mut Vec<TimeRange>,
    ) {
        let (Some(start), Some(end)) = (self.window_start(run.start), self.window_start(run.end))
        else {
            return;
        };
        let Ok(run) = TimeRange::from_start_end(start, end) else {
            return;
        };
        if let Some(clipped) = run.intersection(source)
            && !clipped.is_empty()
            && clipped.duration() >= settings.min_duration
        {
            silences.push(clipped);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(values: &[u8]) -> LevelEnvelope {
        LevelEnvelope {
            step: LevelEnvelope::STEP,
            values: values.to_vec(),
        }
    }

    fn range(start_ms: u32, end_ms: u32) -> TimeRange {
        TimeRange::from_start_end(Flicks::from_millis(start_ms), Flicks::from_millis(end_ms))
            .unwrap()
    }

    fn settings(threshold_dbfs: f64, min_ms: u32) -> SilenceSettings {
        SilenceSettings {
            threshold_dbfs,
            min_duration: Flicks::from_millis(min_ms),
        }
    }

    #[test]
    fn values_are_whole_decibels_below_full_scale() {
        assert_eq!(LevelEnvelope::value_of(1.0), 0);
        assert_eq!(LevelEnvelope::value_of(0.01), 20);
        assert_eq!(LevelEnvelope::value_of(1e-4), 40);
        assert_eq!(LevelEnvelope::value_of(0.0), LevelEnvelope::FLOOR);
        assert_eq!(LevelEnvelope::value_of(1e-30), LevelEnvelope::FLOOR);
        assert_eq!(
            LevelEnvelope::value_of(4.0),
            0,
            "clipped sound stays at full scale"
        );
        assert!((LevelEnvelope::dbfs(40) + 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn silences_follow_the_threshold_and_the_minimum_duration() {
        // 20 ms windows: sound, 5 windows at −50 dBFS, sound, 2 windows at −50, sound.
        let levels = envelope(&[10, 50, 50, 50, 50, 50, 12, 50, 50, 10]);
        let whole = range(0, 200);
        assert_eq!(
            levels.silences(settings(-40.0, 60), whole),
            [range(20, 120)]
        );
        assert_eq!(
            levels.silences(settings(-40.0, 40), whole),
            [range(20, 120), range(140, 180)]
        );
        assert_eq!(levels.silences(settings(-60.0, 40), whole), []);
        assert_eq!(
            levels.silences(settings(-11.0, 20), whole),
            [range(20, 180)],
            "a loose threshold counts the −12 dBFS window as silence"
        );
    }

    #[test]
    fn silences_are_clipped_to_the_source_range() {
        let levels = envelope(&[50, 50, 50, 50, 10, 50, 50, 50]);
        assert_eq!(
            levels.silences(settings(-40.0, 20), range(30, 130)),
            [range(30, 80), range(100, 130)]
        );
        assert_eq!(
            levels.silences(settings(-40.0, 60), range(30, 130)),
            [],
            "clipped runs shorter than the minimum are dropped"
        );
        assert_eq!(
            levels.silences(settings(-40.0, 20), range(100, 400)),
            [range(100, 160)],
            "time past the envelope is not silence"
        );
    }
}
