//! The subset of `ffprobe -print_format json -show_format -show_streams` output we read.
//! Every field is optional: ffprobe omits what it does not know.

use std::collections::BTreeMap;

use serde::Deserialize;

/// The whole document.
#[derive(Debug, Deserialize)]
pub struct ProbeOutput {
    /// Streams in file order.
    #[serde(default)]
    pub streams: Vec<Stream>,
    /// The container.
    pub format: Option<Container>,
}

/// One stream.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Stream {
    /// `video`, `audio`, `subtitle`, `data`…
    pub codec_type: Option<String>,
    /// `h264`, `hevc`, `prores`, `aac`…
    pub codec_name: Option<String>,
    /// `High`, `Main 10`, `LC`…
    pub profile: Option<String>,
    /// `avc1`, `hvc1`, or `[0][0][0][0]` when unset.
    pub codec_tag_string: Option<String>,
    /// Picture width.
    pub width: Option<u32>,
    /// Picture height.
    pub height: Option<u32>,
    /// `yuv420p`, `yuv422p10le`…
    pub pix_fmt: Option<String>,
    /// Codec level (`-99` when unknown).
    pub level: Option<i64>,
    /// Bits per sample of the coded picture, as a string.
    pub bits_per_raw_sample: Option<String>,
    /// Base frame rate (`30000/1001`).
    pub r_frame_rate: Option<String>,
    /// Average frame rate.
    pub avg_frame_rate: Option<String>,
    /// Audio sample rate, as a string.
    pub sample_rate: Option<String>,
    /// Audio channels.
    pub channels: Option<u16>,
    /// Stream duration in seconds, as a string.
    pub duration: Option<String>,
    /// Flags such as `attached_pic` (cover art).
    pub disposition: BTreeMap<String, serde_json::Value>,
    /// Metadata tags (`rotate` in old files).
    pub tags: BTreeMap<String, serde_json::Value>,
    /// Side data (the display matrix holds the rotation).
    pub side_data_list: Vec<SideData>,
}

impl Stream {
    /// Whether this stream is cover art attached to a sound file, not moving pictures.
    pub fn is_attached_picture(&self) -> bool {
        self.disposition
            .get("attached_pic")
            .and_then(serde_json::Value::as_i64)
            == Some(1)
    }
}

/// One side data entry.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct SideData {
    /// `Display Matrix`…
    pub side_data_type: Option<String>,
    /// Counter-clockwise rotation in degrees, for display matrices.
    pub rotation: Option<f64>,
}

/// The container.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Container {
    /// `mov,mp4,m4a,3gp,3g2,mj2`, `png_pipe`…
    pub format_name: Option<String>,
    /// Duration in seconds, as a string.
    pub duration: Option<String>,
}
