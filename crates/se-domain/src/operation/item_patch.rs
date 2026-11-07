//! [`ItemPatch`]: typed changes of an item's content, one patch type per item kind.
//!
//! Every field is optional: `null` (or absent) leaves the value unchanged. Timing goes through
//! [`super::MoveItem`] / [`super::TrimItem`], animatable properties through
//! [`super::SetKeyframes`], link groups through [`super::LinkItems`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Assign, checks::swap_field};
use crate::{
    Animatable, AssetId, Color, DesignPixels, FadeShape, Flicks, ImageFit, ItemKind, Reframe,
    ShapeGeometry, TemplateId, TemplateParamValue, TextAlign, TextCase, TextItem, Vec2,
};

/// A change of an item's content; its kind must match the item's.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ItemPatch {
    /// Changes a media clip.
    MediaClip(MediaClipPatch),
    /// Changes a text.
    Text(TextPatch),
    /// Changes a shape.
    Shape(ShapePatch),
    /// Changes an image item.
    Image(ImagePatch),
    /// Changes a template item.
    MotionTemplate(TemplatePatch),
    /// Changes a caption.
    Caption(CaptionPatch),
}

/// Changes of a [`crate::MediaClip`] (clients may omit unchanged fields).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(optional_fields)]
pub struct MediaClipPatch {
    /// Another video or audio asset.
    pub asset_id: Option<AssetId>,
    /// Another source start (a slip edit: the item stays, the media slides under it).
    pub source_start: Option<Flicks>,
    /// Another playback speed (> 0); the item keeps its duration and uses more or less media.
    pub speed: Option<f64>,
    /// Another fade-in length.
    pub fade_in: Option<Flicks>,
    /// Another fade-out length.
    pub fade_out: Option<Flicks>,
    /// Another fade-in curve.
    pub fade_in_shape: Option<FadeShape>,
    /// Another fade-out curve.
    pub fade_out_shape: Option<FadeShape>,
    /// Another way to fit the frame.
    pub reframe: Option<Reframe>,
}

/// Changes of a [`crate::TextItem`]: its content and the static parts of its style (animatable
/// ones go through [`super::SetKeyframes`]; `fontSize` and `color` here set a static value).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(optional_fields)]
pub struct TextPatch {
    /// Another text.
    pub content: Option<String>,
    /// Another font family.
    pub font_family: Option<String>,
    /// Another weight (1 to 1000).
    pub font_weight: Option<u16>,
    /// Italic or upright.
    pub italic: Option<bool>,
    /// A static font size (replaces keyframes).
    pub font_size: Option<DesignPixels>,
    /// A static glyph colour (replaces keyframes).
    pub color: Option<Color>,
    /// Another line height (multiple of the font size).
    pub line_height: Option<f64>,
    /// Another alignment.
    pub align: Option<TextAlign>,
    /// Another wrapping width, or `{"value": null}` for no wrapping.
    pub max_width: Option<Assign<Option<DesignPixels>>>,
    /// Another letter case.
    pub text_case: Option<TextCase>,
    /// Outline on or off.
    pub stroke_enabled: Option<bool>,
    /// Shadow on or off.
    pub shadow_enabled: Option<bool>,
    /// Background plate on or off.
    pub background_enabled: Option<bool>,
    /// Another plate padding (design pixels, x and y).
    pub background_padding: Option<Vec2>,
    /// Another plate corner radius.
    pub background_corner_radius: Option<DesignPixels>,
}

/// Changes of a [`crate::ShapeItem`] (its animatable colours and outline width go through
/// [`super::SetKeyframes`]; `fill` here sets a static value).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(optional_fields)]
pub struct ShapePatch {
    /// Another outline.
    pub geometry: Option<ShapeGeometry>,
    /// A static fill colour (replaces keyframes).
    pub fill: Option<Color>,
    /// Outline on or off.
    pub stroke_enabled: Option<bool>,
    /// Whether the box covers the whole frame.
    pub cover_frame: Option<bool>,
}

/// Changes of a [`crate::ImageItem`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(optional_fields)]
pub struct ImagePatch {
    /// Another image asset.
    pub asset_id: Option<AssetId>,
    /// Another natural box.
    pub fit: Option<ImageFit>,
}

/// Changes of a [`crate::TemplateItem`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct TemplatePatch {
    /// Another template.
    pub template_id: Option<TemplateId>,
    /// Parameter values to set (a value) or reset to the template default (`null`); parameters
    /// not listed are unchanged.
    pub params: BTreeMap<String, Option<TemplateParamValue>>,
}

/// Changes of a [`crate::CaptionItem`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct CaptionPatch {
    /// Another subtitle text.
    pub text: Option<String>,
}

