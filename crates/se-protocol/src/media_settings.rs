//! Media pipeline DTOs beyond import: settings, this machine's decoders and encoders, derived
//! media requests and the project cache (`docs/architecture.md` §7).

use se_domain::{AssetId, ProjectId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Media settings of the app (stored in `library.db`, edited in the settings screen).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MediaSettings {
    /// Quality of new mezzanines as an x264/x265 CRF (lower is better and larger):
    /// [`MediaSettings::MEZZANINE_QUALITY_RANGE`], default 14 (visually lossless).
    pub mezzanine_quality: u8,
    /// Height (short side) of new proxies.
    pub proxy_resolution: ProxyResolution,
    /// Whether proxies are generated automatically for footage larger than 1080p or that
    /// WebCodecs cannot decode (otherwise only on request).
    pub auto_proxies: bool,
    /// Folder holding ffmpeg and ffprobe, searched before the bundled ones; `null` uses the
    /// bundled tools.
    pub tools_dir: Option<String>,
}

impl MediaSettings {
    /// Accepted mezzanine qualities (CRF).
    pub const MEZZANINE_QUALITY_RANGE: std::ops::RangeInclusive<u8> = 8..=28;
    /// Default mezzanine quality (CRF).
    pub const DEFAULT_MEZZANINE_QUALITY: u8 = 14;
}

impl Default for MediaSettings {
    fn default() -> Self {
        Self {
            mezzanine_quality: Self::DEFAULT_MEZZANINE_QUALITY,
            proxy_resolution: ProxyResolution::P540,
            auto_proxies: true,
            tools_dir: None,
        }
    }
}

/// Size of proxies: their shorter side in pixels (never larger than the original).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, TS)]
pub enum ProxyResolution {
    /// 540 pixels (960 × 540 for 16:9): lightest to decode.
    #[default]
    #[serde(rename = "540p")]
    P540,
    /// 720 pixels (1280 × 720 for 16:9).
    #[serde(rename = "720p")]
    P720,
}

impl ProxyResolution {
    /// The shorter side in pixels.
    #[must_use]
    pub const fn short_side(self) -> u32 {
        match self {
            Self::P540 => 540,
            Self::P720 => 720,
        }
    }
}

/// What this machine's WebCodecs decodes, reported by the renderer (`isConfigSupported`) once per
/// connection. Decides the codecs of new mezzanines and proxies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent yes/no answers of the browser, one per codec"
)]
pub struct DecoderCapabilities {
    /// HEVC Main 10 (4:2:0, 10 bits) up to 4K: mezzanines of 10-bit footage keep their depth.
    pub hevc_main10: bool,
    /// H.264 High (4:2:0, 8 bits) up to 4K: the default mezzanine and proxy codec.
    pub h264_high: bool,
    /// AAC-LC: the sound of mezzanines and proxies; FLAC otherwise.
    pub aac: bool,
    /// FLAC.
    pub flac: bool,
}

/// The ffmpeg tools found and what they can do, for the settings screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MediaToolsStatus {
    /// Path of the ffmpeg in use, `null` when none was found.
    pub ffmpeg_path: Option<String>,
    /// Path of the ffprobe in use, `null` when none was found.
    pub ffprobe_path: Option<String>,
    /// ffmpeg's version (`9.0.2`…), when it could be read.
    pub ffmpeg_version: Option<String>,
    /// Hardware encoders that passed a test encode (`nvenc`, `videotoolbox`, `vaapi`…), in the
    /// order they are tried; empty when encoding is software only.
    pub hardware_encoders: Vec<String>,
    /// Why a tool is missing, when one is.
    pub problem: Option<String>,
}

/// Which derived copy of an asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum DerivedKind {
    /// The high-quality decodable copy of an original WebCodecs cannot decode.
    Mezzanine,
    /// The low-resolution copy for smooth playback.
    Proxy,
}

/// Arguments of [`crate::Command::GenerateDerivedMedia`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct GenerateDerivedRequest {
    /// The open project.
    pub project_id: ProjectId,
    /// The asset.
    pub asset_id: AssetId,
    /// What to generate.
    pub derived: DerivedKind,
}

/// The cache folder of a project and its size.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCacheInfo {
    /// Absolute path of the folder (`<project>/cache`).
    pub path: String,
    /// Bytes used by its files.
    pub size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_serialize_with_readable_resolutions() {
        let json = serde_json::to_string(&MediaSettings::default()).unwrap();
        assert_eq!(
            json,
            r#"{"mezzanineQuality":14,"proxyResolution":"540p","autoProxies":true,"toolsDir":null}"#
        );
        assert_eq!(ProxyResolution::P720.short_side(), 720);
    }
}
