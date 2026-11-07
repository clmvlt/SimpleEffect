//! What a mezzanine or proxy job writes: container, picture and sound treatment. Decided by
//! [`crate::media_policy`], turned into ffmpeg arguments by se-media (which also picks a hardware
//! or software encoder for the target codec).

use std::path::PathBuf;

use se_protocol::{DerivedKind, JobKind};

/// A transcode job: which derived copy, where it goes, and how it is made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscodePlan {
    /// Mezzanine or proxy.
    pub derived: DerivedKind,
    /// The file to write (absolute). Written under a temporary name, then renamed.
    pub output: PathBuf,
    /// Container, picture and sound treatment.
    pub recipe: TranscodeRecipe,
}

impl TranscodePlan {
    /// The job kind of this plan.
    #[must_use]
    pub fn job_kind(&self) -> JobKind {
        Self::kind_of(self.derived)
    }

    /// The job kind making a `derived` copy.
    #[must_use]
    pub const fn kind_of(derived: DerivedKind) -> JobKind {
        match derived {
            DerivedKind::Mezzanine => JobKind::Mezzanine,
            DerivedKind::Proxy => JobKind::Proxy,
        }
    }
}

/// How to make a derived copy, independent of where it goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscodeRecipe {
    /// The file format.
    pub container: Container,
    /// What happens to the picture stream.
    pub video: VideoPlan,
    /// What happens to the first sound stream (other sound streams are dropped).
    pub audio: AudioPlan,
}

/// File format of a derived copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Container {
    /// MPEG-4 with picture (and sound).
    Mp4,
    /// MPEG-4 with sound only.
    M4a,
    /// One still picture (images ffmpeg reads but browsers do not: TIFF, HEIC…).
    Png,
}

impl Container {
    /// The file extension, without the dot.
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::M4a => "m4a",
            Self::Png => "png",
        }
    }
}

/// What happens to the picture stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoPlan {
    /// No picture in the output.
    Drop,
    /// The original stream, unchanged (only its container changes).
    Copy,
    /// Re-encoded.
    Encode(VideoEncoding),
    /// A still picture converted to PNG.
    Still,
}

/// How to re-encode a picture stream. Resolution (unless `max_short_side`), frame rate and
/// display rotation are kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoEncoding {
    /// Codec, profile and bit depth.
    pub codec: VideoCodecTarget,
    /// Quality as an x264/x265 CRF (lower is better); mapped to each hardware encoder's scale.
    pub quality: u8,
    /// Shorter side of the output in pixels; `None` keeps the size. Never enlarges.
    pub max_short_side: Option<u32>,
    /// Frames between keyframes (short GOPs make seeking fast).
    pub keyframe_interval: u32,
    /// Favour decoding speed over compression (no B-frames, `-tune fastdecode`).
    pub fast_decode: bool,
}

/// Target codec of a re-encoded picture stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodecTarget {
    /// HEVC Main 10, 4:2:0, 10 bits: keeps the depth of 10-bit footage.
    HevcMain10,
    /// H.264 High, 4:2:0, 8 bits: decoded everywhere.
    H264High,
}

/// What happens to the first sound stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioPlan {
    /// No sound in the output.
    Drop,
    /// The original stream, unchanged.
    Copy,
    /// Re-encoded.
    Encode(AudioEncoding),
}

/// Codec of a re-encoded sound stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioEncoding {
    /// AAC-LC at this bit rate.
    Aac {
        /// Kilobits per second.
        bitrate_kbps: u32,
    },
    /// FLAC (lossless), when the machine does not decode AAC.
    Flac,
}
