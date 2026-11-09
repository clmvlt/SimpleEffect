//! [`ItemKind`]: the content of an item.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{CaptionItem, ImageItem, MediaClip, ShapeItem, TemplateItem, TextItem, Transform2D};

/// What an item shows or plays. Serialized with its fields next to `"kind"`.
///
/// Only media clips may sit on audio tracks (playing the sound of their asset); every kind may
/// sit on a video track except a clip of an audio-only asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[allow(
    clippy::large_enum_variant,
    reason = "items live inline in their track's list and are cloned whole by every edit; a box               per text would add an allocation for no measurable gain (hundreds of items)"
)]
pub enum ItemKind {
    /// Part of a video or audio asset.
    MediaClip(MediaClip),
    /// Styled text.
    Text(TextItem),
    /// A vector shape.
    Shape(ShapeItem),
    /// A still image asset.
    Image(ImageItem),
    /// An instance of a motion template.
    MotionTemplate(TemplateItem),
    /// A subtitle built from transcript words.
    Caption(CaptionItem),
}

impl ItemKind {
    /// Human name of the kind, for messages (`media clip`, `text`…).
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

    /// The placement of a visual item; `None` for kinds that have none yet.
    #[must_use]
    pub const fn transform(&self) -> Option<&Transform2D> {
        match self {
            Self::MediaClip(clip) => Some(&clip.transform),
            Self::Text(text) => Some(&text.transform),
            Self::Shape(shape) => Some(&shape.transform),
            Self::Image(image) => Some(&image.transform),
            Self::MotionTemplate(template) => Some(&template.transform),
            Self::Caption(_) => None,
        }
    }

    /// Mutable access to the placement of a visual item; `None` for kinds that have none yet.
    pub const fn transform_mut(&mut self) -> Option<&mut Transform2D> {
        match self {
            Self::MediaClip(clip) => Some(&mut clip.transform),
            Self::Text(text) => Some(&mut text.transform),
            Self::Shape(shape) => Some(&mut shape.transform),
            Self::Image(image) => Some(&mut image.transform),
            Self::MotionTemplate(template) => Some(&mut template.transform),
            Self::Caption(_) => None,
        }
    }
}
