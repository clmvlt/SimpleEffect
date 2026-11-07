//! [`ApplyAnimationPreset`]: expands an animation preset into keyframes of an item.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Edit, Effect, checks::check_revision, lookup::find_item, placement::place_item};
use crate::{
    AnimationPhase, DomainError, Flicks, ItemId, PresetPlacement, Project, animation_presets,
    expand_preset,
};

/// Applies the animation preset `preset` (an id of the library, `assets/animation-presets.json`)
/// to an item as an entrance, an exit or an emphasis lasting `duration` (shortened to the item):
/// the preset becomes ordinary keyframes of the properties it animates, which stay editable
/// (`crate::animation_preset`). Inverse: [`super::RestoreItem`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ApplyAnimationPreset {
    /// The item animated.
    pub item: ItemId,
    /// Id of the preset (`fade`, `slideFromLeft`, `typewriter`, `bounce`...).
    pub preset: String,
    /// Entrance, exit or emphasis.
    pub phase: AnimationPhase,
    /// How long the animation lasts.
    pub duration: Flicks,
    /// Where it starts, relative to the item's start; `null` for the phase's place (the start
    /// for an entrance, the end for an exit, the middle for an emphasis). Clamped into the item.
    pub start: Option<Flicks>,
    /// Revision the caller read, if it wants the change refused when the item changed since.
    pub expected_revision: Option<u64>,
}

impl Edit for ApplyAnimationPreset {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let library = animation_presets()?;
        let preset =
            library
                .find(&self.preset)
                .ok_or_else(|| DomainError::UnknownAnimationPreset {
                    preset: self.preset.clone(),
                    known: library.ids(),
                })?;
        let location = find_item(project, self.item)?;
        let item = location.get(project);
        check_revision(item, self.expected_revision)?;
        let placement = PresetPlacement {
            phase: self.phase,
            duration: self.duration,
            start: self.start,
        };
        let animations = expand_preset(
            preset,
            placement,
            &item.kind,
            item.timeline_range.duration(),
        )
        .map_err(|refusal| refusal.into_error(preset, self.phase, self.item, item.kind.label()))?;
        let mut updated = item.clone();
        for animation in &animations {
            if !animation.apply(&mut updated.kind) {
                return Err(DomainError::PropertyNotAvailable {
                    item: self.item,
                    item_kind: item.kind.label(),
                    property: animation.property(),
                });
            }
        }
        place_item(project, location, location.track, updated, effect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Animatable, DomainErrorCode, ItemKind,
        operation::testing::{apply_err, apply_ok, example, find},
        test_support::millis,
    };

    fn apply_preset(item: u64, preset: &str, phase: AnimationPhase) -> ApplyAnimationPreset {
        ApplyAnimationPreset {
            item: ItemId::from_test_index(item),
            preset: preset.to_owned(),
            phase,
            duration: millis(500),
            start: None,
            expected_revision: None,
        }
    }

    #[test]
    fn writes_the_preset_as_editable_keyframes() {
        // Shape 6 lasts 2 s and is static.
        let applied = apply_ok(
            &example(),
            apply_preset(6, "slideFromLeft", AnimationPhase::In),
        );
        let shape = find(&applied.project, 6);
        let transform = shape.kind.transform().unwrap();
        let Animatable::Keyframed { keyframes } = &transform.position else {
            panic!("the position should be keyframed");
        };
        assert_eq!(keyframes.len(), 2);
        assert_eq!(keyframes[1].time, millis(500));
        assert!((keyframes[0].value.x - 0.4).abs() < 1e-9);
        assert!(transform.opacity.is_keyframed());
    }

    #[test]
    fn types_out_a_text() {
        let applied = apply_ok(
            &example(),
            apply_preset(3, "typewriter", AnimationPhase::In),
        );
        let ItemKind::Text(text) = &find(&applied.project, 3).kind else {
            panic!("expected a text");
        };
        assert!(text.reveal.is_keyframed());
    }

    #[test]
    fn refuses_unknown_presets_and_items_without_the_property() {
        let error = apply_err(&example(), apply_preset(3, "wobble", AnimationPhase::In));
        assert_eq!(error.code(), DomainErrorCode::UnknownAnimationPreset);
        assert!(
            error
                .to_string()
                .contains("use one of: fade, slideFromLeft")
        );
        let error = apply_err(
            &example(),
            apply_preset(6, "typewriter", AnimationPhase::In),
        );
        assert_eq!(error.code(), DomainErrorCode::WrongItemKind);
        assert_eq!(
            error.to_string(),
            "preset 'typewriter' animates reveal, which item 00000000-0006-7000-8000-000000000000 \
             (a shape) does not have; choose a preset without it or apply it to another item"
        );
        let error = apply_err(&example(), apply_preset(3, "pulse", AnimationPhase::Out));
        assert_eq!(error.code(), DomainErrorCode::PresetPhaseNotSupported);
    }
}
