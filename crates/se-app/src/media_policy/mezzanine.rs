//! The mezzanine decision (`MezzaninePolicy`): whether an asset needs a decodable copy, and how
//! to make it.

use se_domain::{Asset, AssetKind, MediaInfo, StreamDecodability, VideoStreamInfo};
use se_protocol::DecoderCapabilities;

use super::{
    ASSUMED_CAPABILITIES, MEZZANINE_AAC_KBPS, one_second_of_frames, pixel_layout_supported,
    sound_plan,
};
use crate::ports::{
    AudioPlan, Container, TranscodeRecipe, VideoCodecTarget, VideoEncoding, VideoPlan,
};

/// Picture codecs an MP4 file can carry unchanged (when only the sound needs converting).
const MP4_VIDEO_CODECS: [&str; 4] = ["h264", "hevc", "av1", "vp9"];

/// Whether an asset needs a mezzanine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MezzanineDecision {
    /// A stream still waits for the renderer's decodability check: decide later.
    Waiting,
    /// The original plays as it is (or the asset was never probed).
    NotNeeded,
    /// A mezzanine made with this recipe.
    Needed(TranscodeRecipe),
}

/// Decides the mezzanine of `asset`: needed when the renderer cannot decode its picture (codec,
/// or a bit depth/chroma layout outside 4:2:0 8–10 bits) or its first sound stream.
///
/// The picture becomes HEVC Main 10 when the source has more than 8 bits and this machine decodes
/// HEVC Main 10, H.264 High otherwise; a decodable picture is copied when only the sound needs
/// work. `capabilities` are the renderer's report ([`ASSUMED_CAPABILITIES`] until it arrives);
/// `quality` is the CRF of the settings.
#[must_use]
pub fn mezzanine_decision(
    asset: &Asset,
    capabilities: Option<DecoderCapabilities>,
    quality: u8,
) -> MezzanineDecision {
    let Some(info) = &asset.media_info else {
        return MezzanineDecision::NotNeeded;
    };
    if asset.decodability.is_pending() {
        return MezzanineDecision::Waiting;
    }
    let capabilities = capabilities.unwrap_or(ASSUMED_CAPABILITIES);
    match (asset.kind, &info.video_stream) {
        (AssetKind::Image, _) => still_decision(asset),
        (AssetKind::Video, Some(stream)) => {
            video_decision(asset, info, stream, capabilities, quality)
        }
        (AssetKind::Video | AssetKind::Audio, _) => sound_decision(asset, info, capabilities),
    }
}

fn video_decision(
    asset: &Asset,
    info: &MediaInfo,
    stream: &VideoStreamInfo,
    capabilities: DecoderCapabilities,
    quality: u8,
) -> MezzanineDecision {
    let picture_needs_work = asset.decodability.video == StreamDecodability::Undecodable
        || !pixel_layout_supported(stream);
    let sound_needs_work = asset.decodability.audio == StreamDecodability::Undecodable;
    if !picture_needs_work && !sound_needs_work {
        return MezzanineDecision::NotNeeded;
    }
    let video = if picture_needs_work || !MP4_VIDEO_CODECS.contains(&stream.codec.as_str()) {
        VideoPlan::Encode(VideoEncoding {
            codec: target_codec(stream, capabilities),
            quality,
            max_short_side: None,
            keyframe_interval: one_second_of_frames(stream.frame_rate),
            fast_decode: false,
        })
    } else {
        VideoPlan::Copy
    };
    MezzanineDecision::Needed(TranscodeRecipe {
        container: Container::Mp4,
        video,
        audio: sound_plan(
            info.audio_streams.first(),
            asset.decodability.audio,
            capabilities,
            MEZZANINE_AAC_KBPS,
        ),
    })
}

fn sound_decision(
    asset: &Asset,
    info: &MediaInfo,
    capabilities: DecoderCapabilities,
) -> MezzanineDecision {
    if asset.decodability.audio != StreamDecodability::Undecodable {
        return MezzanineDecision::NotNeeded;
    }
    MezzanineDecision::Needed(TranscodeRecipe {
        container: Container::M4a,
        video: VideoPlan::Drop,
        audio: sound_plan(
            info.audio_streams.first(),
            asset.decodability.audio,
            capabilities,
            MEZZANINE_AAC_KBPS,
        ),
    })
}

fn still_decision(asset: &Asset) -> MezzanineDecision {
    if asset.decodability.video == StreamDecodability::Undecodable {
        MezzanineDecision::Needed(TranscodeRecipe {
            container: Container::Png,
            video: VideoPlan::Still,
            audio: AudioPlan::Drop,
        })
    } else {
        MezzanineDecision::NotNeeded
    }
}

/// HEVC Main 10 keeps the depth of footage above 8 bits when the machine decodes it; H.264 High
/// is faster to encode and decoded everywhere, and loses nothing on 8-bit footage.
fn target_codec(stream: &VideoStreamInfo, capabilities: DecoderCapabilities) -> VideoCodecTarget {
    if stream.bit_depth.is_some_and(|depth| depth > 8) && capabilities.hevc_main10 {
        VideoCodecTarget::HevcMain10
    } else {
        VideoCodecTarget::H264High
    }
}

#[cfg(test)]
mod tests {
    use se_domain::StreamDecodability::{Decodable, NoStream, Undecodable, Unknown};

    use super::*;
    use crate::{
        media_policy::testing::{reported, video},
        ports::AudioEncoding,
    };

