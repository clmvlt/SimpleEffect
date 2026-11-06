//! [`MediaInfo`]: what probing a media file revealed.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::time::{Flicks, FrameRate};

/// Facts about a media file, filled from `ffprobe` (prompt 08).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    /// Playable length; `None` for still images and when the container does not say.
    pub duration: Option<Flicks>,
    /// Container format as reported by the prober (e.g. `mov,mp4,m4a,3gp,3g2,mj2`).
    pub container: String,
    /// The main picture stream, if any.
    pub video_stream: Option<VideoStreamInfo>,
    /// Sound streams, in file order.
    pub audio_streams: Vec<AudioStreamInfo>,
}

/// The main picture stream of a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct VideoStreamInfo {
    /// Codec name (e.g. `h264`, `hevc`, `prores`).
    pub codec: String,
    /// Codec profile (e.g. `High`, `Main 10`), when known.
    pub profile: Option<String>,
    /// Bits per colour component (8, 10, 12), when known.
    pub bit_depth: Option<u8>,
    /// Chroma subsampling (`4:2:0`, `4:2:2`, `4:4:4`), when known.
    pub chroma_subsampling: Option<String>,
    /// Coded width in pixels, before rotation.
    pub width: u32,
    /// Coded height in pixels, before rotation.
    pub height: u32,
    /// Nominal frame rate; `None` if unknown or not exactly representable in flicks (variable
    /// frame rate footage is conformed through a mezzanine).
    pub frame_rate: Option<FrameRate>,
    /// Clockwise display rotation in degrees: 0, 90, 180 or 270 (phone footage).
    pub rotation: u16,
}

/// One sound stream of a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AudioStreamInfo {
    /// Codec name (e.g. `aac`, `pcm_s24le`).
    pub codec: String,
    /// Number of channels (1 mono, 2 stereo…).
    pub channels: u16,
    /// Samples per second (e.g. 48000).
    pub sample_rate: u32,
}
