//! [`LoudnessMeter`]: EBU R128 loudness of interleaved PCM (crate `ebur128`): integrated
//! loudness, loudness range, peaks, and the short-term loudness sampled at a fixed step.

use ebur128::{EbuR128, Mode};
use se_domain::{Flicks, LoudnessCurve, analysis::finite};

use super::amplitude_db;

/// Everything the meter measured.
#[derive(Debug, Clone, PartialEq)]
pub struct LoudnessReport {
    /// Integrated loudness in LUFS (`None`: everything below the absolute gate).
    pub integrated_lufs: Option<f64>,
    /// Loudness range in LU.
    pub loudness_range_lu: Option<f64>,
    /// Highest true peak of all channels, in dBTP.
    pub true_peak_dbtp: Option<f64>,
    /// Highest sample value of all channels, in dBFS.
    pub sample_peak_dbfs: Option<f64>,
    /// Short-term loudness every step.
    pub short_term: LoudnessCurve,
}

/// Measures loudness while samples arrive in any chunk sizes.
pub struct LoudnessMeter {
    meter: EbuR128,
    channels: usize,
    step: Flicks,
    step_frames: usize,
    frames_in_step: usize,
    curve: Vec<Option<f64>>,
    /// Samples of a frame split across two chunks.
    partial: Vec<f32>,
}

impl LoudnessMeter {
    /// A meter for `channels` interleaved channels at `sample_rate`, sampling the short-term
    /// loudness every `step` (rounded to whole frames).
    ///
    /// # Errors
    /// The meter refuses the format (no channel, absurd rate).
    pub fn new(channels: u16, sample_rate: u32, step: Flicks) -> Result<Self, String> {
        let mode = Mode::I | Mode::S | Mode::LRA | Mode::SAMPLE_PEAK | Mode::TRUE_PEAK;
        let meter = EbuR128::new(u32::from(channels), sample_rate, mode).map_err(|error| {
            format!("the loudness meter refused {channels} channels at {sample_rate} Hz: {error:?}")
        })?;
        // Steps are fractions of a second: far below 2^52 frames.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let step_frames = (step.to_seconds_f64() * f64::from(sample_rate))
            .round()
            .max(1.0) as usize;
        Ok(Self {
            meter,
            channels: usize::from(channels.max(1)),
            step,
            step_frames,
            frames_in_step: 0,
            curve: Vec::new(),
            partial: Vec::new(),
        })
    }

    /// Measures more interleaved samples; a frame split across chunks is completed by the next.
    ///
    /// # Errors
    /// The meter failed (it never does for valid input).
    pub fn push(&mut self, samples: &[f32]) -> Result<(), String> {
        let mut samples = samples;
        if !self.partial.is_empty() {
            let missing = (self.channels - self.partial.len()).min(samples.len());
            self.partial.extend_from_slice(&samples[..missing]);
            samples = &samples[missing..];
            if self.partial.len() == self.channels {
                let frame = std::mem::take(&mut self.partial);
                self.measure(&frame)?;
            }
        }
        let whole = samples.len() - samples.len() % self.channels;
        self.partial.extend_from_slice(&samples[whole..]);
        self.measure(&samples[..whole])
    }

    /// Measures whole frames, sampling the short-term loudness at every step boundary.
    fn measure(&mut self, frames: &[f32]) -> Result<(), String> {
        let mut rest = frames;
        while !rest.is_empty() {
            let frames = (rest.len() / self.channels).min(self.step_frames - self.frames_in_step);
            let (now, later) = rest.split_at(frames * self.channels);
            self.meter
                .add_frames_f32(now)
                .map_err(|error| format!("measuring loudness failed: {error:?}"))?;
            self.frames_in_step += frames;
            if self.frames_in_step == self.step_frames {
                self.curve
                    .push(self.meter.loudness_shortterm().ok().and_then(finite));
                self.frames_in_step = 0;
            }
            rest = later;
        }
        Ok(())
    }

    /// The measurements.
    #[must_use]
    pub fn finish(self) -> LoudnessReport {
        let peak = |measure: &dyn Fn(u32) -> Option<f64>| {
            (0..u32::try_from(self.channels).unwrap_or(1))
                .filter_map(measure)
                .fold(None, |highest: Option<f64>, value| {
                    Some(highest.map_or(value, |highest| highest.max(value)))
                })
                .and_then(amplitude_db)
        };
        LoudnessReport {
            integrated_lufs: self.meter.loudness_global().ok().and_then(finite),
            loudness_range_lu: self.meter.loudness_range().ok().and_then(finite),
            true_peak_dbtp: peak(&|channel| self.meter.true_peak(channel).ok()),
            sample_peak_dbfs: peak(&|channel| self.meter.sample_peak(channel).ok()),
            short_term: LoudnessCurve {
                step: self.step,
                values: self.curve,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::TAU;

    use super::*;

    const RATE: u32 = 48_000;

    /// `seconds` of a 1 kHz sine of `amplitude` on both channels of a stereo signal.
    fn stereo_sine(seconds: u32, amplitude: f32) -> Vec<f32> {
        #[allow(clippy::cast_precision_loss)]
        (0..seconds * RATE)
            .flat_map(|frame| {
                let value = amplitude * (TAU * 1000.0 * frame as f32 / RATE as f32).sin();
                [value, value]
            })
            .collect()
    }

    fn close(actual: Option<f64>, expected: f64, tolerance: f64) -> bool {
        actual.is_some_and(|actual| (actual - expected).abs() <= tolerance)
    }

    #[test]
    fn a_minus_20_dbfs_stereo_sine_measures_minus_20_lufs() {
        let mut meter = LoudnessMeter::new(2, RATE, Flicks::from_millis(500)).unwrap();
        for chunk in stereo_sine(10, 0.1).chunks(4_097) {
            meter.push(chunk).unwrap();
        }
        let report = meter.finish();
        assert!(close(report.integrated_lufs, -20.0, 0.1), "{report:?}");
        assert!(close(report.sample_peak_dbfs, -20.0, 0.01), "{report:?}");
        assert!(close(report.true_peak_dbtp, -20.0, 0.2), "{report:?}");
        assert_eq!(report.short_term.values.len(), 20);
        let first = report.short_term.values[0].unwrap();
        assert!(
            first < -27.0,
            "the first window counts the time before 0 as silence: {first}"
        );
        assert!(
            close(report.short_term.values[19], -20.0, 0.1),
            "{report:?}"
        );
        assert!(close(report.loudness_range_lu, 0.0, 0.1), "{report:?}");
    }

    #[test]
    fn digital_silence_has_no_loudness_and_no_peak() {
        let mut meter = LoudnessMeter::new(1, RATE, Flicks::from_millis(500)).unwrap();
        meter.push(&vec![0.0; RATE as usize * 4]).unwrap();
        let report = meter.finish();
        assert_eq!(report.integrated_lufs, None);
        assert_eq!(report.sample_peak_dbfs, None);
        assert!(report.short_term.values.iter().all(Option::is_none));
    }
}
