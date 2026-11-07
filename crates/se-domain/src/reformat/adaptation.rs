//! How items adapt when their sequence takes another format: [`FormatAdaptation`] and
//! [`adapted_patch`].

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    Animatable, Assign, DesignPixels, Format, ItemKind, ItemPatch, MediaClipPatch, Reframe,
    TextItem, TextPatch,
};

/// Share of the frame's width a text may span once [`FormatAdaptation::fit_text_widths`] applied.
pub const TEXT_WIDTH_SHARE: f64 = 0.9;

/// What happens to the reframe mode of video clips when their sequence changes format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum ClipReframePolicy {
    /// Every clip keeps its mode.
    #[default]
    Keep,
    /// Every clip shows its whole picture ([`Reframe::Fit`]).
    Fit,
    /// Every clip covers the frame: clips already filling (or tracked) keep their focus, the
    /// others fill around the centre.
    Fill,
}

impl ClipReframePolicy {
    /// The mode a clip framed by `current` takes, or `None` when it stays.
    #[must_use]
    pub fn reframe_for(self, current: &Reframe) -> Option<Reframe> {
        match (self, current) {
            (Self::Keep, _)
            | (Self::Fit, Reframe::Fit)
            | (Self::Fill, Reframe::Fill { .. } | Reframe::Tracked { .. }) => None,
            (Self::Fit, _) => Some(Reframe::Fit),
            (Self::Fill, _) => Some(Reframe::fill()),
        }
    }
}

/// How the items of a sequence adapt to its new format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FormatAdaptation {
    /// The reframe mode of video clips.
    pub clip_reframe: ClipReframePolicy,
    /// Gives texts wider than [`TEXT_WIDTH_SHARE`] of the new frame (wrapping width times their
    /// horizontal scale; texts without wrapping count as too wide) that wrapping width, so long
    /// lines wrap instead of leaving a narrower frame.
    pub fit_text_widths: bool,
}

/// The change that adapts an item of `kind` on a track of the given kind to `format`, or `None`
/// when it stays as it is. Audio clips never change (their framing is not seen).
#[must_use]
pub fn adapted_patch(
    kind: &ItemKind,
    on_video_track: bool,
    format: &Format,
    adaptation: FormatAdaptation,
) -> Option<ItemPatch> {
    match kind {
        ItemKind::MediaClip(clip) if on_video_track => adaptation
            .clip_reframe
            .reframe_for(&clip.reframe)
            .map(|reframe| {
                ItemPatch::MediaClip(MediaClipPatch {
                    reframe: Some(reframe),
                    ..MediaClipPatch::default()
                })
            }),
        ItemKind::Text(text) if adaptation.fit_text_widths => {
            fitted_text_width(text, format).map(|width| {
                ItemPatch::Text(TextPatch {
                    max_width: Some(Assign { value: Some(width) }),
                    ..TextPatch::default()
                })
            })
        }
        _ => None,
    }
}

/// The wrapping width that keeps `text` within [`TEXT_WIDTH_SHARE`] of the frame's width (whole
/// design pixels), or `None` when it already does or is flattened to nothing.
fn fitted_text_width(text: &TextItem, format: &Format) -> Option<DesignPixels> {
    let widest_scale = largest_horizontal_scale(&text.transform.scale);
    if widest_scale <= 0.0 || !widest_scale.is_finite() {
        return None;
    }
    let frame_width = f64::from(format.width) / format.design_scale();
    let limit = (frame_width * TEXT_WIDTH_SHARE / widest_scale).floor();
    if limit < 1.0 {
        return None;
    }
    match text.style.max_width {
        Some(width) if width.0 <= limit => None,
        _ => Some(DesignPixels(limit)),
    }
}

