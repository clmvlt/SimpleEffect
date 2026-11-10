//! What an export's progress and finished file tell (pure): the time left, and what ffprobe
//! found compared with what was asked.

use se_domain::{Flicks, MediaInfo};
use se_protocol::{ExportCheck, ExportSettings};

/// Fraction done before the time left is estimated (the first frames load fonts and decoders).
const ETA_MIN_FRACTION: f64 = 0.02;
/// Seconds of rendering before the time left is estimated.
const ETA_MIN_ELAPSED_SECONDS: f64 = 2.0;
/// Difference between the file's duration and the span tolerated before a warning (sound
/// encoders add a little padding).
const DURATION_TOLERANCE_SECONDS: f64 = 0.25;

/// Seconds left when `fraction` of the work took `elapsed_seconds`, once that is meaningful.
#[must_use]
pub fn eta_seconds(elapsed_seconds: f64, fraction: f64) -> Option<f64> {
    if !(ETA_MIN_FRACTION..=1.0).contains(&fraction) || elapsed_seconds < ETA_MIN_ELAPSED_SECONDS {
        return None;
    }
    Some(elapsed_seconds * (1.0 - fraction) / fraction)
}

/// What ffprobe found in a finished export (`info`), with a warning when it differs from
/// `settings` or from the span's `duration`.
#[must_use]
pub fn file_check(info: &MediaInfo, settings: &ExportSettings, duration: Flicks) -> ExportCheck {
    let found = info.duration.map_or(0.0, Flicks::to_seconds_f64);
    let video = info.video_stream.as_ref();
    let audio = info.audio_streams.first();
    let mut problems = Vec::new();
    let expected = duration.to_seconds_f64();
    if (found - expected).abs() > DURATION_TOLERANCE_SECONDS {
        problems.push(format!(
            "the file lasts {found:.2} s instead of {expected:.2} s"
        ));
    }
    match (settings.video, video) {
        (Some(_), None) => problems.push("the file has no picture".to_owned()),
        (Some(asked), Some(stream))
            if (stream.width, stream.height) != (asked.width, asked.height) =>
        {
            problems.push(format!(
                "the picture is {}×{} instead of {}×{}",
                stream.width, stream.height, asked.width, asked.height
            ));
        }
        _ => {}
    }
    if settings.audio.is_some() && audio.is_none() {
        problems.push("the file has no sound".to_owned());
    }
    ExportCheck {
        duration_seconds: found,
        video_codec: video.map(|stream| stream.codec.clone()),
        width: video.map(|stream| stream.width),
        height: video.map(|stream| stream.height),
        audio_codec: audio.map(|stream| stream.codec.clone()),
        warning: (!problems.is_empty()).then(|| problems.join("; ")),
    }
}

#[cfg(test)]
mod tests {
    use se_domain::{AudioStreamInfo, VideoStreamInfo};
    use se_protocol::{
        AudioEncoderChoice, AudioExportCodec, AudioExportSettings, ExportContainer, ExportQuality,
        VideoExportCodec, VideoExportSettings, VideoRate,
    };

    use super::*;

    fn settings() -> ExportSettings {
        ExportSettings {
            container: ExportContainer::Mp4,
            video: Some(VideoExportSettings {
                codec: VideoExportCodec::H264,
                width: 1920,
                height: 1080,
                rate: VideoRate::Quality {
                    level: ExportQuality::High,
                },
            }),
            audio: Some(AudioExportSettings {
                codec: AudioExportCodec::Aac,
                bits_per_second: 192_000,
                encoder: AudioEncoderChoice::WebCodecs,
            }),
        }
    }

    fn info(seconds: f64, width: u32, with_audio: bool) -> MediaInfo {
        let video = VideoStreamInfo {
            codec: "h264".to_owned(),
            profile: None,
            level: None,
            codec_tag: None,
            pixel_format: None,
            bit_depth: None,
            chroma_subsampling: None,
            width,
            height: 1080,
            frame_rate: None,
            rotation: 0,
        };
        let audio = AudioStreamInfo {
            codec: "aac".to_owned(),
            profile: None,
            channels: 2,
            sample_rate: 48_000,
        };
        MediaInfo {
            duration: Some(Flicks::from_seconds_f64(seconds).unwrap()),
            container: "mov,mp4".to_owned(),
            video_stream: Some(video),
            audio_streams: if with_audio { vec![audio] } else { Vec::new() },
        }
    }

    #[test]
    fn a_matching_file_has_no_warning() {
        let check = file_check(
            &info(10.02, 1920, true),
            &settings(),
            Flicks::from_millis(10_000),
        );
        assert_eq!(check.warning, None);
        assert_eq!(check.video_codec.as_deref(), Some("h264"));
        assert_eq!(check.audio_codec.as_deref(), Some("aac"));
        assert_eq!((check.width, check.height), (Some(1920), Some(1080)));
    }

    #[test]
    fn differences_are_warned_about() {
        let check = file_check(
            &info(7.0, 1280, false),
            &settings(),
            Flicks::from_millis(10_000),
        );
        let warning = check.warning.unwrap();
        assert!(warning.contains("7.00 s instead of 10.00 s"), "{warning}");
        assert!(
            warning.contains("1280×1080 instead of 1920×1080"),
            "{warning}"
        );
        assert!(warning.contains("no sound"), "{warning}");
    }

    #[test]
    fn the_time_left_waits_for_meaningful_progress() {
        assert_eq!(eta_seconds(1.0, 0.5), None);
        assert_eq!(eta_seconds(10.0, 0.01), None);
        assert_eq!(eta_seconds(10.0, 0.25), Some(30.0));
        assert_eq!(eta_seconds(10.0, 1.0), Some(0.0));
    }
}
