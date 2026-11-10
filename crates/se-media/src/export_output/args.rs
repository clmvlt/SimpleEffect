//! ffmpeg arguments of exports (pure, prompt 27): masters encoded from raw RGBA frames
//! (ProRes, DNxHR) and AAC sound joined to a muxed picture.
//!
//! Outputs are partial files whose names do not end with the container's extension, so the
//! muxer is always named with `-f`. Pictures from the renderer are full-range RGB: they are
//! converted to limited-range BT.709 YUV and tagged so.

use std::path::Path;

use se_domain::FrameRate;
use se_protocol::{ExportContainer, VideoExportCodec, VideoExportSettings};

/// The muxer of a container.
#[must_use]
pub const fn muxer_of(container: ExportContainer) -> &'static str {
    match container {
        ExportContainer::Mp4 | ExportContainer::M4a => "mp4",
        ExportContainer::Mov => "mov",
        ExportContainer::Webm => "webm",
        ExportContainer::Wav => "wav",
    }
}

/// Codec options and pixel format of a master codec; `None` for codecs the renderer encodes.
#[must_use]
pub fn master_codec_args(codec: VideoExportCodec) -> Option<(Vec<&'static str>, &'static str)> {
    match codec {
        VideoExportCodec::Prores422Hq => Some((
            vec!["-c:v", "prores_ks", "-profile:v", "3", "-vendor", "apl0"],
            "yuv422p10le",
        )),
        VideoExportCodec::Prores4444 => Some((
            vec![
                "-c:v",
                "prores_ks",
                "-profile:v",
                "4",
                "-vendor",
                "apl0",
                "-alpha_bits",
                "16",
            ],
            "yuva444p10le",
        )),
        VideoExportCodec::DnxhrHq => {
            Some((vec!["-c:v", "dnxhd", "-profile:v", "dnxhr_hq"], "yuv422p"))
        }
        VideoExportCodec::H264
        | VideoExportCodec::Hevc
        | VideoExportCodec::Av1
        | VideoExportCodec::Vp9 => None,
    }
}