/// The largest horizontal scale (absolute value: negative scales mirror) a transform reaches.
fn largest_horizontal_scale(scale: &Animatable<crate::Vec2>) -> f64 {
    scale
        .values()
        .into_iter()
        .map(|value| value.x.abs())
        .fold(0.0, f64::max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AssetId, FormatPreset, FrameRate, MediaClip, Vec2};

    fn vertical() -> Format {
        Format::from_preset(FormatPreset::Vertical9x16, FrameRate::FPS_30).unwrap()
    }

    fn clip(reframe: Reframe) -> ItemKind {
        let mut clip = MediaClip::new(AssetId::from_test_index(1));
        clip.reframe = reframe;
        ItemKind::MediaClip(clip)
    }

    fn reframe_of(patch: Option<ItemPatch>) -> Option<Reframe> {
        match patch {
            Some(ItemPatch::MediaClip(patch)) => patch.reframe,
            None => None,
            Some(other) => unreachable!("a clip gets a clip patch, got {other:?}"),
        }
    }

    #[test]
    fn clip_policies_change_only_what_differs() {
        let fill = FormatAdaptation {
            clip_reframe: ClipReframePolicy::Fill,
            fit_text_widths: false,
        };
        let focused = Reframe::Fill {
            focus: Animatable::constant(Vec2::new(0.2, 0.5)),
        };
        let patch =
            |kind: &ItemKind, adaptation| adapted_patch(kind, true, &vertical(), adaptation);
        assert_eq!(
            reframe_of(patch(&clip(Reframe::Fit), fill)),
            Some(Reframe::fill())
        );
        assert_eq!(
            reframe_of(patch(&clip(Reframe::Custom), fill)),
            Some(Reframe::fill())
        );
        assert_eq!(reframe_of(patch(&clip(focused.clone()), fill)), None);
        let fit = FormatAdaptation {
            clip_reframe: ClipReframePolicy::Fit,
            ..fill
        };
        assert_eq!(reframe_of(patch(&clip(focused), fit)), Some(Reframe::Fit));
        assert_eq!(reframe_of(patch(&clip(Reframe::Fit), fit)), None);
        let keep = FormatAdaptation::default();
        assert_eq!(reframe_of(patch(&clip(Reframe::Custom), keep)), None);
        assert_eq!(
            adapted_patch(&clip(Reframe::Fit), false, &vertical(), fill),
            None
        );
    }

    /// The wrapping width a text patch sets (texts always get one).
    fn width_of(patch: Option<ItemPatch>) -> Option<DesignPixels> {
        match patch {
            Some(ItemPatch::Text(patch)) => patch.max_width.and_then(|assign| assign.value),
            None => None,
            Some(other) => unreachable!("a text gets a text patch, got {other:?}"),
        }
    }

    #[test]
    fn texts_too_wide_for_the_frame_wrap_at_ninety_percent_of_it() {
        let adaptation = FormatAdaptation {
            clip_reframe: ClipReframePolicy::Keep,
            fit_text_widths: true,
        };
        let mut text = TextItem::new("Un titre assez long");
        text.style.max_width = Some(DesignPixels(1600.0));
        let adapt = |text: &TextItem| {
            width_of(adapted_patch(
                &ItemKind::Text(text.clone()),
                true,
                &vertical(),
                adaptation,
            ))
        };
        assert_eq!(adapt(&text), Some(DesignPixels(972.0)));
        text.style.max_width = None;
        assert_eq!(adapt(&text), Some(DesignPixels(972.0)));
        text.style.max_width = Some(DesignPixels(800.0));
        assert_eq!(adapt(&text), None);
        text.transform.scale = Animatable::constant(Vec2::new(-2.0, 1.0));
        assert_eq!(adapt(&text), Some(DesignPixels(486.0)));
        text.transform.scale = Animatable::constant(Vec2::new(0.0, 1.0));
        assert_eq!(adapt(&text), None);
        let off = FormatAdaptation::default();
        assert_eq!(
            adapted_patch(
                &ItemKind::Text(TextItem::new("Titre")),
                true,
                &vertical(),
                off
            ),
            None
        );
    }

    #[test]
    fn wide_frames_measure_in_design_pixels() {
        let uhd = Format::from_preset(FormatPreset::Uhd16x9, FrameRate::FPS_30).unwrap();
        let text = TextItem::new("Titre");
        let adaptation = FormatAdaptation {
            clip_reframe: ClipReframePolicy::Keep,
            fit_text_widths: true,
        };
        assert_eq!(
            width_of(adapted_patch(&ItemKind::Text(text), true, &uhd, adaptation)),
            Some(DesignPixels(1728.0))
        );
    }
}