    const HEVC: DecoderCapabilities = DecoderCapabilities {
        hevc_main10: true,
        h264_high: true,
        aac: true,
        flac: true,
    };
    const NO_HEVC: DecoderCapabilities = DecoderCapabilities {
        hevc_main10: false,
        ..HEVC
    };
    const AAC_320: AudioPlan = AudioPlan::Encode(AudioEncoding::Aac { bitrate_kbps: 320 });

    fn encoded(codec: VideoCodecTarget) -> VideoPlan {
        VideoPlan::Encode(VideoEncoding {
            codec,
            quality: 14,
            max_short_side: None,
            keyframe_interval: 25,
            fast_decode: false,
        })
    }

    fn needed(video: VideoPlan, audio: AudioPlan) -> MezzanineDecision {
        MezzanineDecision::Needed(TranscodeRecipe {
            container: Container::Mp4,
            video,
            audio,
        })
    }

    /// One row of the table: the asset, the machine, the expected decision.
    fn decide(asset: &Asset, capabilities: DecoderCapabilities) -> MezzanineDecision {
        mezzanine_decision(asset, Some(capabilities), 14)
    }

    #[test]
    fn decodable_h264_needs_nothing() {
        let asset = reported(
            video("h264", 8, "4:2:0", (1920, 1080), "aac"),
            Decodable,
            Decodable,
        );
        assert_eq!(decide(&asset, HEVC), MezzanineDecision::NotNeeded);
    }

    #[test]
    fn ten_bit_prores_becomes_hevc_main10_when_the_machine_decodes_it() {
        let prores = reported(
            video("prores", 10, "4:2:2", (1920, 1080), "pcm_s16le"),
            Undecodable,
            Decodable,
        );
        assert_eq!(
            decide(&prores, HEVC),
            needed(encoded(VideoCodecTarget::HevcMain10), AAC_320)
        );
        assert_eq!(
            decide(&prores, NO_HEVC),
            needed(encoded(VideoCodecTarget::H264High), AAC_320)
        );
    }

    #[test]
    fn eight_bit_sources_become_h264() {
        let dnxhd = reported(
            video("dnxhd", 8, "4:2:2", (1920, 1080), "pcm_s24le"),
            Undecodable,
            Decodable,
        );
        assert_eq!(
            decide(&dnxhd, HEVC),
            needed(encoded(VideoCodecTarget::H264High), AAC_320)
        );
    }

    #[test]
    fn unsupported_pixel_layouts_need_a_mezzanine_even_when_reported_decodable() {
        let h264_422 = reported(
            video("h264", 10, "4:2:2", (3840, 2160), "aac"),
            Decodable,
            Decodable,
        );
        assert_eq!(
            decide(&h264_422, HEVC),
            needed(encoded(VideoCodecTarget::HevcMain10), AudioPlan::Copy)
        );
        let twelve_bits = reported(
            video("hevc", 12, "4:2:0", (3840, 2160), "aac"),
            Decodable,
            Decodable,
        );
        assert!(matches!(
            decide(&twelve_bits, HEVC),
            MezzanineDecision::Needed(_)
        ));
    }

    #[test]
    fn only_the_sound_is_converted_when_the_picture_plays() {
        let ac3 = reported(
            video("h264", 8, "4:2:0", (1920, 1080), "ac3"),
            Decodable,
            Undecodable,
        );
        assert_eq!(decide(&ac3, HEVC), needed(VideoPlan::Copy, AAC_320));
    }

    #[test]
    fn decisions_wait_for_every_stream_check() {
        let asset = reported(
            video("prores", 10, "4:2:2", (1920, 1080), "pcm_s16le"),
            Undecodable,
            Unknown,
        );
        assert_eq!(decide(&asset, HEVC), MezzanineDecision::Waiting);
    }

    #[test]
    fn undecodable_sound_files_become_m4a_or_flac_without_aac() {
        let mut alac = reported(
            video("h264", 8, "4:2:0", (1, 1), "alac"),
            NoStream,
            Undecodable,
        );
        alac.kind = AssetKind::Audio;
        alac.media_info.as_mut().unwrap().video_stream = None;
        let expected = |audio| {
            MezzanineDecision::Needed(TranscodeRecipe {
                container: Container::M4a,
                video: VideoPlan::Drop,
                audio,
            })
        };
        assert_eq!(decide(&alac, HEVC), expected(AAC_320));
        let no_aac = DecoderCapabilities { aac: false, ..HEVC };
        assert_eq!(
            decide(&alac, no_aac),
            expected(AudioPlan::Encode(AudioEncoding::Flac))
        );
    }

    #[test]
    fn undecodable_stills_become_png() {
        let mut tiff = reported(
            video("tiff", 8, "4:4:4", (800, 600), "aac"),
            Undecodable,
            NoStream,
        );
        tiff.kind = AssetKind::Image;
        tiff.media_info.as_mut().unwrap().audio_streams.clear();
        let expected = MezzanineDecision::Needed(TranscodeRecipe {
            container: Container::Png,
            video: VideoPlan::Still,
            audio: AudioPlan::Drop,
        });
        assert_eq!(decide(&tiff, HEVC), expected);
        let png = reported(tiff, Decodable, NoStream);
        assert_eq!(decide(&png, HEVC), MezzanineDecision::NotNeeded);
    }

    #[test]
    fn unprobed_assets_need_nothing_and_unknown_machines_get_h264() {
        let mut asset = reported(
            video("prores", 10, "4:2:2", (1920, 1080), "aac"),
            Undecodable,
            Decodable,
        );
        assert_eq!(
            mezzanine_decision(&asset, None, 14),
            needed(encoded(VideoCodecTarget::H264High), AudioPlan::Copy)
        );
        asset.media_info = None;
        assert_eq!(decide(&asset, HEVC), MezzanineDecision::NotNeeded);
    }
}
