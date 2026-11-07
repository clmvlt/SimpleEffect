//! [`AnimatableProperty`]: the properties of items that keyframes can drive, and typed access to
//! them on an [`ItemKind`] ([`PropertyRef`], [`PropertyMut`]).
//!
//! Every animatable property of every item kind is listed here once: `SetKeyframes`, animation
//! presets and validation all go through [`ItemKind::property`] / [`ItemKind::property_mut`], so
//! a new property needs a variant, its slot below and its range (and a `PropertyAnimation`
//! variant in `operation/animation.rs`).

use std::{fmt, ops::RangeInclusive};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Animatable, ItemKind, MediaClip};
use crate::{Color, DesignPixels, Vec2};

/// The animatable properties of items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum AnimatableProperty {
    /// `transform.position` of visual items (normalised in the frame).
    Position,
    /// `transform.scale` of visual items (relative).
    Scale,
    /// `transform.rotation` of visual items (clockwise degrees).
    Rotation,
    /// `transform.anchor` of visual items (normalised in the item's box).
    Anchor,
    /// `transform.opacity` of visual items (0 to 1).
    Opacity,
    /// `transform.blur` of visual items (design pixels).
    Blur,
    /// `volumeDb` of media clips.
    VolumeDb,
    /// `reframe.focus` of media clips that fill the frame (a point of the picture, normalised in
    /// it; ADR-0026).
    ReframeFocus,
    /// `style.fontSize` of texts (design pixels).
    FontSize,
    /// `style.color` of texts, `fill` of shapes.
    FillColor,
    /// `style.letterSpacing` of texts (design pixels).
    LetterSpacing,
    /// `reveal` of texts (share of characters shown, 0 to 1).
    Reveal,
    /// Outline colour of texts and shapes (`stroke.color`).
    StrokeColor,
    /// Outline width of texts and shapes (`stroke.width`, design pixels).
    StrokeWidth,
    /// `style.shadow.color` of texts.
    ShadowColor,
    /// `style.shadow.blur` of texts (design pixels).
    ShadowBlur,
    /// `style.shadow.offset` of texts (design pixels).
    ShadowOffset,
    /// `style.background.color` of texts.
    BackgroundColor,
}

impl AnimatableProperty {
    /// Every property, in inspector order.
    pub const ALL: [Self; 18] = [
        Self::Position,
        Self::Scale,
        Self::Rotation,
        Self::Anchor,
        Self::Opacity,
        Self::Blur,
        Self::VolumeDb,
        Self::ReframeFocus,
        Self::FontSize,
        Self::FillColor,
        Self::LetterSpacing,
        Self::Reveal,
        Self::StrokeColor,
        Self::StrokeWidth,
        Self::ShadowColor,
        Self::ShadowBlur,
        Self::ShadowOffset,
        Self::BackgroundColor,
    ];

    /// The JSON name of the property (`fontSize`…), as in messages and the protocol.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Position => "position",
            Self::Scale => "scale",
            Self::Rotation => "rotation",
            Self::Anchor => "anchor",
            Self::Opacity => "opacity",
            Self::Blur => "blur",
            Self::VolumeDb => "volumeDb",
            Self::ReframeFocus => "reframeFocus",
            Self::FontSize => "fontSize",
            Self::FillColor => "fillColor",
            Self::LetterSpacing => "letterSpacing",
            Self::Reveal => "reveal",
            Self::StrokeColor => "strokeColor",
            Self::StrokeWidth => "strokeWidth",
            Self::ShadowColor => "shadowColor",
            Self::ShadowBlur => "shadowBlur",
            Self::ShadowOffset => "shadowOffset",
            Self::BackgroundColor => "backgroundColor",
        }
    }

    /// Values a numeric property accepts (each component of a pair; NaN never does), and how to
    /// say so; `None` for properties without limits (positions, scales, rotations, colours…).
    #[must_use]
    pub fn allowed_range(self) -> Option<(RangeInclusive<f64>, &'static str)> {
        const NON_NEGATIVE: RangeInclusive<f64> = 0.0..=f64::MAX;
        match self {
            Self::Opacity => Some((0.0..=1.0, "between 0 (invisible) and 1 (opaque)")),
            Self::Reveal => Some((0.0..=1.0, "between 0 (nothing shown) and 1 (all shown)")),
            Self::ReframeFocus => Some((
                0.0..=1.0,
                "between 0 and 1 on each axis (a point of the picture)",
            )),
            Self::VolumeDb => Some((
                MediaClip::MIN_VOLUME_DB..=MediaClip::MAX_VOLUME_DB,
                "between -96 dB and 24 dB",
            )),
            Self::FontSize => Some((f64::MIN_POSITIVE..=f64::MAX, "a size above 0 px")),
            Self::Blur | Self::StrokeWidth | Self::ShadowBlur => {
                Some((NON_NEGATIVE, "0 px or more"))
            }
            Self::LetterSpacing => Some((f64::MIN..=f64::MAX, "a finite number of pixels")),
            Self::Position
            | Self::Scale
            | Self::Rotation
            | Self::Anchor
            | Self::FillColor
            | Self::StrokeColor
            | Self::ShadowColor
            | Self::ShadowOffset
            | Self::BackgroundColor => None,
        }
    }
}

