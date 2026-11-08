//! [`AudioAnalyzer`]: one pass over interleaved PCM measuring loudness and silences together.

use se_domain::{AudioAnalysis, Flicks, SilenceSettings};

use super::{LoudnessMeter, SilenceDetector};

/// Step of the stored short-term loudness curve.
const SHORT_TERM_STEP: Flicks = Flicks::from_millis(500);

/// Measures what [`AudioAnalysis`] holds; feed it interleaved `f32` samples.
pub struct AudioAnalyzer {
    channels: u16,
    sample_rate: u32,
    silence_settings: SilenceSettings,
    loudness: LoudnessMeter,
    silences: SilenceDetector,
}

impl AudioAnalyzer {
    /// An analyzer for `channels` interleaved channels at `sample_rate`.
    ///
    /// # Errors
    /// The loudness meter refuses the format.
    pub fn new(
        channels: u16,
        sample_rate: u32,
        silence_settings: SilenceSettings,
    ) -> Result<Self, String> {
        Ok(Self {
            channels,
            sample_rate,
            silence_settings,
            loudness: LoudnessMeter::new(channels, sample_rate, SHORT_TERM_STEP)?,
            silences: SilenceDetector::new(channels, sample_rate, silence_settings),
        })
    }

    /// Measures more samples.
    ///
    /// # Errors
    /// The loudness meter failed.
    pub fn push(&mut self, samples: &[f32]) -> Result<(), String> {
        self.silences.push(samples);
        self.loudness.push(samples)
    }

    /// The result.
    #[must_use]
    pub fn finish(self) -> AudioAnalysis {
        let loudness = self.loudness.finish();
        let (silences, levels) = self.silences.finish_with_levels();
        AudioAnalysis {
            sample_rate: self.sample_rate,
            channels: self.channels,
            integrated_lufs: loudness.integrated_lufs,
            loudness_range_lu: loudness.loudness_range_lu,
            true_peak_dbtp: loudness.true_peak_dbtp,
            sample_peak_dbfs: loudness.sample_peak_dbfs,
            short_term: loudness.short_term,
            silence_settings: self.silence_settings,
            silences,
            levels: Some(levels),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measures_loudness_and_silences_in_one_pass() {
        let rate = 48_000_u32;
        #[allow(clippy::cast_precision_loss)]
        let tone: Vec<f32> = (0..rate * 4)
            .map(|frame| 0.25 * (std::f32::consts::TAU * 440.0 * frame as f32 / rate as f32).sin())
            .collect();
        let mut analyzer = AudioAnalyzer::new(1, rate, SilenceSettings::SPEECH).unwrap();
        analyzer.push(&tone).unwrap();
        analyzer.push(&vec![0.0; rate as usize]).unwrap();
        let analysis = analyzer.finish();
        assert_eq!(analysis.silences.len(), 1);
        assert_eq!(analysis.silences[0].start(), Flicks::from_millis(4_000));
        assert!(analysis.integrated_lufs.is_some());
        assert_eq!(analysis.short_term.values.len(), 10);
        assert_eq!(analysis.channels, 1);
        let levels = analysis.levels.unwrap();
        assert_eq!(levels.values.len(), 250, "5 s of 20 ms windows");
    }
}
