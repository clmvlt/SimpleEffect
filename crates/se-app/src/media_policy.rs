//! Which derived copies an asset needs and how to make them (`docs/architecture.md` §7,
//! ADR-0019). Pure decisions from the asset's probed streams, the decodability the renderer
//! reported, this machine's decoders and the media settings; tested as decision tables.
//!
//! - [`mezzanine_decision`]: a high-quality copy when WebCodecs cannot decode the original (its
//!   codec, or its bit depth or chroma subsampling).
//! - [`proxy_recipe`]: a light H.264 copy for footage larger than 1080p or not decodable.

mod mezzanine;
mod proxy;

pub use self::{
    mezzanine::{MezzanineDecision, mezzanine_decision},
    proxy::{auto_proxy_wanted, proxy_recipe},
};
use se_domain::{AudioStreamInfo, FrameRate, StreamDecodability, VideoStreamInfo};
use se_protocol::DecoderCapabilities;

use crate::ports::{AudioEncoding, AudioPlan};

/// Bit rate of AAC sound in mezzanines (transparent).
pub const MEZZANINE_AAC_KBPS: u32 = 320;
/// Bit rate of AAC sound in proxies.
pub const PROXY_AAC_KBPS: u32 = 160;

/// Capabilities assumed before the renderer reported any: H.264 and AAC (Electron ships them),
/// no HEVC (only some GPUs decode it), FLAC (software in every Chromium).
pub const ASSUMED_CAPABILITIES: DecoderCapabilities = DecoderCapabilities {
    hevc_main10: false,
    h264_high: true,
    aac: true,
    flac: true,
};

/// Sound codecs an MP4 file can carry unchanged.
const MP4_AUDIO_CODECS: [&str; 4] = ["aac", "mp3", "opus", "flac"];

/// Whether the browser path renders this picture layout: 4:2:0 at 8 or 10 bits. Other layouts
/// (4:2:2, 4:4:4, 12 bits) may pass `isConfigSupported` yet decode slowly or not at all.
fn pixel_layout_supported(stream: &VideoStreamInfo) -> bool {
    let depth_ok = stream.bit_depth.is_none_or(|depth| depth <= 10);
    let chroma_ok = stream
        .chroma_subsampling
        .as_deref()
        .is_none_or(|chroma| chroma == "4:2:0");
    depth_ok && chroma_ok
}

/// Frames between keyframes for about one second of footage (30 when the rate is unknown).
fn one_second_of_frames(rate: Option<FrameRate>) -> u32 {
    rate.map_or(30, |rate| rate.nominal_fps().clamp(1, 120))
}

/// The sound of a derived copy: copied when the renderer decodes it and MP4 can carry it,
/// otherwise AAC (or FLAC when the machine has no AAC decoder).
fn sound_plan(
    stream: Option<&AudioStreamInfo>,
    decodability: StreamDecodability,
    capabilities: DecoderCapabilities,
    aac_kbps: u32,
) -> AudioPlan {
    let Some(stream) = stream else {
        return AudioPlan::Drop;
    };
    if decodability == StreamDecodability::Decodable
        && MP4_AUDIO_CODECS.contains(&stream.codec.as_str())
    {
        return AudioPlan::Copy;
    }
    AudioPlan::Encode(if capabilities.aac || !capabilities.flac {
        AudioEncoding::Aac {
            bitrate_kbps: aac_kbps,
        }
    } else {
        AudioEncoding::Flac
    })
}

#[cfg(test)]
mod testing {
    //! Assets for the decision tables.

    use se_domain::{
        Asset, AssetKind, AudioStreamInfo, Decodability, MediaInfo, StreamDecodability,
        VideoStreamInfo, test_support::example_project,
    };

    /// A video asset with this picture codec, depth, chroma and size, stereo `sound`.
    pub fn video(codec: &str, depth: u8, chroma: &str, size: (u32, u32), sound: &str) -> Asset {
        let mut asset = example_project().assets[0].clone();
        asset.kind = AssetKind::Video;
        asset.derived = se_domain::DerivedMedia::default();
        asset.media_info = Some(MediaInfo {
            duration: None,
            container: "mov".to_owned(),
            video_stream: Some(VideoStreamInfo {
                codec: codec.to_owned(),
                profile: None,
                level: None,
                codec_tag: None,
                pixel_format: None,
                bit_depth: Some(depth),
                chroma_subsampling: Some(chroma.to_owned()),
                width: size.0,
                height: size.1,
                frame_rate: Some(se_domain::FrameRate::FPS_25),
                rotation: 0,
            }),
            audio_streams: vec![AudioStreamInfo {
                codec: sound.to_owned(),
                profile: None,
                channels: 2,
                sample_rate: 48_000,
            }],
        });
        asset
    }

    /// `asset` with this reported decodability.
    pub fn reported(
        mut asset: Asset,
        video: StreamDecodability,
        audio: StreamDecodability,
    ) -> Asset {
        asset.decodability = Decodability { video, audio };
        asset
    }
}
