//! ffmpeg arguments of a mezzanine or proxy job (pure): picture, sound and container treatment
//! of a [`TranscodePlan`], with the options of a hardware encoder or of libx264/libx265.
//!
//! Every output keeps the coded picture and its display rotation (`-noautorotate` copies the
//! display matrix), so the engine treats originals and derived copies alike.

use std::path::Path;

use se_app::ports::{
    AudioEncoding, AudioPlan, Container, MediaJobSpec, TranscodePlan, VideoCodecTarget,
    VideoEncoding, VideoPlan,
};

use crate::{encoders::HardwareEncoder, platform};

/// libx264/libx265 preset: a balance of speed and size for long camera files.
const SOFTWARE_PRESET: &str = "fast";
/// NVENC preset (p1 fastest … p7 best).
const NVENC_PRESET: &str = "p5";

/// Options an encoder needs around the input and in the filter chain.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EncoderArgs {
    /// Before `-i` (hardware device).
    pub input: Vec<String>,
    /// Appended to the picture filter chain (upload to the GPU).
    pub filters: Vec<String>,
    /// Codec options.
    pub output: Vec<String>,
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

/// VideoToolbox quality (1–100, higher is better) matching a CRF (lower is better).
fn videotoolbox_quality(crf: u8) -> u8 {
    let quality = 100 - u16::from(crf) * 5 / 2;
    u8::try_from(quality.clamp(1, 100)).unwrap_or(60)
}

/// Codec options of a picture encoding, for `hardware` or software.
#[must_use]
pub fn video_encoder_args(
    encoding: &VideoEncoding,
    hardware: Option<HardwareEncoder>,
) -> EncoderArgs {
    let quality = encoding.quality.to_string();
    let hevc = encoding.codec == VideoCodecTarget::HevcMain10;
    let (profile, software_format, hardware_format) = if hevc {
        ("main10", "yuv420p10le", "p010le")
    } else {
        ("high", "yuv420p", "yuv420p")
    };
    let mut args = EncoderArgs::default();
    let output = &mut args.output;
    match hardware {
        None => {
            let codec = if hevc { "libx265" } else { "libx264" };
            output.extend(strings(&[
                "-c:v",
                codec,
                "-preset",
                SOFTWARE_PRESET,
                "-crf",
                &quality,
            ]));
            output.extend(strings(&[
                "-profile:v",
                profile,
                "-pix_fmt",
                software_format,
            ]));
            if hevc {
                output.extend(strings(&["-x265-params", "log-level=error"]));
            }
            if encoding.fast_decode {
                output.extend(strings(&["-tune", "fastdecode"]));
            }
        }
        Some(HardwareEncoder::Nvenc) => {
            let codec = HardwareEncoder::Nvenc.encoder(encoding.codec);
            output.extend(strings(&[
                "-c:v",
                codec,
                "-preset",
                NVENC_PRESET,
                "-rc",
                "vbr",
            ]));
            output.extend(strings(&["-cq", &quality, "-b:v", "0"]));
            output.extend(strings(&[
                "-profile:v",
                profile,
                "-pix_fmt",
                hardware_format,
            ]));
        }
        Some(HardwareEncoder::VideoToolbox) => {
            let codec = HardwareEncoder::VideoToolbox.encoder(encoding.codec);
            let quality = videotoolbox_quality(encoding.quality).to_string();
            output.extend(strings(&["-c:v", codec, "-q:v", &quality]));
            output.extend(strings(&[
                "-profile:v",
                profile,
                "-pix_fmt",
                hardware_format,
            ]));
        }
        Some(HardwareEncoder::Vaapi) => {
            let upload = if hevc {
                "format=p010,hwupload"
            } else {
                "format=nv12,hwupload"
            };
            args.input = strings(&["-vaapi_device", platform::VAAPI_DEVICE]);
            args.filters.push(upload.to_owned());
            let codec = HardwareEncoder::Vaapi.encoder(encoding.codec);
            args.output.extend(strings(&[
                "-c:v", codec, "-rc_mode", "CQP", "-qp", &quality,
            ]));
            args.output.extend(strings(&["-profile:v", profile]));
        }
    }
    let keyframes = encoding.keyframe_interval.max(1).to_string();
    args.output.extend(strings(&["-g", &keyframes]));
    if hevc {
        args.output.extend(strings(&["-tag:v", "hvc1"]));
    }
    if encoding.fast_decode {
        args.output.extend(strings(&["-bf", "0"]));
    }
    args
}

