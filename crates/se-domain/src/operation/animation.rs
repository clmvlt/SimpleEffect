//! [`SetKeyframes`]: sets one animatable property of an item, typed by [`PropertyAnimation`].

use std::fmt;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Edit, Effect, checks::check_revision, lookup::find_item, placement::place_item};
use crate::{Animatable, DomainError, ItemId, ItemKind, Project, Vec2};

/// The animatable properties of items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum AnimatableProperty {
    /// `transform.position` of visual items.
    Position,
    /// `transform.scale` of visual items.
    Scale,
    /// `transform.rotation` of visual items.
    Rotation,
    /// `transform.anchor` of visual items.
    Anchor,
    /// `transform.opacity` of visual items.
    Opacity,
    /// `volumeDb` of media clips.
    VolumeDb,
}

impl fmt::Display for AnimatableProperty {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Position => "position",
            Self::Scale => "scale",
            Self::Rotation => "rotation",
            Self::Anchor => "anchor",
            Self::Opacity => "opacity",
            Self::VolumeDb => "volume",
        })
    }
}

/// An animatable property with its new value, typed by property (positions are [`Vec2`],
/// rotations numbers…). Serialized as `{"property": "opacity", "value": {"kind": "keyframed", …}}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "property", rename_all = "camelCase")]
pub enum PropertyAnimation {
    /// New position.
    Position {
        /// Static value or keyframes.
        value: Animatable<Vec2>,
    },
    /// New scale.
    Scale {
        /// Static value or keyframes.
        value: Animatable<Vec2>,
    },
    /// New rotation.
    Rotation {
        /// Static value or keyframes.
        value: Animatable<f64>,
    },
    /// New anchor.
    Anchor {
        /// Static value or keyframes.
        value: Animatable<Vec2>,
    },
    /// New opacity.
    Opacity {
        /// Static value or keyframes.
        value: Animatable<f64>,
    },
    /// New volume.
    VolumeDb {
        /// Static value or keyframes.
        value: Animatable<f64>,
    },
}

impl PropertyAnimation {
    /// The property set.
    #[must_use]
    pub const fn property(&self) -> AnimatableProperty {
        match self {
            Self::Position { .. } => AnimatableProperty::Position,
            Self::Scale { .. } => AnimatableProperty::Scale,
            Self::Rotation { .. } => AnimatableProperty::Rotation,
            Self::Anchor { .. } => AnimatableProperty::Anchor,
            Self::Opacity { .. } => AnimatableProperty::Opacity,
            Self::VolumeDb { .. } => AnimatableProperty::VolumeDb,
        }
    }

    /// Sets the property on `kind`; `false` (and no change) when the kind has no such property.
    fn apply(&self, kind: &mut ItemKind) -> bool {
        if let Self::VolumeDb { value } = self {
            return match kind {
                ItemKind::MediaClip(clip) => {
                    clip.volume_db = value.clone();
                    true
                }
                _ => false,
            };
        }
        let Some(transform) = kind.transform_mut() else {
            return false;
        };
        match self {
            Self::Position { value } => transform.position = value.clone(),
            Self::Scale { value } => transform.scale = value.clone(),
            Self::Rotation { value } => transform.rotation = value.clone(),
            Self::Anchor { value } => transform.anchor = value.clone(),
            Self::Opacity { value } => transform.opacity = value.clone(),
            Self::VolumeDb { .. } => return false,
        }
        true
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
    fn refuses_properties_the_item_does_not_have() {
        let volume = PropertyAnimation::VolumeDb {
            value: Animatable::constant(0.0),
        };
        let error = apply_err(&example(), set(3, volume));
        assert_eq!(
            error.to_string(),
            "item 00000000-0003-7000-8000-000000000000 is a text, which has no volume property"
        );
        let opacity = PropertyAnimation::Opacity {
            value: Animatable::constant(1.0),
        };
        assert!(matches!(
            apply_err(&example(), set(7, opacity)),
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
