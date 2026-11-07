//! [`MediaClip`]: an item playing part of an asset, and its [`Reframe`] mode (ADR-0026).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Animatable, FadeShape, Transform2D};
use crate::{
    ids::AssetId,
    spatial::Vec2,
    time::{Flicks, TimeRange},
};

/// Shows and/or plays part of a video or audio asset.
///
/// On a video track it shows the picture; on an audio track it plays the sound (a video asset's
/// sound included). Opacity lives in [`Transform2D::opacity`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MediaClip {
    /// The asset played.
    pub asset_id: AssetId,
    /// Time in the asset played at the item's first instant (≥ 0).
    pub source_start: Flicks,
    /// Playback speed (> 0): 2.0 plays twice as fast, so the clip uses twice as much source.
    pub speed: f64,
    /// Placement of the picture.
    pub transform: Transform2D,
    /// Gain in decibels (0 = unchanged, within [`Self::MIN_VOLUME_DB`]..=[`Self::MAX_VOLUME_DB`]);
    /// keyframes make the volume curve (automation).
    pub volume_db: Animatable<f64>,
    /// Fade from black/silence at the start (≥ 0; fades together fit in the item).
    pub fade_in: Flicks,
    /// Fade to black/silence at the end (≥ 0).
    pub fade_out: Flicks,
    /// Curve of the fade-in. Linear when absent, so documents written before schema version 4
    /// still read.
    #[serde(default)]
    pub fade_in_shape: FadeShape,
    /// Curve of the fade-out (linear when absent).
    #[serde(default)]
    pub fade_out_shape: FadeShape,
    /// How the picture fits the sequence frame before the transform applies.
    pub reframe: Reframe,
}

impl MediaClip {
    /// The quietest volume a clip may have (−96 dB: below the noise floor of 16-bit sound,
    /// heard as silence).
    pub const MIN_VOLUME_DB: f64 = -96.0;
    /// The loudest volume a clip may have.
    pub const MAX_VOLUME_DB: f64 = 24.0;

    /// A clip of `asset_id` from its start, at normal speed, centred and unfaded.
    #[must_use]
    pub fn new(asset_id: AssetId) -> Self {
        Self {
            asset_id,
            source_start: Flicks::ZERO,
            speed: 1.0,
            transform: Transform2D::default(),
            volume_db: Animatable::constant(0.0),
            fade_in: Flicks::ZERO,
            fade_out: Flicks::ZERO,
            fade_in_shape: FadeShape::Linear,
            fade_out_shape: FadeShape::Linear,
            reframe: Reframe::Fit,
        }
    }

    /// The part of the asset used when the item lasts `timeline_duration`: it starts at
    /// `source_start` and lasts `timeline_duration × speed` (rounded to the nearest flick).
    /// `None` if the speed is not a positive number or the range is not representable.
    #[must_use]
    pub fn source_range(&self, timeline_duration: Flicks) -> Option<TimeRange> {
        if self.speed.is_nan() || self.speed <= 0.0 {
            return None;
        }
        let duration = timeline_duration.checked_mul_f64(self.speed)?;
        TimeRange::new(self.source_start, duration).ok()
    }
}

/// How a clip's picture fits the sequence frame before its transform applies (ADR-0026).
///
/// The fill modes crop the picture to the frame's aspect around a **focus**: a point of the
/// upright picture, normalised in it (`(0, 0)` top-left, `(1, 1)` bottom-right), kept at the
/// frame's centre as far as the picture allows (the crop never leaves the picture). The focus
/// does not depend on the frame's aspect, so it survives format changes and versions in other
/// formats; it is the animatable property [`crate::AnimatableProperty::ReframeFocus`].
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Reframe {
    /// The whole picture is visible, as large as the frame allows; empty bands fill the rest
    /// (letterbox).
    #[default]
    Fit,
    /// The picture covers the frame and is cropped to it around `focus` (animatable: which part
    /// of the picture to keep over time).
    Fill {
        /// The point of the picture kept at the frame's centre (centre when absent, as in
        /// documents written before schema version 6).
        #[serde(default = "Reframe::centred")]
        focus: Animatable<Vec2>,
    },
    /// The picture at its natural size, one picture pixel per design pixel (ADR-0015), framed
    /// freely by the transform; it does not adapt to the frame.
    Custom,
    /// Like [`Reframe::Fill`], with a focus that follows the subject (keyframes written by
    /// automatic reframing).
    ///
    /// TODO(prompt-26): subject tracking writes these keyframes (and its settings).
    Tracked {
        /// The point of the picture kept at the frame's centre over time.
        focus: Animatable<Vec2>,
    },
}

impl Reframe {
    /// The centre of the picture, the default focus.
    pub const CENTRE: Vec2 = Vec2::new(0.5, 0.5);

    /// Fill around the centre of the picture.
    #[must_use]
    pub const fn fill() -> Self {
        Self::Fill {
            focus: Self::centred(),
        }
    }

    /// A static focus on the centre of the picture.
    #[must_use]
    pub const fn centred() -> Animatable<Vec2> {
        Animatable::constant(Self::CENTRE)
    }

    /// The focus of the fill modes, `None` for [`Reframe::Fit`] and [`Reframe::Custom`].
    #[must_use]
    pub const fn focus(&self) -> Option<&Animatable<Vec2>> {
        match self {
            Self::Fill { focus } | Self::Tracked { focus } => Some(focus),
            Self::Fit | Self::Custom => None,
        }
    }

    /// Mutable access to the focus of the fill modes.
    pub const fn focus_mut(&mut self) -> Option<&mut Animatable<Vec2>> {
        match self {
            Self::Fill { focus } | Self::Tracked { focus } => Some(focus),
            Self::Fit | Self::Custom => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(value: i64) -> Flicks {
        Flicks::new(value).unwrap()
    }

    #[test]
    fn source_range_scales_with_speed() {
        let mut clip = MediaClip::new(AssetId::from_test_index(1));
        clip.source_start = at(100);
        clip.speed = 2.0;
        assert_eq!(
            clip.source_range(at(50)),
            Some(TimeRange::new(at(100), at(100)).unwrap())
        );
        clip.speed = 0.0;
        assert_eq!(clip.source_range(at(50)), None);
        clip.speed = f64::NAN;
        assert_eq!(clip.source_range(at(50)), None);
    }

    #[test]
    fn source_range_is_none_when_unrepresentable() {
        let mut clip = MediaClip::new(AssetId::from_test_index(1));
        clip.source_start = Flicks::MAX;
        assert_eq!(clip.source_range(at(1)), None);
    }
}
