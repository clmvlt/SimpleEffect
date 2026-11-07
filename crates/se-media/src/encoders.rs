//! Hardware video encoders (NVENC, VideoToolbox, VAAPI): which ones work on this machine, found
//! once per ffmpeg by a tiny test encode and cached. Encoders an ffmpeg build lists may still
//! fail (no GPU, old driver), so listing them is not enough.

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use se_app::ports::{VideoCodecTarget, VideoEncoding};
use tokio::sync::Mutex;

use crate::{jobs::transcode_args::video_encoder_args, platform, process::Sidecar};

/// Longest a test encode may take (GPU initialisation included).
const TEST_ENCODE_TIMEOUT: Duration = Duration::from_secs(20);

/// A family of hardware encoders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareEncoder {
    /// NVIDIA (`h264_nvenc`, `hevc_nvenc`).
    Nvenc,
    /// Apple (`h264_videotoolbox`, `hevc_videotoolbox`).
    VideoToolbox,
    /// Linux VA-API (`h264_vaapi`, `hevc_vaapi`): Intel and AMD GPUs.
    Vaapi,
}

impl HardwareEncoder {
    /// Short name shown in the settings (`nvenc`, `videotoolbox`, `vaapi`).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Nvenc => "nvenc",
            Self::VideoToolbox => "videotoolbox",
            Self::Vaapi => "vaapi",
        }
    }

    /// The ffmpeg encoder of this family for `codec`.
    #[must_use]
    pub const fn encoder(self, codec: VideoCodecTarget) -> &'static str {
        match (self, codec) {
            (Self::Nvenc, VideoCodecTarget::H264High) => "h264_nvenc",
            (Self::Nvenc, VideoCodecTarget::HevcMain10) => "hevc_nvenc",
            (Self::VideoToolbox, VideoCodecTarget::H264High) => "h264_videotoolbox",
            (Self::VideoToolbox, VideoCodecTarget::HevcMain10) => "hevc_videotoolbox",
            (Self::Vaapi, VideoCodecTarget::H264High) => "h264_vaapi",
            (Self::Vaapi, VideoCodecTarget::HevcMain10) => "hevc_vaapi",
        }
    }
}

/// The hardware encoder that works for each target codec, if any.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EncoderSupport {
    /// For H.264 High 8 bits.
    pub h264: Option<HardwareEncoder>,
    /// For HEVC Main 10.
    pub hevc_main10: Option<HardwareEncoder>,
}

impl EncoderSupport {
    /// The hardware encoder for `codec`; `None` means software (libx264/libx265).
    #[must_use]
    pub const fn for_codec(self, codec: VideoCodecTarget) -> Option<HardwareEncoder> {
        match codec {
            VideoCodecTarget::H264High => self.h264,
            VideoCodecTarget::HevcMain10 => self.hevc_main10,
        }
    }

    /// Names of the families that work, without duplicates.
    #[must_use]
    pub fn names(self) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        for encoder in [self.h264, self.hevc_main10].into_iter().flatten() {
            if !names.iter().any(|name| name == encoder.name()) {
                names.push(encoder.name().to_owned());
            }
        }
        names
    }
}

/// The detection result for the ffmpeg last asked about (detected again when the tools change).
#[derive(Debug, Default)]
pub struct EncoderCache {
    detected: Mutex<Option<(PathBuf, EncoderSupport)>>,
}

impl EncoderCache {
    /// What works with the ffmpeg at `ffmpeg` (detected on first use; concurrent callers wait).
    pub async fn support(&self, ffmpeg: &Path) -> EncoderSupport {
        let mut detected = self.detected.lock().await;
        if let Some((path, support)) = detected.as_ref()
            && path == ffmpeg
        {
            return *support;
        }
        let support = detect(ffmpeg).await;
        tracing::info!(encoders = ?support.names(), "hardware encoders detected");
        *detected = Some((ffmpeg.to_path_buf(), support));
        support
    }
}

/// Tries this platform's candidates for each codec, in order.
async fn detect(ffmpeg: &Path) -> EncoderSupport {
    let mut support = EncoderSupport::default();
    for codec in [VideoCodecTarget::H264High, VideoCodecTarget::HevcMain10] {
        for encoder in platform::HARDWARE_ENCODER_CANDIDATES {
            if test_encode(ffmpeg, *encoder, codec).await {
                match codec {
                    VideoCodecTarget::H264High => support.h264 = Some(*encoder),
                    VideoCodecTarget::HevcMain10 => support.hevc_main10 = Some(*encoder),
                }
                break;
            }
        }
    }
    support
}

/// Whether five frames encode with `encoder` for `codec`, with the arguments jobs use.
async fn test_encode(ffmpeg: &Path, encoder: HardwareEncoder, codec: VideoCodecTarget) -> bool {
    let encoding = VideoEncoding {
        codec,
        quality: 23,
        max_short_side: None,
        keyframe_interval: 25,
        fast_decode: false,
    };
    let encoder_args = video_encoder_args(&encoding, Some(encoder));
    let mut args: Vec<String> = ["-hide_banner", "-nostdin", "-v", "error"]
        .map(str::to_owned)
        .to_vec();
    args.extend(encoder_args.input);
    args.extend(["-f", "lavfi", "-i", "color=c=black:s=256x256:r=25:d=0.2"].map(str::to_owned));
    if !encoder_args.filters.is_empty() {
        args.push("-vf".to_owned());
        args.push(encoder_args.filters.join(","));
    }
    args.extend(encoder_args.output);
    args.extend(["-frames:v", "5", "-f", "null", "-"].map(str::to_owned));
    let Ok(process) = Sidecar::spawn(ffmpeg, &args, None) else {
        return false;
    };
    matches!(
        tokio::time::timeout(TEST_ENCODE_TIMEOUT, process.finish()).await,
        Ok(Ok(()))
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn support_names_each_family_once() {
        let support = EncoderSupport {
            h264: Some(HardwareEncoder::Nvenc),
            hevc_main10: Some(HardwareEncoder::Nvenc),
        };
        assert_eq!(support.names(), ["nvenc"]);
        assert_eq!(
            support.for_codec(VideoCodecTarget::HevcMain10),
            Some(HardwareEncoder::Nvenc)
        );
        assert_eq!(EncoderSupport::default().names(), Vec::<String>::new());
        assert_eq!(
            HardwareEncoder::Vaapi.encoder(VideoCodecTarget::HevcMain10),
            "hevc_vaapi"
        );
    }
}
