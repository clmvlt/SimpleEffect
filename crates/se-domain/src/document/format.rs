//! [`Format`]: the frame size and rates of a sequence, and its presets.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{spatial, time::FrameRate};

/// Sample rate of new sequences, in hertz.
pub const DEFAULT_SAMPLE_RATE: u32 = 48_000;

/// Frame size, frame rate and audio sample rate of a sequence.
///
/// Validation requires even dimensions between 2 and 16384 (video encoders need them) and a
/// sample rate whose sample lasts a whole number of flicks (44.1, 48, 96 kHz…).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Format {
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Frames per second.
    pub frame_rate: FrameRate,
    /// Audio samples per second.
    pub sample_rate: u32,
}

impl Format {
    /// The format of `preset` at `frame_rate` and the default sample rate; `None` for
    /// [`FormatPreset::Custom`].
    #[must_use]
    pub const fn from_preset(preset: FormatPreset, frame_rate: FrameRate) -> Option<Self> {
        match preset.dimensions() {
            Some((width, height)) => Some(Self {
                width,
                height,
                frame_rate,
                sample_rate: DEFAULT_SAMPLE_RATE,
            }),
            None => None,
        }
    }

    /// The preset whose dimensions match, or [`FormatPreset::Custom`].
    #[must_use]
    pub fn preset(&self) -> FormatPreset {
        FormatPreset::NAMED
            .into_iter()
            .find(|preset| preset.dimensions() == Some((self.width, self.height)))
            .unwrap_or(FormatPreset::Custom)
    }

    /// Real pixels per design pixel in this format (ADR-0015): shorter side / 1080.
    #[must_use]
    pub fn design_scale(&self) -> f64 {
        spatial::design_scale(self.width, self.height)
    }
}

/// Common sequence formats, named by aspect ratio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum FormatPreset {
    /// 16:9, 1920 × 1080.
    Hd16x9,
    /// 16:9, 3840 × 2160.
    Uhd16x9,
    /// 9:16, 1080 × 1920 (Shorts, Reels, `TikTok`).
    Vertical9x16,
    /// 1:1, 1080 × 1080.
    Square1x1,
    /// 4:5, 1080 × 1350.
    Portrait4x5,
    /// Any other size.
    Custom,
}

impl FormatPreset {
    /// Every preset with fixed dimensions.
    pub const NAMED: [Self; 5] = [
        Self::Hd16x9,
        Self::Uhd16x9,
        Self::Vertical9x16,
        Self::Square1x1,
        Self::Portrait4x5,
    ];

    /// Width and height in pixels; `None` for [`FormatPreset::Custom`].
    #[must_use]
    pub const fn dimensions(self) -> Option<(u32, u32)> {
        match self {
            Self::Hd16x9 => Some((1920, 1080)),
            Self::Uhd16x9 => Some((3840, 2160)),
            Self::Vertical9x16 => Some((1080, 1920)),
            Self::Square1x1 => Some((1080, 1080)),
            Self::Portrait4x5 => Some((1080, 1350)),
            Self::Custom => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_round_trip_through_formats() {
        for preset in FormatPreset::NAMED {
            let format = Format::from_preset(preset, FrameRate::FPS_30).unwrap();
            assert_eq!(format.preset(), preset);
            assert_eq!(format.sample_rate, DEFAULT_SAMPLE_RATE);
        }
        assert_eq!(
            Format::from_preset(FormatPreset::Custom, FrameRate::FPS_30),
            None
        );
    }

    #[test]
    fn other_sizes_are_custom() {
        let format = Format {
            width: 1280,
            height: 720,
            frame_rate: FrameRate::FPS_25,
            sample_rate: DEFAULT_SAMPLE_RATE,
        };
        assert_eq!(format.preset(), FormatPreset::Custom);
        assert!((format.design_scale() - 720.0 / 1080.0).abs() < 1e-12);
    }
}
