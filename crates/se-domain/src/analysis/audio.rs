//! [`AudioAnalysis`]: silences, loudness (EBU R128) and peaks of an asset's first sound stream.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::time::{Flicks, TimeRange};

/// What silence means for a detection: windows quieter than `threshold_dbfs`, lasting at least
/// `min_duration`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SilenceSettings {
    /// Level (RMS of short windows, in dBFS) under which sound counts as silence.
    pub threshold_dbfs: f64,
    /// Shortest silence reported; quieter stretches that are shorter are ignored.
    pub min_duration: Flicks,
}

impl SilenceSettings {
    /// Defaults suited to speech: −40 dBFS for at least half a second (pauses worth cutting).
    pub const SPEECH: Self = Self {
        threshold_dbfs: -40.0,
        min_duration: Flicks::from_millis(500),
    };
}

/// Short-term loudness (3-second window, EBU R128) sampled every `step`: value `i` is the
/// loudness of the 3 seconds ending at `(i + 1) × step` (the first values count the time before
/// the start as silence), `None` while that window is silent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LoudnessCurve {
    /// Time between two values.
    pub step: Flicks,
    /// Loudness values in LUFS.
    pub values: Vec<Option<f64>>,
}

/// Full result of the audio analysis of an asset (stored in the project database).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AudioAnalysis {
    /// Sample rate of the analysed sound.
    pub sample_rate: u32,
    /// Channels analysed (at most 2: more are downmixed to stereo first).
    pub channels: u16,
    /// Integrated loudness in LUFS; `None` when everything is below the absolute gate (silence).
    pub integrated_lufs: Option<f64>,
    /// Loudness range in LU; `None` when it cannot be measured (too short or silent).
    pub loudness_range_lu: Option<f64>,
    /// Highest true peak (oversampled) of all channels, in dBTP; `None` for digital silence.
    pub true_peak_dbtp: Option<f64>,
    /// Highest sample value of all channels, in dBFS; `None` for digital silence.
    pub sample_peak_dbfs: Option<f64>,
    /// Short-term loudness over time.
    pub short_term: LoudnessCurve,
    /// How silences were detected.
    pub silence_settings: SilenceSettings,
    /// Silent stretches in source time, in order, never overlapping.
    pub silences: Vec<TimeRange>,
}

impl AudioAnalysis {
    /// The summary kept in the document.
    #[must_use]
    pub fn summary(&self) -> AudioSummary {
        let silence_duration = self
            .silences
            .iter()
            .try_fold(Flicks::ZERO, |total, range| {
                total.checked_add(range.duration())
            })
            .unwrap_or(Flicks::MAX);
        AudioSummary {
            integrated_lufs: self.integrated_lufs,
            loudness_range_lu: self.loudness_range_lu,
            true_peak_dbtp: self.true_peak_dbtp,
            silence_count: u32::try_from(self.silences.len()).unwrap_or(u32::MAX),
            silence_duration,
        }
    }
}

/// The audio analysis of an asset as the document keeps it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AudioSummary {
    /// Integrated loudness in LUFS (`None`: silent).
    pub integrated_lufs: Option<f64>,
    /// Loudness range in LU.
    pub loudness_range_lu: Option<f64>,
    /// Highest true peak in dBTP.
    pub true_peak_dbtp: Option<f64>,
    /// Number of silences detected.
    pub silence_count: u32,
    /// Their total duration.
    pub silence_duration: Flicks,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(start_ms: u32, end_ms: u32) -> TimeRange {
        TimeRange::from_start_end(Flicks::from_millis(start_ms), Flicks::from_millis(end_ms))
            .unwrap()
    }

    #[test]
    fn the_summary_counts_silences_and_their_length() {
        let analysis = AudioAnalysis {
            sample_rate: 48_000,
            channels: 2,
            integrated_lufs: Some(-16.5),
            loudness_range_lu: Some(4.0),
            true_peak_dbtp: Some(-1.2),
            sample_peak_dbfs: Some(-1.5),
            short_term: LoudnessCurve {
                step: Flicks::from_millis(500),
                values: vec![None, Some(-18.0)],
            },
            silence_settings: SilenceSettings::SPEECH,
            silences: vec![range(1_000, 2_000), range(5_000, 5_500)],
        };
        let summary = analysis.summary();
        assert_eq!(summary.silence_count, 2);
        assert_eq!(summary.silence_duration, Flicks::from_millis(1_500));
        assert_eq!(summary.integrated_lufs, Some(-16.5));
        assert_eq!(summary.true_peak_dbtp, Some(-1.2));
    }

    #[test]
    fn speech_settings_are_minus_40_dbfs_for_half_a_second() {
        assert_eq!(
            serde_json::to_string(&SilenceSettings::SPEECH).unwrap(),
            r#"{"thresholdDbfs":-40.0,"minDuration":352800000}"#
        );
    }
}
