//! Pure analysis of decoded media: silences and loudness of PCM sound ([`AudioAnalyzer`]), shot
//! changes from ffmpeg's `showinfo` output ([`parse_scene_changes`]). No IO: the jobs feed them
//! samples or text, so they are tested on synthetic signals and recorded output.

mod audio_analyzer;
mod loudness;
mod scene_changes;
mod silence;

pub use self::{
    audio_analyzer::AudioAnalyzer,
    loudness::{LoudnessMeter, LoudnessReport},
    scene_changes::parse_scene_changes,
    silence::SilenceDetector,
};
use se_domain::Flicks;

/// The source time of sample frame `frame` at `sample_rate` (flicks divide every common rate
/// exactly; others round down).
#[must_use]
pub fn frame_time(frame: u64, sample_rate: u32) -> Flicks {
    let flicks =
        i128::from(frame) * i128::from(Flicks::PER_SECOND) / i128::from(sample_rate.max(1));
    i64::try_from(flicks)
        .ok()
        .and_then(|value| Flicks::new(value).ok())
        .unwrap_or(Flicks::MAX)
}

/// `linear` amplitude (1.0 = full scale) in decibels; `None` for silence (0).
#[must_use]
pub fn amplitude_db(linear: f64) -> Option<f64> {
    se_domain::analysis::finite(20.0 * linear.log10())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_convert_to_exact_flicks() {
        assert_eq!(frame_time(48_000, 48_000), Flicks::one_second());
        assert_eq!(frame_time(22_050, 44_100), Flicks::from_millis(500));
    }

    #[test]
    fn amplitudes_convert_to_decibels() {
        assert_eq!(amplitude_db(1.0), Some(0.0));
        assert!((amplitude_db(0.5).unwrap() + 6.0206).abs() < 1e-3);
        assert_eq!(amplitude_db(0.0), None);
    }
}
