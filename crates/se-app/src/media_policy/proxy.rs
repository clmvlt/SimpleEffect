//! The proxy decision: which assets get an automatic proxy, and the recipe of every proxy.

use se_domain::{Asset, AssetKind, StreamDecodability};
use se_protocol::{DecoderCapabilities, ProxyResolution};

use super::{ASSUMED_CAPABILITIES, PROXY_AAC_KBPS, pixel_layout_supported, sound_plan};
use crate::ports::{Container, TranscodeRecipe, VideoCodecTarget, VideoEncoding, VideoPlan};

/// Largest shorter side played without an automatic proxy (1080p).
const MAX_DIRECT_SHORT_SIDE: u32 = 1080;
/// Proxy quality (x264 CRF): small files, good enough to judge a cut.
const PROXY_QUALITY: u8 = 23;
/// Frames between proxy keyframes: very short GOPs make scrubbing smooth.
const PROXY_KEYFRAME_INTERVAL: u32 = 15;

/// Whether an automatic proxy is wanted: a video whose shorter side exceeds 1080 pixels, or whose
/// picture the renderer cannot play directly.
#[must_use]
pub fn auto_proxy_wanted(asset: &Asset) -> bool {
    let Some(stream) = asset
        .media_info
        .as_ref()
        .and_then(|info| info.video_stream.as_ref())
    else {
        return false;
    };
    asset.kind == AssetKind::Video
        && (stream.width.min(stream.height) > MAX_DIRECT_SHORT_SIDE
            || asset.decodability.video == StreamDecodability::Undecodable
            || !pixel_layout_supported(stream))
}

/// The recipe of a proxy of `asset` (any video, automatic or requested): H.264 High 8 bits at
/// `resolution` (never enlarged), fast-decode tuning, a keyframe every 15 frames, the first sound
/// stream copied or converted. `None` for assets without a picture to scrub (sound, images).
#[must_use]
pub fn proxy_recipe(
    asset: &Asset,
    capabilities: Option<DecoderCapabilities>,
    resolution: ProxyResolution,
) -> Option<TranscodeRecipe> {
    let info = asset.media_info.as_ref()?;
    if asset.kind != AssetKind::Video || info.video_stream.is_none() {
        return None;
    }
    let capabilities = capabilities.unwrap_or(ASSUMED_CAPABILITIES);
    Some(TranscodeRecipe {
        container: Container::Mp4,
        video: VideoPlan::Encode(VideoEncoding {
            codec: VideoCodecTarget::H264High,
            quality: PROXY_QUALITY,
            max_short_side: Some(resolution.short_side()),
            keyframe_interval: PROXY_KEYFRAME_INTERVAL,
            fast_decode: true,
        }),
        audio: sound_plan(
            info.audio_streams.first(),
            asset.decodability.audio,
            capabilities,
            PROXY_AAC_KBPS,
        ),
    })
}

#[cfg(test)]
mod tests {
    use se_domain::StreamDecodability::{Decodable, Undecodable, Unknown};

    use super::*;
    use crate::{
        media_policy::testing::{reported, video},
        ports::{AudioEncoding, AudioPlan},
    };

    #[test]
    fn large_or_undecodable_footage_gets_an_automatic_proxy() {
        let hd = reported(
            video("h264", 8, "4:2:0", (1920, 1080), "aac"),
            Decodable,
            Decodable,
        );
        assert!(!auto_proxy_wanted(&hd));
        let uhd = reported(
            video("h264", 8, "4:2:0", (3840, 2160), "aac"),
            Decodable,
            Decodable,
        );
        assert!(auto_proxy_wanted(&uhd));
        let portrait_uhd = reported(
            video("hevc", 8, "4:2:0", (2160, 3840), "aac"),
            Decodable,
            Decodable,
        );
        assert!(auto_proxy_wanted(&portrait_uhd));
        let prores = reported(
            video("prores", 10, "4:2:2", (1920, 1080), "pcm_s16le"),
            Undecodable,
            Decodable,
        );
        assert!(auto_proxy_wanted(&prores));
        let unchecked = reported(
            video("h264", 8, "4:2:0", (1280, 720), "aac"),
            Unknown,
            Unknown,
        );
        assert!(!auto_proxy_wanted(&unchecked));
    }

    #[test]
    fn proxies_are_light_h264_with_the_chosen_height() {
        let prores = reported(
            video("prores", 10, "4:2:2", (3840, 2160), "pcm_s24le"),
            Undecodable,
            Decodable,
        );
        let recipe = proxy_recipe(&prores, None, ProxyResolution::P720).unwrap();
        assert_eq!(
            recipe,
            TranscodeRecipe {
                container: Container::Mp4,
                video: VideoPlan::Encode(VideoEncoding {
                    codec: VideoCodecTarget::H264High,
                    quality: 23,
                    max_short_side: Some(720),
                    keyframe_interval: 15,
                    fast_decode: true,
                }),
                audio: AudioPlan::Encode(AudioEncoding::Aac { bitrate_kbps: 160 }),
            }
        );
        let h264 = reported(
            video("h264", 8, "4:2:0", (3840, 2160), "aac"),
            Decodable,
            Decodable,
        );
        assert_eq!(
            proxy_recipe(&h264, None, ProxyResolution::P540)
                .unwrap()
                .audio,
            AudioPlan::Copy
        );
    }

    #[test]
    fn sound_files_have_no_proxy() {
        let mut sound = reported(
            video("h264", 8, "4:2:0", (1, 1), "aac"),
            Decodable,
            Decodable,
        );
        sound.kind = AssetKind::Audio;
        assert!(!auto_proxy_wanted(&sound));
        assert_eq!(proxy_recipe(&sound, None, ProxyResolution::P540), None);
    }
}
