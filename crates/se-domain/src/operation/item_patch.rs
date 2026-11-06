//! [`ItemPatch`]: typed changes of an item's content, one patch type per item kind.
//!
//! Every field is optional: `null` (or absent) leaves the value unchanged. Timing goes through
//! [`super::MoveItem`] / [`super::TrimItem`], animatable properties through
//! [`super::SetKeyframes`], link groups through [`super::LinkItems`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::checks::swap_field;
use crate::{
    AssetId, Color, DesignPixels, Flicks, ItemKind, Reframe, ShapeGeometry, TemplateId,
    TemplateParamValue,
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

/// Changes of a [`crate::MediaClip`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
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
    /// Another way to fit the frame.
    pub reframe: Option<Reframe>,
}

/// Changes of a [`crate::TextItem`].
///
/// TODO(prompt-15): the style fields added to `TextStyle`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct TextPatch {
    /// Another text.
    pub content: Option<String>,
    /// Another font family.
    pub font_family: Option<String>,
    /// Another font size.
    pub font_size: Option<DesignPixels>,
    /// Another glyph colour.
    pub color: Option<Color>,
}

/// Changes of a [`crate::ShapeItem`].
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct ShapePatch {
    /// Another outline.
    pub geometry: Option<ShapeGeometry>,
    /// Another fill colour.
    pub fill: Option<Color>,
}

/// Changes of a [`crate::ImageItem`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct ImagePatch {
    /// Another image asset.
    pub asset_id: Option<AssetId>,
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
                swap_field(&mut clip.reframe, patch.reframe.as_ref());
            }
            (Self::Text(patch), ItemKind::Text(text)) => {
                swap_field(&mut text.content, patch.content.as_ref());
                swap_field(&mut text.style.font_family, patch.font_family.as_ref());
                swap_field(&mut text.style.font_size, patch.font_size.as_ref());
                swap_field(&mut text.style.color, patch.color.as_ref());
            }
            (Self::Shape(patch), ItemKind::Shape(shape)) => {
                swap_field(&mut shape.geometry, patch.geometry.as_ref());
                swap_field(&mut shape.fill, patch.fill.as_ref());
            }
            (Self::Image(patch), ItemKind::Image(image)) => {
                swap_field(&mut image.asset_id, patch.asset_id.as_ref());
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

#[cfg(test)]
mod tests {
    use super::*;

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
