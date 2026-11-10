//! Export settings (pure): what each container accepts, the limits, and how the file is put
//! together ([`pipeline_of`]).

use se_protocol::{
    AudioEncoderChoice, AudioExportCodec, AudioExportSettings, ExportContainer, ExportPipeline,
    ExportSettings, VideoExportCodec, VideoExportSettings, VideoRate,
};

/// Narrowest or lowest picture accepted, in pixels.
pub const MIN_DIMENSION: u32 = 16;
/// Widest picture accepted (8K UHD).
pub const MAX_WIDTH: u32 = 7680;
/// Tallest picture accepted (portrait 8K).
pub const MAX_HEIGHT: u32 = 7680;
/// Most pixels per frame (8K UHD).
pub const MAX_PIXELS: u64 = 7680 * 4320;
/// Largest RGBA frame an export uploads for ffmpeg, in bytes.
pub const MAX_FRAME_BYTES: usize = 7680 * 4320 * 4;
/// Largest chunk of a muxed file or WAV uploaded at once, in bytes (the renderer sends 8 MiB).
pub const MAX_CHUNK_BYTES: usize = 64 * 1024 * 1024;
/// Picture bitrates accepted, in bits per second.
pub const VIDEO_BITRATE_RANGE: std::ops::RangeInclusive<u32> = 100_000..=500_000_000;
/// Compressed sound bitrates accepted, in bits per second.
pub const AUDIO_BITRATE_RANGE: std::ops::RangeInclusive<u32> = 32_000..=512_000;

/// The file extension of `container` (without the dot).
#[must_use]
pub const fn extension_of(container: ExportContainer) -> &'static str {
    match container {
        ExportContainer::Mp4 => "mp4",
        ExportContainer::Mov => "mov",
        ExportContainer::Webm => "webm",
        ExportContainer::M4a => "m4a",
        ExportContainer::Wav => "wav",
    }
}

/// Whether ffmpeg encodes `codec` (frames uploaded by the renderer) rather than WebCodecs.
#[must_use]
pub const fn is_ffmpeg_codec(codec: VideoExportCodec) -> bool {
    matches!(
        codec,
        VideoExportCodec::Prores422Hq | VideoExportCodec::Prores4444 | VideoExportCodec::DnxhrHq
    )
}

/// How the file of `settings` is put together (checked settings).
#[must_use]
pub fn pipeline_of(settings: &ExportSettings) -> ExportPipeline {
    if let Some(video) = settings.video
        && is_ffmpeg_codec(video.codec)
    {
        return ExportPipeline::FfmpegFrames {
            with_audio: settings.audio.is_some(),
        };
    }
    match settings.audio {
        Some(audio) if audio.encoder == AudioEncoderChoice::Ffmpeg => ExportPipeline::AacRemux {
            with_video: settings.video.is_some(),
        },
        _ => ExportPipeline::Muxed,
    }
}

/// Checks `settings`; the error says what to change.
///
/// # Errors
/// A message naming the setting that cannot be used and what would work.
pub fn check_settings(settings: &ExportSettings) -> Result<(), String> {
    if settings.video.is_none() && settings.audio.is_none() {
        return Err("an export needs a picture, a sound or both".to_owned());
    }
    if let Some(video) = &settings.video {
        check_video(settings.container, video)?;
    }
    if let Some(audio) = settings.audio {
        check_audio(settings, audio)?;
    }
    Ok(())
}

fn check_video(container: ExportContainer, video: &VideoExportSettings) -> Result<(), String> {
    let allowed: &[VideoExportCodec] = match container {
        ExportContainer::Mp4 => &[
            VideoExportCodec::H264,
            VideoExportCodec::Hevc,
            VideoExportCodec::Av1,
        ],
        ExportContainer::Mov => &[
            VideoExportCodec::H264,
            VideoExportCodec::Hevc,
            VideoExportCodec::Prores422Hq,
            VideoExportCodec::Prores4444,
            VideoExportCodec::DnxhrHq,
        ],
        ExportContainer::Webm => &[VideoExportCodec::Vp9, VideoExportCodec::Av1],
        ExportContainer::M4a | ExportContainer::Wav => {
            return Err(format!(
                "a .{} file holds sound only: choose MP4, MOV or WebM for a picture",
                extension_of(container)
            ));
        }
    };
    if !allowed.contains(&video.codec) {
        return Err(format!(
            "a .{} file cannot hold {:?} pictures",
            extension_of(container),
            video.codec
        ));
    }
    check_size(video.width, video.height)?;
    if let VideoRate::Bitrate { bits_per_second } = video.rate
        && !VIDEO_BITRATE_RANGE.contains(&bits_per_second)
    {
        return Err(format!(
            "the picture bitrate must be between {} and {} kb/s",
            VIDEO_BITRATE_RANGE.start() / 1000,
            VIDEO_BITRATE_RANGE.end() / 1000
        ));
    }
    Ok(())
}

