//! [`SilenceDetector`]: silent stretches of interleaved PCM, by the RMS level of short windows.

use se_domain::{SilenceSettings, TimeRange};

use super::frame_time;

/// Length of a measurement window: 20 ms (short enough to find the gaps between words).
const WINDOWS_PER_SECOND: u32 = 50;

/// Finds the stretches where the RMS level of 20 ms windows (all channels together) stays under
/// a threshold for a minimum duration. Feed it interleaved samples in any chunk sizes.
#[derive(Debug, Clone)]
pub struct SilenceDetector {
    channels: usize,
    sample_rate: u32,
    window_frames: u64,
    /// Mean square under which a window is silent (`10^(threshold_dbfs / 10)`).
    threshold_mean_square: f64,
    /// Shortest silence reported, in frames.
    min_frames: u64,
    /// Sum of squares of the current window and the samples it holds so far.
    window_sum: f64,
    window_samples: u64,
    /// Frames fully measured so far.
    frames: u64,
    /// First frame of the silent run in progress.
    run_start: Option<u64>,
    silences: Vec<TimeRange>,
}

impl SilenceDetector {
    /// A detector for `channels` interleaved channels at `sample_rate`.
    #[must_use]
    pub fn new(channels: u16, sample_rate: u32, settings: SilenceSettings) -> Self {
        let sample_rate = sample_rate.max(1);
        let min_seconds = settings.min_duration.to_seconds_f64().max(0.0);
        Self {
            channels: usize::from(channels.max(1)),
            sample_rate,
            window_frames: u64::from((sample_rate / WINDOWS_PER_SECOND).max(1)),
            threshold_mean_square: 10_f64.powf(settings.threshold_dbfs / 10.0),
            // Durations are far below 2^52 frames: the conversion is exact enough.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            min_frames: (min_seconds * f64::from(sample_rate)).round() as u64,
            window_sum: 0.0,
            window_samples: 0,
            frames: 0,
            run_start: None,
            silences: Vec::new(),
        }
    }

    /// Measures more interleaved samples.
    pub fn push(&mut self, samples: &[f32]) {
        let window_samples = self.window_frames * self.channels as u64;
        for sample in samples {
            self.window_sum += f64::from(*sample) * f64::from(*sample);
            self.window_samples += 1;
            if self.window_samples == window_samples {
                self.close_window();
            }
        }
    }

    /// The silences found, the last window (even partial) included.
    #[must_use]
    pub fn finish(mut self) -> Vec<TimeRange> {
        if self.window_samples >= self.channels as u64 {
            self.close_window();
        }
        self.end_run();
        self.silences
    }

    fn close_window(&mut self) {
        let frames = self.window_samples / self.channels as u64;
        // Sample counts stay far below 2^52: the conversion is exact.
        #[allow(clippy::cast_precision_loss)]
        let mean_square = self.window_sum / self.window_samples.max(1) as f64;
        let silent = mean_square < self.threshold_mean_square;
        if silent && self.run_start.is_none() {
            self.run_start = Some(self.frames);
        }
        self.frames += frames;
        if !silent {
            self.end_run_at(self.frames - frames);
        }
        self.window_sum = 0.0;
        self.window_samples = 0;
    }

    fn end_run(&mut self) {
        self.end_run_at(self.frames);
    }

    fn end_run_at(&mut self, end: u64) {
        if let Some(start) = self.run_start.take()
            && end - start >= self.min_frames
            && let Ok(range) = TimeRange::from_start_end(
                frame_time(start, self.sample_rate),
                frame_time(end, self.sample_rate),
            )
        {
            self.silences.push(range);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::TAU;

    use se_domain::Flicks;

    use super::*;

    const RATE: u32 = 48_000;

    /// `seconds` of a 440 Hz sine of `amplitude` on `channels` channels.
    fn sine(seconds: f32, amplitude: f32, channels: usize) -> Vec<f32> {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let frames = (seconds * RATE as f32) as usize;
        #[allow(clippy::cast_precision_loss)]
        (0..frames)
            .flat_map(|frame| {
                let value = amplitude * (TAU * 440.0 * frame as f32 / RATE as f32).sin();
                std::iter::repeat_n(value, channels)
            })
            .collect()
    }

    fn silence(seconds: f32, channels: usize) -> Vec<f32> {
        sine(seconds, 0.0, channels)
    }

    fn range(start_ms: u32, end_ms: u32) -> TimeRange {
        TimeRange::from_start_end(Flicks::from_millis(start_ms), Flicks::from_millis(end_ms))
            .unwrap()
    }

    #[test]
    fn finds_a_silence_between_two_tones() {
        let mut detector = SilenceDetector::new(2, RATE, SilenceSettings::SPEECH);
        detector.push(&sine(1.0, 0.5, 2));
        detector.push(&silence(1.0, 2));
        detector.push(&sine(1.0, 0.5, 2));
        assert_eq!(detector.finish(), [range(1_000, 2_000)]);
    }

    #[test]
    fn short_pauses_and_quiet_tones_above_the_threshold_are_not_silences() {
        let mut detector = SilenceDetector::new(1, RATE, SilenceSettings::SPEECH);
        detector.push(&sine(1.0, 0.5, 1));
        detector.push(&silence(0.3, 1));
        // −30 dBFS peak (≈ −33 dBFS RMS) is above the −40 dBFS threshold.
        detector.push(&sine(1.0, 0.031_6, 1));
        assert_eq!(detector.finish(), []);
    }

    #[test]
    fn a_trailing_silence_runs_to_the_end_and_chunk_sizes_do_not_matter() {
        let mut signal = sine(0.5, 0.5, 1);
        signal.extend(silence(0.75, 1));
        let mut detector = SilenceDetector::new(1, RATE, SilenceSettings::SPEECH);
        for chunk in signal.chunks(777) {
            detector.push(chunk);
        }
        assert_eq!(detector.finish(), [range(500, 1_250)]);
    }
}