/// The scale filter giving a shorter side of at most `short_side` pixels (never enlarging).
#[must_use]
pub fn scale_filter(short_side: u32) -> String {
    format!(
        "scale=w='if(gte(iw,ih),-2,min({short_side},iw))':h='if(gte(iw,ih),min({short_side},ih),-2)':flags=bicubic"
    )
}

/// Every argument of the ffmpeg run writing `plan` to `output`, encoding with `hardware` (or
/// software). Progress goes to stdout (`-progress pipe:1`).
#[must_use]
pub fn transcode_args(
    spec: &MediaJobSpec,
    plan: &TranscodePlan,
    hardware: Option<HardwareEncoder>,
    output: &Path,
) -> Vec<String> {
    let mut args = strings(&["-hide_banner", "-nostdin", "-v", "error", "-y", "-nostats"]);
    args.extend(strings(&["-progress", "pipe:1"]));
    let encoder = match &plan.recipe.video {
        VideoPlan::Encode(encoding) => Some((encoding, video_encoder_args(encoding, hardware))),
        VideoPlan::Drop | VideoPlan::Copy | VideoPlan::Still => None,
    };
    if let Some((_, encoder)) = &encoder {
        args.extend(encoder.input.iter().cloned());
    }
    args.extend(strings(&["-noautorotate", "-i"]));
    args.push(spec.input.to_string_lossy().into_owned());
    match (&plan.recipe.video, encoder) {
        (VideoPlan::Drop, _) => args.push("-vn".to_owned()),
        (VideoPlan::Copy, _) => args.extend(strings(&["-map", "0:v:0", "-c:v", "copy"])),
        (VideoPlan::Still, _) => {
            args.extend(strings(&["-map", "0:v:0", "-frames:v", "1", "-c:v", "png"]));
        }
        (VideoPlan::Encode(_), Some((encoding, encoder))) => {
            args.extend(strings(&["-map", "0:v:0"]));
            let mut filters: Vec<String> = encoding
                .max_short_side
                .map(scale_filter)
                .into_iter()
                .collect();
            filters.extend(encoder.filters);
            if !filters.is_empty() {
                args.push("-vf".to_owned());
                args.push(filters.join(","));
            }
            args.extend(encoder.output);
        }
        (VideoPlan::Encode(_), None) => {}
    }
    match plan.recipe.audio {
        AudioPlan::Drop => args.push("-an".to_owned()),
        AudioPlan::Copy => args.extend(strings(&["-map", "0:a:0", "-c:a", "copy"])),
        AudioPlan::Encode(AudioEncoding::Aac { bitrate_kbps }) => {
            args.extend(strings(&["-map", "0:a:0", "-c:a", "aac", "-b:a"]));
            args.push(format!("{bitrate_kbps}k"));
        }
        AudioPlan::Encode(AudioEncoding::Flac) => {
            args.extend(strings(&["-map", "0:a:0", "-c:a", "flac"]));
        }
    }
    args.extend(strings(&["-sn", "-dn", "-map_chapters", "-1"]));
    if matches!(plan.recipe.container, Container::Mp4 | Container::M4a) {
        args.extend(strings(&["-movflags", "+faststart"]));
    }
    args.push(output.to_string_lossy().into_owned());
    args
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use se_app::ports::TranscodeRecipe;
    use se_domain::{AssetKind, MediaInfo};
    use se_protocol::DerivedKind;

    use super::*;

    fn encoding(
        codec: VideoCodecTarget,
        max_short_side: Option<u32>,
        fast_decode: bool,
    ) -> VideoEncoding {
        VideoEncoding {
            codec,
            quality: 14,
            max_short_side,
            keyframe_interval: 25,
            fast_decode,
        }
    }

    fn joined(args: &[String]) -> String {
        args.join(" ")
    }

    fn plan(
        video: VideoPlan,
        audio: AudioPlan,
        container: Container,
    ) -> (MediaJobSpec, TranscodePlan) {
        let spec = MediaJobSpec {
            task: se_app::ports::MediaTask::Thumbnails,
            input: PathBuf::from("in.mov"),
            output_dir: PathBuf::from("cache"),
            asset_kind: AssetKind::Video,
            media_info: MediaInfo {
                duration: None,
                container: "mov".to_owned(),
                video_stream: None,
                audio_streams: Vec::new(),
            },
        };
        let plan = TranscodePlan {
            derived: DerivedKind::Mezzanine,
            output: PathBuf::from("mezzanine.mp4"),
            recipe: TranscodeRecipe {
                container,
                video,
                audio,
            },
        };
        (spec, plan)
    }

    #[test]
    fn software_encodings_use_crf_profiles_and_short_gops() {
        let hevc = video_encoder_args(&encoding(VideoCodecTarget::HevcMain10, None, false), None);
        assert_eq!(
            joined(&hevc.output),
            "-c:v libx265 -preset fast -crf 14 -profile:v main10 -pix_fmt yuv420p10le -x265-params log-level=error -g 25 -tag:v hvc1"
        );
        let proxy =
            video_encoder_args(&encoding(VideoCodecTarget::H264High, Some(540), true), None);
        assert_eq!(
            joined(&proxy.output),
            "-c:v libx264 -preset fast -crf 14 -profile:v high -pix_fmt yuv420p -tune fastdecode -g 25 -bf 0"
        );
    }

    #[test]
    fn hardware_encodings_map_the_quality_and_upload_frames_when_needed() {
        let nvenc = video_encoder_args(
            &encoding(VideoCodecTarget::HevcMain10, None, false),
            Some(HardwareEncoder::Nvenc),
        );
        assert_eq!(
            joined(&nvenc.output),
            "-c:v hevc_nvenc -preset p5 -rc vbr -cq 14 -b:v 0 -profile:v main10 -pix_fmt p010le -g 25 -tag:v hvc1"
        );
        let toolbox = video_encoder_args(
            &encoding(VideoCodecTarget::H264High, None, false),
            Some(HardwareEncoder::VideoToolbox),
        );
        assert!(joined(&toolbox.output).starts_with("-c:v h264_videotoolbox -q:v 65 "));
        let vaapi = video_encoder_args(
            &encoding(VideoCodecTarget::HevcMain10, None, false),
            Some(HardwareEncoder::Vaapi),
        );
        assert_eq!(vaapi.input, ["-vaapi_device", platform::VAAPI_DEVICE]);
        assert_eq!(vaapi.filters, ["format=p010,hwupload"]);
    }

    #[test]
    fn proxies_scale_and_mezzanines_keep_rotation() {
        let (spec, plan) = plan(
            VideoPlan::Encode(encoding(VideoCodecTarget::H264High, Some(540), true)),
            AudioPlan::Encode(AudioEncoding::Aac { bitrate_kbps: 160 }),
            Container::Mp4,
        );
        let args = joined(&transcode_args(
            &spec,
            &plan,
            None,
            Path::new("proxy.part.mp4"),
        ));
        assert!(
            args.contains(
                "-noautorotate -i in.mov -map 0:v:0 -vf scale=w='if(gte(iw,ih),-2,min(540,iw))'"
            ),
            "{args}"
        );
        assert!(args.contains("-map 0:a:0 -c:a aac -b:a 160k"), "{args}");
        assert!(
            args.ends_with("-movflags +faststart proxy.part.mp4"),
            "{args}"
        );
    }

    #[test]
    fn copies_and_stills_need_no_encoder() {
        let (spec, copy) = plan(
            VideoPlan::Copy,
            AudioPlan::Encode(AudioEncoding::Flac),
            Container::Mp4,
        );
        let args = joined(&transcode_args(
            &spec,
            &copy,
            Some(HardwareEncoder::Nvenc),
            Path::new("m.mp4"),
        ));
        assert!(
            args.contains("-map 0:v:0 -c:v copy -map 0:a:0 -c:a flac"),
            "{args}"
        );
        assert!(!args.contains("nvenc"), "{args}");
        let (spec, still) = plan(VideoPlan::Still, AudioPlan::Drop, Container::Png);
        let args = joined(&transcode_args(&spec, &still, None, Path::new("m.png")));
        assert!(args.contains("-frames:v 1 -c:v png -an"), "{args}");
        assert!(!args.contains("faststart"), "{args}");
    }
}
