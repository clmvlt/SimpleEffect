//! [`SetKeyframes`]: sets one animatable property of an item, typed by [`PropertyAnimation`].

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Edit, Effect, checks::check_revision, lookup::find_item, placement::place_item};
use crate::{
    Animatable, AnimatableProperty, Color, DesignPixels, DomainError, ItemId, ItemKind, Project,
    PropertyMut, PropertyRef, Vec2,
};

/// Declares [`PropertyAnimation`] with one variant per [`AnimatableProperty`] (same name), typed
/// by the property's values, and its conversions from and to [`PropertyRef`] / [`PropertyMut`].
macro_rules! property_animations {
    ($($variant:ident: $value:ty => $slot:ident;)*) => {
        /// An animatable property with its new value, typed by property (positions are
        /// [`Vec2`], rotations numbers, sizes design pixels, colours [`Color`]). Serialized as
        /// `{"property": "opacity", "value": {"kind": "keyframed", …}}`.
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
        #[serde(tag = "property", rename_all = "camelCase")]
        pub enum PropertyAnimation {
            $(
                #[doc = concat!("New value of `", stringify!($variant), "` (see [`AnimatableProperty`]).")]
                $variant {
                    /// Static value or keyframes.
                    value: Animatable<$value>,
                },
            )*
        }

        impl PropertyAnimation {
            /// The property set.
            #[must_use]
            pub const fn property(&self) -> AnimatableProperty {
                match self {
                    $(Self::$variant { .. } => AnimatableProperty::$variant,)*
                }
            }

            /// Whether the new value has keyframes (rather than a static value).
            #[must_use]
            pub const fn is_keyframed(&self) -> bool {
                match self {
                    $(Self::$variant { value } => value.is_keyframed(),)*
                }
            }

            /// The current value of `property` read through `slot`, as an animation (`None`
            /// when the slot does not hold that property's type).
            #[must_use]
            pub fn read(property: AnimatableProperty, slot: PropertyRef<'_>) -> Option<Self> {
                match (property, slot) {
                    $((AnimatableProperty::$variant, PropertyRef::$slot(value)) => {
                        Some(Self::$variant { value: value.clone() })
                    })*
                    _ => None,
                }
            }

            /// Writes the value into `slot`; `false` (and no change) when the slot holds another
            /// type.
            pub(crate) fn assign(&self, slot: PropertyMut<'_>) -> bool {
                match (self, slot) {
                    $((Self::$variant { value }, PropertyMut::$slot(target)) => {
                        *target = value.clone();
                        true
                    })*
                    _ => false,
                }
            }
        }
    };
}

property_animations! {
    Position: Vec2 => Point;
    Scale: Vec2 => Point;
    Rotation: f64 => Number;
    Anchor: Vec2 => Point;
    Opacity: f64 => Number;
    Blur: DesignPixels => Pixels;
    VolumeDb: f64 => Number;
    ReframeFocus: Vec2 => Point;
    FontSize: DesignPixels => Pixels;
    FillColor: Color => Color;
    LetterSpacing: DesignPixels => Pixels;
    Reveal: f64 => Number;
    StrokeColor: Color => Color;
    StrokeWidth: DesignPixels => Pixels;
    ShadowColor: Color => Color;
    ShadowBlur: DesignPixels => Pixels;
    ShadowOffset: Vec2 => Point;
    BackgroundColor: Color => Color;
}

impl PropertyAnimation {
    /// Sets the property on `kind`; `false` (and no change) when the kind has no such property.
    pub(crate) fn apply(&self, kind: &mut ItemKind) -> bool {
        kind.property_mut(self.property())
            .is_some_and(|slot| self.assign(slot))
    }
}

/// Replaces the keyframes (or the static value) of one animatable property of an item. Keyframe
/// times are relative to the item's start and must be sorted and within the item.
/// Inverse: [`super::RestoreItem`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SetKeyframes {
    /// The item animated.
    pub item: ItemId,
    /// The property and its new value.
    pub animation: PropertyAnimation,
    /// Revision the caller read, if it wants the change refused when the item changed since.
    pub expected_revision: Option<u64>,
}

impl Edit for SetKeyframes {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let location = find_item(project, self.item)?;
        let item = location.get(project);
        check_revision(item, self.expected_revision)?;
        let mut updated = item.clone();
        if !self.animation.apply(&mut updated.kind) {
            return Err(DomainError::PropertyNotAvailable {
                item: self.item,
                item_kind: item.kind.label(),
                property: self.animation.property(),
            });
        }
        place_item(project, location, location.track, updated, effect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Easing, Keyframe, ViolationCode,
        operation::testing::{apply_err, apply_ok, clip, example, find},
        test_support::millis,
    };