impl fmt::Display for AnimatableProperty {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

/// Read access to one animatable property, typed by its values.
#[derive(Debug, Clone, Copy)]
pub enum PropertyRef<'a> {
    /// A plain number (opacity, rotation, decibels, reveal).
    Number(&'a Animatable<f64>),
    /// A length in design pixels.
    Pixels(&'a Animatable<DesignPixels>),
    /// A pair of numbers (position, scale, anchor, shadow offset, reframe focus).
    Point(&'a Animatable<Vec2>),
    /// A colour.
    Color(&'a Animatable<Color>),
}

/// Write access to one animatable property, typed by its values.
#[derive(Debug)]
pub enum PropertyMut<'a> {
    /// A plain number.
    Number(&'a mut Animatable<f64>),
    /// A length in design pixels.
    Pixels(&'a mut Animatable<DesignPixels>),
    /// A pair of numbers.
    Point(&'a mut Animatable<Vec2>),
    /// A colour.
    Color(&'a mut Animatable<Color>),
}

impl PropertyRef<'_> {
    /// Whether the property has keyframes.
    #[must_use]
    pub const fn is_keyframed(&self) -> bool {
        match self {
            Self::Number(property) => property.is_keyframed(),
            Self::Pixels(property) => property.is_keyframed(),
            Self::Point(property) => property.is_keyframed(),
            Self::Color(property) => property.is_keyframed(),
        }
    }
}

impl ItemKind {
    /// The property of this item, or `None` when its kind has no such property (a shape has no
    /// font size, a text no volume…).
    #[must_use]
    pub fn property(&self, property: AnimatableProperty) -> Option<PropertyRef<'_>> {
        use AnimatableProperty as P;
        if let Some(transform) = self.transform() {
            match property {
                P::Position => return Some(PropertyRef::Point(&transform.position)),
                P::Scale => return Some(PropertyRef::Point(&transform.scale)),
                P::Rotation => return Some(PropertyRef::Number(&transform.rotation)),
                P::Anchor => return Some(PropertyRef::Point(&transform.anchor)),
                P::Opacity => return Some(PropertyRef::Number(&transform.opacity)),
                P::Blur => return Some(PropertyRef::Pixels(&transform.blur)),
                _ => {}
            }
        }
        match (self, property) {
            (Self::MediaClip(clip), P::VolumeDb) => Some(PropertyRef::Number(&clip.volume_db)),
            (Self::MediaClip(clip), P::ReframeFocus) => {
                clip.reframe.focus().map(PropertyRef::Point)
            }
            (Self::Text(text), _) => {
                let style = &text.style;
                Some(match property {
                    P::FontSize => PropertyRef::Pixels(&style.font_size),
                    P::FillColor => PropertyRef::Color(&style.color),
                    P::LetterSpacing => PropertyRef::Pixels(&style.letter_spacing),
                    P::Reveal => PropertyRef::Number(&text.reveal),
                    P::StrokeColor => PropertyRef::Color(&style.stroke.color),
                    P::StrokeWidth => PropertyRef::Pixels(&style.stroke.width),
                    P::ShadowColor => PropertyRef::Color(&style.shadow.color),
                    P::ShadowBlur => PropertyRef::Pixels(&style.shadow.blur),
                    P::ShadowOffset => PropertyRef::Point(&style.shadow.offset),
                    P::BackgroundColor => PropertyRef::Color(&style.background.color),
                    _ => return None,
                })
            }
            (Self::Shape(shape), P::FillColor) => Some(PropertyRef::Color(&shape.fill)),
            (Self::Shape(shape), P::StrokeColor) => Some(PropertyRef::Color(&shape.stroke.color)),
            (Self::Shape(shape), P::StrokeWidth) => Some(PropertyRef::Pixels(&shape.stroke.width)),
            _ => None,
        }
    }

    /// Mutable access to a property (see [`ItemKind::property`]).
    pub fn property_mut(&mut self, property: AnimatableProperty) -> Option<PropertyMut<'_>> {
        use AnimatableProperty as P;
        let transform_property = matches!(
            property,
            P::Position | P::Scale | P::Rotation | P::Anchor | P::Opacity | P::Blur
        );
        if transform_property {
            let transform = self.transform_mut()?;
            return Some(match property {
                P::Position => PropertyMut::Point(&mut transform.position),
                P::Scale => PropertyMut::Point(&mut transform.scale),
                P::Rotation => PropertyMut::Number(&mut transform.rotation),
                P::Anchor => PropertyMut::Point(&mut transform.anchor),
                P::Opacity => PropertyMut::Number(&mut transform.opacity),
                _ => PropertyMut::Pixels(&mut transform.blur),
            });
        }
        match (self, property) {
            (Self::MediaClip(clip), P::VolumeDb) => Some(PropertyMut::Number(&mut clip.volume_db)),
            (Self::MediaClip(clip), P::ReframeFocus) => {
                clip.reframe.focus_mut().map(PropertyMut::Point)
            }
            (Self::Text(text), _) => text_property_mut(text, property),
            (Self::Shape(shape), P::FillColor) => Some(PropertyMut::Color(&mut shape.fill)),
            (Self::Shape(shape), P::StrokeColor) => {
                Some(PropertyMut::Color(&mut shape.stroke.color))
            }
            (Self::Shape(shape), P::StrokeWidth) => {
                Some(PropertyMut::Pixels(&mut shape.stroke.width))
            }
            _ => None,
        }
    }
}

fn text_property_mut(
    text: &mut super::TextItem,
    property: AnimatableProperty,
) -> Option<PropertyMut<'_>> {
    use AnimatableProperty as P;
    let style = &mut text.style;
    Some(match property {
        P::FontSize => PropertyMut::Pixels(&mut style.font_size),
        P::FillColor => PropertyMut::Color(&mut style.color),
        P::LetterSpacing => PropertyMut::Pixels(&mut style.letter_spacing),
        P::Reveal => PropertyMut::Number(&mut text.reveal),
        P::StrokeColor => PropertyMut::Color(&mut style.stroke.color),
        P::StrokeWidth => PropertyMut::Pixels(&mut style.stroke.width),
        P::ShadowColor => PropertyMut::Color(&mut style.shadow.color),
        P::ShadowBlur => PropertyMut::Pixels(&mut style.shadow.blur),
        P::ShadowOffset => PropertyMut::Point(&mut style.shadow.offset),
        P::BackgroundColor => PropertyMut::Color(&mut style.background.color),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AssetId, Reframe, ShapeItem, ShapeKind, TextItem};

    #[test]
    fn reads_and_writes_reach_the_same_property() {
        let mut text = ItemKind::Text(TextItem::new("Titre"));
        for property in AnimatableProperty::ALL {
            let read = text.property(property).is_some();
            let write = text.property_mut(property).is_some();
            assert_eq!(read, write, "{property}");
            let clip_only = matches!(
                property,
                AnimatableProperty::VolumeDb | AnimatableProperty::ReframeFocus
            );
            assert_eq!(read, !clip_only, "{property}");
        }
        let shape = ItemKind::Shape(ShapeItem::new(ShapeKind::Ellipse));
        let shape_properties: Vec<_> = AnimatableProperty::ALL
            .into_iter()
            .filter(|property| shape.property(*property).is_some())
            .collect();
        assert_eq!(shape_properties.len(), 9);
        assert!(shape.property(AnimatableProperty::FontSize).is_none());
    }

    #[test]
    fn only_clips_that_fill_the_frame_have_a_focus() {
        let mut clip = MediaClip::new(AssetId::from_test_index(1));
        let focus = AnimatableProperty::ReframeFocus;
        for (reframe, has_focus) in [
            (Reframe::Fit, false),
            (Reframe::Custom, false),
            (Reframe::fill(), true),
            (
                Reframe::Tracked {
                    focus: Reframe::centred(),
                },
                true,
            ),
        ] {
            clip.reframe = reframe;
            let mut kind = ItemKind::MediaClip(clip.clone());
            assert_eq!(kind.property(focus).is_some(), has_focus);
            assert_eq!(kind.property_mut(focus).is_some(), has_focus);
        }
    }

    #[test]
    fn names_match_the_json_names() {
        for property in AnimatableProperty::ALL {
            let json = serde_json::to_string(&property).unwrap();
            assert_eq!(json, format!("\"{}\"", property.name()));
        }
    }
}