fn path_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn owned(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

/// Encodes RGBA frames read from stdin (`video.width × video.height × 4` bytes each, at
/// `frame_rate`) with a master codec, with the PCM sound of `audio` when given, into `output`.
///
/// # Errors
/// A message when `video.codec` is not a master codec.
pub fn frames_args(
    video: &VideoExportSettings,
    frame_rate: FrameRate,
    audio: Option<&Path>,
    output: &Path,
) -> Result<Vec<String>, String> {
    let (codec, pixel_format) = master_codec_args(video.codec)
        .ok_or_else(|| format!("{:?} is encoded by the editor, not by ffmpeg", video.codec))?;
    // stdin carries the frames: no `-nostdin`.
    let mut args = owned(&["-hide_banner", "-loglevel", "error"]);
    args.extend(owned(&["-f", "rawvideo", "-pix_fmt", "rgba", "-s"]));
    args.push(format!("{}x{}", video.width, video.height));
    args.push("-framerate".to_owned());
    args.push(format!(
        "{}/{}",
        frame_rate.numerator(),
        frame_rate.denominator()
    ));
    args.extend(owned(&["-i", "pipe:0"]));
    if let Some(audio) = audio {
        args.push("-i".to_owned());
        args.push(path_arg(audio));
    }
    args.extend(owned(&["-map", "0:v:0"]));
    if audio.is_some() {
        args.extend(owned(&["-map", "1:a:0", "-c:a", "pcm_s24le"]));
    }
    args.push("-vf".to_owned());
    args.push(format!(
        "scale=out_color_matrix=bt709:out_range=tv,format={pixel_format}"
    ));
    args.extend(owned(&codec));
    args.extend(owned(&[
        "-color_primaries",
        "bt709",
        "-color_trc",
        "bt709",
        "-colorspace",
        "bt709",
        "-color_range",
        "tv",
        "-f",
        "mov",
        "-y",
    ]));
    args.push(path_arg(output));
    Ok(args)
}

/// Encodes the WAV `audio` as AAC at `bits_per_second`, joined to the picture of `video` (copied)
/// when given, into `output` (`container`: MP4, MOV or M4A).
#[must_use]
pub fn aac_remux_args(
    video: Option<&Path>,
    audio: &Path,
    bits_per_second: u32,
    container: ExportContainer,
    output: &Path,
) -> Vec<String> {
    let mut args = owned(&["-hide_banner", "-loglevel", "error"]);
    if let Some(video) = video {
        args.push("-i".to_owned());
        args.push(path_arg(video));
    }
    args.push("-i".to_owned());
    args.push(path_arg(audio));
    let audio_input = usize::from(video.is_some());
    if video.is_some() {
        args.extend(owned(&["-map", "0:v:0", "-c:v", "copy"]));
    }
    args.push("-map".to_owned());
    args.push(format!("{audio_input}:a:0"));
    args.extend(owned(&["-c:a", "aac", "-b:a"]));
    args.push(bits_per_second.to_string());
    args.extend(owned(&[
        "-movflags",
        "+faststart",
        "-f",
        muxer_of(container),
        "-y",
    ]));
    args.push(path_arg(output));
    args
}

#[cfg(test)]
mod tests {
    use se_protocol::{ExportQuality, VideoRate};

    use super::*;

    fn video(codec: VideoExportCodec) -> VideoExportSettings {
        VideoExportSettings {
            codec,
            width: 1920,
            height: 1080,
            rate: VideoRate::Quality {
                level: ExportQuality::VeryHigh,
            },
        }
    }

    fn joined(args: &[String]) -> String {
        args.join(" ")
    }

    #[test]
    fn masters_read_raw_frames_and_the_wav() {
        let args = frames_args(
            &video(VideoExportCodec::Prores422Hq),
            FrameRate::FPS_23_976,
            Some(Path::new("film.mov.audio.part")),
            Path::new("film.mov.part"),
        )
        .unwrap();
        let line = joined(&args);
        assert!(
            line.starts_with(
                "-hide_banner -loglevel error -f rawvideo -pix_fmt rgba -s 1920x1080 \
                 -framerate 24000/1001 -i pipe:0 -i film.mov.audio.part -map 0:v:0 -map 1:a:0 \
                 -c:a pcm_s24le"
            ),
            "{line}"
        );
        assert!(
            line.contains("format=yuv422p10le -c:v prores_ks -profile:v 3"),
            "{line}"
        );
        assert!(
            line.ends_with("-color_range tv -f mov -y film.mov.part"),
            "{line}"
        );
    }

    #[test]
    fn silent_masters_have_one_input() {
        let args = frames_args(
            &video(VideoExportCodec::DnxhrHq),
            FrameRate::FPS_25,
            None,
            Path::new("out.part"),
        )
        .unwrap();
        let line = joined(&args);
        assert!(!line.contains("1:a:0"), "{line}");
        assert!(line.contains("-c:v dnxhd -profile:v dnxhr_hq"), "{line}");
        assert!(line.contains("format=yuv422p "), "{line}");
        assert!(
            frames_args(
                &video(VideoExportCodec::H264),
                FrameRate::FPS_25,
                None,
                Path::new("out.part")
            )
            .is_err()
        );
    }

    #[test]
    fn aac_is_joined_to_a_copied_picture() {
        let line = joined(&aac_remux_args(
            Some(Path::new("v.part")),
            Path::new("a.part"),
            192_000,
            ExportContainer::Mp4,
            Path::new("film.mp4.part"),
        ));
        assert_eq!(
            line,
            "-hide_banner -loglevel error -i v.part -i a.part -map 0:v:0 -c:v copy -map 1:a:0 \
             -c:a aac -b:a 192000 -movflags +faststart -f mp4 -y film.mp4.part"
        );
        let sound_only = joined(&aac_remux_args(
            None,
            Path::new("a.part"),
            128_000,
            ExportContainer::M4a,
            Path::new("son.m4a.part"),
        ));
        assert!(
            sound_only.contains("-i a.part -map 0:a:0 -c:a aac"),
            "{sound_only}"
        );
    }
}