    fn keyframes(points: &[(i64, f64)]) -> Animatable<f64> {
        Animatable::Keyframed {
            keyframes: points
                .iter()
                .map(|&(time_ms, value)| Keyframe {
                    time: millis(time_ms),
                    value,
                    easing: Easing::Linear,
                })
                .collect(),
        }
    }

    fn set(item: u64, animation: PropertyAnimation) -> SetKeyframes {
        SetKeyframes {
            item: ItemId::from_test_index(item),
            animation,
            expected_revision: None,
        }
    }

    #[test]
    fn sets_transform_and_volume_keyframes() {
        let value = keyframes(&[(0, 0.0), (1000, 90.0)]);
        let rotation = PropertyAnimation::Rotation {
            value: value.clone(),
        };
        let applied = apply_ok(&example(), set(4, rotation));
        assert_eq!(
            find(&applied.project, 4).kind.transform().unwrap().rotation,
            value
        );
        let volume = PropertyAnimation::VolumeDb {
            value: Animatable::constant(-3.0),
        };
        let applied = apply_ok(&example(), set(9, volume));
        assert_eq!(
            clip(find(&applied.project, 9)).volume_db,
            Animatable::constant(-3.0)
        );
    }

    #[test]
    fn sets_text_and_shape_style_properties() {
        let size = PropertyAnimation::FontSize {
            value: Animatable::constant(DesignPixels(120.0)),
        };
        let applied = apply_ok(&example(), set(3, size));
        let ItemKind::Text(text) = &find(&applied.project, 3).kind else {
            panic!("expected a text");
        };
        assert_eq!(
            text.style.font_size,
            Animatable::constant(DesignPixels(120.0))
        );
        let fill = PropertyAnimation::FillColor {
            value: Animatable::constant(Color::WHITE),
        };
        let applied = apply_ok(&example(), set(6, fill));
        let ItemKind::Shape(shape) = &find(&applied.project, 6).kind else {
            panic!("expected a shape");
        };
        assert_eq!(shape.fill, Animatable::constant(Color::WHITE));
    }

    #[test]
    fn reads_back_what_it_wrote() {
        let project = example();
        let title = &find(&project, 3).kind;
        let opacity = PropertyAnimation::read(
            AnimatableProperty::Opacity,
            title.property(AnimatableProperty::Opacity).unwrap(),
        )
        .unwrap();
        let PropertyAnimation::Opacity { value } = &opacity else {
            panic!("expected an opacity");
        };
        assert!(value.is_keyframed());
        assert!(
            PropertyAnimation::read(
                AnimatableProperty::Rotation,
                title.property(AnimatableProperty::Opacity).unwrap()
            )
            .is_some(),
            "both are numbers"
        );
        assert!(
            PropertyAnimation::read(
                AnimatableProperty::FontSize,
                title.property(AnimatableProperty::Opacity).unwrap()
            )
            .is_none()
        );
    }

    #[test]
    fn refuses_properties_the_item_does_not_have() {
        let volume = PropertyAnimation::VolumeDb {
            value: Animatable::constant(0.0),
        };
        let error = apply_err(&example(), set(3, volume));
        assert_eq!(
            error.to_string(),
            "item 00000000-0003-7000-8000-000000000000 is a text, which has no volumeDb property"
        );
        let opacity = PropertyAnimation::Opacity {
            value: Animatable::constant(1.0),
        };
        assert!(matches!(
            apply_err(&example(), set(7, opacity)),
            DomainError::PropertyNotAvailable { .. }
        ));
        let size = PropertyAnimation::FontSize {
            value: Animatable::constant(DesignPixels(10.0)),
        };
        assert!(matches!(
            apply_err(&example(), set(6, size)),
            DomainError::PropertyNotAvailable { .. }
        ));
    }

    #[test]
    fn keyframes_must_be_sorted_and_inside_the_item() {
        // Image 4 lasts 2 s.
        for (points, code) in [
            (
                &[(1000, 0.0), (500, 1.0)][..],
                ViolationCode::KeyframesNotSorted,
            ),
            (
                &[(0, 0.0), (2500, 1.0)][..],
                ViolationCode::KeyframeOutsideItem,
            ),
        ] {
            let opacity = PropertyAnimation::Opacity {
                value: keyframes(points),
            };
            let DomainError::InvalidDocument { violations } =
                apply_err(&example(), set(4, opacity))
            else {
                panic!("expected a validation error");
            };
            assert_eq!(violations[0].code, code);
        }
    }
}