impl ItemPatch {
    /// Human name of the patched kind, matching [`ItemKind::label`].
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::MediaClip(_) => "media clip",
            Self::Text(_) => "text",
            Self::Shape(_) => "shape",
            Self::Image(_) => "image",
            Self::MotionTemplate(_) => "motion template",
            Self::Caption(_) => "caption",
        }
    }

    /// Applies the patch to `kind`; `false` (and no change) when the kinds differ.
    pub(crate) fn apply(&self, kind: &mut ItemKind) -> bool {
        match (self, kind) {
            (Self::MediaClip(patch), ItemKind::MediaClip(clip)) => {
                swap_field(&mut clip.asset_id, patch.asset_id.as_ref());
                swap_field(&mut clip.source_start, patch.source_start.as_ref());
                swap_field(&mut clip.speed, patch.speed.as_ref());
                swap_field(&mut clip.fade_in, patch.fade_in.as_ref());
                swap_field(&mut clip.fade_out, patch.fade_out.as_ref());
                swap_field(&mut clip.fade_in_shape, patch.fade_in_shape.as_ref());
                swap_field(&mut clip.fade_out_shape, patch.fade_out_shape.as_ref());
                swap_field(&mut clip.reframe, patch.reframe.as_ref());
            }
            (Self::Text(patch), ItemKind::Text(text)) => patch.apply(text),
            (Self::Shape(patch), ItemKind::Shape(shape)) => {
                swap_field(&mut shape.geometry, patch.geometry.as_ref());
                if let Some(fill) = patch.fill {
                    shape.fill = Animatable::constant(fill);
                }
                swap_field(&mut shape.stroke.enabled, patch.stroke_enabled.as_ref());
                swap_field(&mut shape.cover_frame, patch.cover_frame.as_ref());
            }
            (Self::Image(patch), ItemKind::Image(image)) => {
                swap_field(&mut image.asset_id, patch.asset_id.as_ref());
                swap_field(&mut image.fit, patch.fit.as_ref());
            }
            (Self::MotionTemplate(patch), ItemKind::MotionTemplate(template)) => {
                swap_field(&mut template.template_id, patch.template_id.as_ref());
                for (name, value) in &patch.params {
                    match value {
                        Some(value) => template.params.insert(name.clone(), value.clone()),
                        None => template.params.remove(name),
                    };
                }
            }
            (Self::Caption(patch), ItemKind::Caption(caption)) => {
                swap_field(&mut caption.text, patch.text.as_ref());
            }
            _ => return false,
        }
        true
    }
}

impl TextPatch {
    fn apply(&self, text: &mut TextItem) {
        let style = &mut text.style;
        swap_field(&mut text.content, self.content.as_ref());
        swap_field(&mut style.font_family, self.font_family.as_ref());
        swap_field(&mut style.font_weight, self.font_weight.as_ref());
        swap_field(&mut style.italic, self.italic.as_ref());
        if let Some(size) = self.font_size {
            style.font_size = Animatable::constant(size);
        }
        if let Some(color) = self.color {
            style.color = Animatable::constant(color);
        }
        swap_field(&mut style.line_height, self.line_height.as_ref());
        swap_field(&mut style.align, self.align.as_ref());
        if let Some(Assign { value }) = self.max_width {
            style.max_width = value;
        }
        swap_field(&mut style.text_case, self.text_case.as_ref());
        swap_field(&mut style.stroke.enabled, self.stroke_enabled.as_ref());
        swap_field(&mut style.shadow.enabled, self.shadow_enabled.as_ref());
        swap_field(
            &mut style.background.enabled,
            self.background_enabled.as_ref(),
        );
        swap_field(
            &mut style.background.padding,
            self.background_padding.as_ref(),
        );
        swap_field(
            &mut style.background.corner_radius,
            self.background_corner_radius.as_ref(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clears_the_wrapping_width() {
        let mut text = TextItem::new("Long");
        text.style.max_width = Some(DesignPixels(800.0));
        let patch: TextPatch = serde_json::from_str(r#"{"maxWidth":{"value":null}}"#).unwrap();
        patch.apply(&mut text);
        assert_eq!(text.style.max_width, None);
        let keep: TextPatch = serde_json::from_str(r#"{"maxWidth":null}"#).unwrap();
        text.style.max_width = Some(DesignPixels(800.0));
        keep.apply(&mut text);
        assert_eq!(text.style.max_width, Some(DesignPixels(800.0)));
    }

    #[test]
    fn missing_fields_mean_unchanged() {
        let patch: ItemPatch = serde_json::from_str(r#"{"kind":"text","content":"Hi"}"#).unwrap();
        assert_eq!(
            patch,
            ItemPatch::Text(TextPatch {
                content: Some("Hi".to_owned()),
                ..TextPatch::default()
            })
        );
        let reset: ItemPatch =
            serde_json::from_str(r#"{"kind":"motionTemplate","params":{"title":null}}"#).unwrap();
        let ItemPatch::MotionTemplate(reset) = reset else {
            unreachable!()
        };
        assert_eq!(reset.params.get("title"), Some(&None));
    }
}