fn check_size(width: u32, height: u32) -> Result<(), String> {
    if !width.is_multiple_of(2) || !height.is_multiple_of(2) {
        return Err(format!(
            "the picture size {width}×{height} must have even dimensions"
        ));
    }
    if !(MIN_DIMENSION..=MAX_WIDTH).contains(&width)
        || !(MIN_DIMENSION..=MAX_HEIGHT).contains(&height)
        || u64::from(width) * u64::from(height) > MAX_PIXELS
    {
        return Err(format!(
            "the picture size {width}×{height} is out of range (from {MIN_DIMENSION} px to 8K)"
        ));
    }
    Ok(())
}

fn check_audio(settings: &ExportSettings, audio: AudioExportSettings) -> Result<(), String> {
    let ffmpeg_video = settings
        .video
        .is_some_and(|video| is_ffmpeg_codec(video.codec));
    let allowed: &[AudioExportCodec] = match settings.container {
        _ if ffmpeg_video => &[AudioExportCodec::Pcm],
        ExportContainer::Mp4 => &[AudioExportCodec::Aac, AudioExportCodec::Opus],
        ExportContainer::Mov | ExportContainer::M4a => &[AudioExportCodec::Aac],
        ExportContainer::Webm => &[AudioExportCodec::Opus],
        ExportContainer::Wav => &[AudioExportCodec::Pcm],
    };
    if !allowed.contains(&audio.codec) {
        let place = if ffmpeg_video {
            "ProRes and DNxHR masters carry PCM sound".to_owned()
        } else {
            format!(
                "a .{} file cannot hold it",
                extension_of(settings.container)
            )
        };
        return Err(format!(
            "{:?} sound cannot be used here: {place}",
            audio.codec
        ));
    }
    if audio.encoder == AudioEncoderChoice::Ffmpeg && audio.codec != AudioExportCodec::Aac {
        return Err("only AAC sound can be encoded by ffmpeg".to_owned());
    }
    if audio.codec != AudioExportCodec::Pcm && !AUDIO_BITRATE_RANGE.contains(&audio.bits_per_second)
    {
        return Err(format!(
            "the sound bitrate must be between {} and {} kb/s",
            AUDIO_BITRATE_RANGE.start() / 1000,
            AUDIO_BITRATE_RANGE.end() / 1000
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use se_protocol::ExportQuality;

    use super::*;

    fn video(codec: VideoExportCodec) -> VideoExportSettings {
        VideoExportSettings {
            codec,
            width: 1920,
            height: 1080,
            rate: VideoRate::Quality {
                level: ExportQuality::High,
            },
        }
    }

    fn audio(codec: AudioExportCodec, encoder: AudioEncoderChoice) -> AudioExportSettings {
        AudioExportSettings {
            codec,
            bits_per_second: 192_000,
            encoder,
        }
    }

    fn settings(
        container: ExportContainer,
        video_codec: Option<VideoExportCodec>,
        audio_codec: Option<(AudioExportCodec, AudioEncoderChoice)>,
    ) -> ExportSettings {
        ExportSettings {
            container,
            video: video_codec.map(video),
            audio: audio_codec.map(|(codec, encoder)| audio(codec, encoder)),
        }
    }

    const WEB: AudioEncoderChoice = AudioEncoderChoice::WebCodecs;
    const FFMPEG: AudioEncoderChoice = AudioEncoderChoice::Ffmpeg;

    #[test]
    fn usual_presets_are_accepted() {
        for accepted in [
            settings(
                ExportContainer::Mp4,
                Some(VideoExportCodec::H264),
                Some((AudioExportCodec::Aac, WEB)),
            ),
            settings(
                ExportContainer::Mp4,
                Some(VideoExportCodec::Hevc),
                Some((AudioExportCodec::Aac, FFMPEG)),
            ),
            settings(
                ExportContainer::Mov,
                Some(VideoExportCodec::Prores422Hq),
                Some((AudioExportCodec::Pcm, WEB)),
            ),
            settings(
                ExportContainer::Webm,
                Some(VideoExportCodec::Vp9),
                Some((AudioExportCodec::Opus, WEB)),
            ),
            settings(ExportContainer::Mp4, Some(VideoExportCodec::Av1), None),
            settings(
                ExportContainer::M4a,
                None,
                Some((AudioExportCodec::Aac, WEB)),
            ),
            settings(
                ExportContainer::Wav,
                None,
                Some((AudioExportCodec::Pcm, WEB)),
            ),
        ] {
            assert_eq!(check_settings(&accepted), Ok(()), "{accepted:?}");
        }
    }

    #[test]
    fn mismatches_are_refused_with_what_to_change() {
        let cases = [
            (
                settings(ExportContainer::Mp4, None, None),
                "picture, a sound",
            ),
            (
                settings(
                    ExportContainer::Mp4,
                    Some(VideoExportCodec::Prores422Hq),
                    None,
                ),
                "cannot hold",
            ),
            (
                settings(ExportContainer::Wav, Some(VideoExportCodec::H264), None),
                "sound only",
            ),
            (
                settings(
                    ExportContainer::Mov,
                    Some(VideoExportCodec::DnxhrHq),
                    Some((AudioExportCodec::Aac, WEB)),
                ),
                "PCM",
            ),
            (
                settings(
                    ExportContainer::Webm,
                    Some(VideoExportCodec::Vp9),
                    Some((AudioExportCodec::Aac, WEB)),
                ),
                "cannot hold it",
            ),
            (
                settings(
                    ExportContainer::Mp4,
                    None,
                    Some((AudioExportCodec::Opus, FFMPEG)),
                ),
                "only AAC",
            ),
        ];
        for (refused, expected) in cases {
            let message = check_settings(&refused).unwrap_err();
            assert!(message.contains(expected), "{message}");
        }
    }

    #[test]
    fn sizes_and_bitrates_have_limits() {
        let mut odd = settings(ExportContainer::Mp4, Some(VideoExportCodec::H264), None);
        if let Some(video) = &mut odd.video {
            video.width = 1919;
        }
        assert!(check_settings(&odd).unwrap_err().contains("even"));
        let mut huge = settings(ExportContainer::Mp4, Some(VideoExportCodec::H264), None);
        if let Some(video) = &mut huge.video {
            video.width = 7680;
            video.height = 7680;
        }
        assert!(check_settings(&huge).unwrap_err().contains("out of range"));
        let mut slow = settings(ExportContainer::Mp4, Some(VideoExportCodec::H264), None);
        if let Some(video) = &mut slow.video {
            video.rate = VideoRate::Bitrate {
                bits_per_second: 1000,
            };
        }
        assert!(check_settings(&slow).unwrap_err().contains("bitrate"));
        let mut quiet = settings(
            ExportContainer::M4a,
            None,
            Some((AudioExportCodec::Aac, WEB)),
        );
        if let Some(audio) = &mut quiet.audio {
            audio.bits_per_second = 8000;
        }
        assert!(
            check_settings(&quiet)
                .unwrap_err()
                .contains("sound bitrate")
        );
    }

    #[test]
    fn pipelines_follow_the_encoders() {
        assert_eq!(
            pipeline_of(&settings(
                ExportContainer::Mp4,
                Some(VideoExportCodec::H264),
                Some((AudioExportCodec::Aac, WEB)),
            )),
            ExportPipeline::Muxed
        );
        assert_eq!(
            pipeline_of(&settings(
                ExportContainer::Mov,
                Some(VideoExportCodec::Prores4444),
                Some((AudioExportCodec::Pcm, WEB)),
            )),
            ExportPipeline::FfmpegFrames { with_audio: true }
        );
        assert_eq!(
            pipeline_of(&settings(
                ExportContainer::Mov,
                Some(VideoExportCodec::Prores422Hq),
                None
            )),
            ExportPipeline::FfmpegFrames { with_audio: false }
        );
        assert_eq!(
            pipeline_of(&settings(
                ExportContainer::Mp4,
                Some(VideoExportCodec::H264),
                Some((AudioExportCodec::Aac, FFMPEG)),
            )),
            ExportPipeline::AacRemux { with_video: true }
        );
        assert_eq!(
            pipeline_of(&settings(
                ExportContainer::M4a,
                None,
                Some((AudioExportCodec::Aac, FFMPEG))
            )),
            ExportPipeline::AacRemux { with_video: false }
        );
    }
}
