//! [`MediaInfo`]: what probing a media file revealed.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::time::{Flicks, FrameRate};

/// Facts about a media file, filled from `ffprobe` by se-media.
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
    /// Codec level as the prober reports it (H.264 `41` for 4.1, HEVC `153` for 5.1, AV1
    /// `seq_level_idx`), when known. Needed to build WebCodecs codec strings.
    pub level: Option<u16>,
    /// Four-character codec tag of the container (`avc1`, `hvc1`, `hev1`…), when set.
    pub codec_tag: Option<String>,
    /// Pixel format as the prober names it (`yuv420p`, `yuv422p10le`…), when known.
    pub pixel_format: Option<String>,
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

impl VideoStreamInfo {
    /// Width and height of the upright picture (the coded size, swapped for quarter turns).
    #[must_use]
    pub const fn display_size(&self) -> (u32, u32) {
        if self.rotation % 180 == 90 {
            (self.height, self.width)
        } else {
            (self.width, self.height)
        }
    }

    /// Width over height of the upright picture; `None` for an empty size.
    #[must_use]
    pub fn display_aspect(&self) -> Option<f64> {
        let (width, height) = self.display_size();
        (width > 0 && height > 0).then(|| f64::from(width) / f64::from(height))
    }
}

/// One sound stream of a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AudioStreamInfo {
    /// Codec name (e.g. `aac`, `pcm_s24le`).
    pub codec: String,
    /// Codec profile (e.g. `LC`, `HE-AAC`), when known. Needed to build WebCodecs codec strings.
    pub profile: Option<String>,
    /// Number of channels (1 mono, 2 stereo…).
    pub channels: u16,
    /// Samples per second (e.g. 48000).
    pub sample_rate: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stream(width: u32, height: u32, rotation: u16) -> VideoStreamInfo {
        VideoStreamInfo {
            codec: "h264".to_owned(),
            profile: None,
            level: None,
            codec_tag: None,
            pixel_format: None,
            bit_depth: None,
            chroma_subsampling: None,
            width,
            height,
            frame_rate: None,
            rotation,
        }
    }

    #[test]
    fn quarter_turns_swap_the_display_size() {
        assert_eq!(stream(1920, 1080, 0).display_size(), (1920, 1080));
        assert_eq!(stream(1920, 1080, 90).display_size(), (1080, 1920));
        assert_eq!(stream(1920, 1080, 180).display_size(), (1920, 1080));
        assert_eq!(
            stream(1920, 1080, 270).display_aspect(),
            Some(1080.0 / 1920.0)
        );
        assert_eq!(stream(0, 1080, 0).display_aspect(), None);
    }
}
