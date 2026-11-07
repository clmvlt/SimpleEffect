//! Animation presets: named animations (fade, slides, pop, typewriter, bounce…) defined as data
//! in `assets/animation-presets.json` and **expanded into ordinary keyframes** of an item, which
//! stay editable (golden rule 6). The UI and AI agents share this one implementation through the
//! `ApplyAnimationPreset` operation; the format is described in `crates/se-domain/CLAUDE.md`.

mod definition;
mod expansion;

use std::sync::LazyLock;

pub use definition::{
    AnimationPhase, AnimationPreset, AnimationPresetLibrary, PresetKeyframe, PresetNumber,
    PresetTrack, PresetValue,
};
pub use expansion::{PresetPlacement, PresetRefusal, expand_preset};

use crate::{DomainError, Flicks, ItemKind, PropertyAnimation, TextItem};

/// The library file, embedded in the binary.
const LIBRARY_JSON: &str = include_str!("../assets/animation-presets.json");

static LIBRARY: LazyLock<Result<AnimationPresetLibrary, String>> = LazyLock::new(|| {
    let library: AnimationPresetLibrary =
        serde_json::from_str(LIBRARY_JSON).map_err(|error| error.to_string())?;
    library.check()?;
    Ok(library)
});

/// The preset library.
///
/// # Errors
/// [`DomainError::PresetLibraryUnavailable`] if the embedded file is broken (a test guards
/// against it).
pub fn animation_presets() -> Result<&'static AnimationPresetLibrary, DomainError> {
    LIBRARY
        .as_ref()
        .map_err(|reason| DomainError::PresetLibraryUnavailable {
            reason: reason.clone(),
        })
}

/// What a preset does to a reference text (« Aa », centred, opaque) over one second, in each of
/// its phases, for previews: the same expansion as applying it to an item.
#[must_use]
pub fn preset_samples(preset: &AnimationPreset) -> Vec<(AnimationPhase, Vec<PropertyAnimation>)> {
    let sample = ItemKind::Text(TextItem::new("Aa"));
    preset
        .phases
        .iter()
        .filter_map(|&phase| {
            let placement = PresetPlacement {
                phase,
                duration: Flicks::one_second(),
                start: None,
            };
            expand_preset(preset, placement, &sample, Flicks::one_second())
                .ok()
                .map(|animations| (phase, animations))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Animatable, AnimatableProperty, ItemId, ShapeItem, ShapeKind, Vec2, test_support::millis,
    };

    fn preset(id: &str) -> &'static AnimationPreset {
        animation_presets().unwrap().find(id).unwrap()
    }

    fn placement(phase: AnimationPhase, duration_ms: i64) -> PresetPlacement {
        PresetPlacement {
            phase,
            duration: millis(duration_ms),
            start: None,
        }
    }

    fn text() -> ItemKind {
        ItemKind::Text(TextItem::new("Bonjour"))
    }

    #[test]
    fn the_embedded_library_is_valid_and_covers_the_roadmap() {
        let library = animation_presets().unwrap();
        for id in [
            "fade",
            "slideFromLeft",
            "slideFromRight",
            "slideFromTop",
            "slideFromBottom",
            "pop",
            "blur",
            "typewriter",
            "bounce",
            "shake",
            "pulse",
        ] {
            assert!(library.find(id).is_some(), "missing preset {id}");
        }
    }

    #[test]
    fn every_preset_expands_on_a_text_in_every_phase() {
        let mut expansions = Vec::new();
        for preset in &animation_presets().unwrap().presets {
            let samples = preset_samples(preset);
            assert_eq!(samples.len(), preset.phases.len(), "{}", preset.id);
            for (phase, animations) in samples {
                expansions.push((format!("{} {phase}", preset.id), animations));
            }
        }
        insta::assert_json_snapshot!("preset_samples", expansions);
    }

    #[test]
    fn an_entrance_reaches_the_item_value_and_an_exit_leaves_it() {
        let fade = preset("fade");
        let entrance = expand_preset(
            fade,
            placement(AnimationPhase::In, 500),
            &text(),
            millis(4000),
        )
        .unwrap();
        let PropertyAnimation::Opacity {
            value: Animatable::Keyframed { keyframes },
        } = &entrance[0]
        else {
            panic!("expected opacity keyframes");
        };
        let points: Vec<_> = keyframes.iter().map(|k| (k.time, k.value)).collect();
        assert_eq!(points, [(millis(0), 0.0), (millis(500), 1.0)]);

        let exit = expand_preset(
            fade,
            placement(AnimationPhase::Out, 500),
            &text(),
            millis(4000),
        )
        .unwrap();
        let PropertyAnimation::Opacity {
            value: Animatable::Keyframed { keyframes },
        } = &exit[0]
        else {
            panic!("expected opacity keyframes");
        };
        let points: Vec<_> = keyframes.iter().map(|k| (k.time, k.value)).collect();
        assert_eq!(points, [(millis(3500), 1.0), (millis(4000), 0.0)]);
        // The entrance eases out, so the exit eases in.
        assert_eq!(
            keyframes[0].easing,
            crate::Easing::Preset {
                name: crate::EasingPreset::EaseIn
            }
        );
    }

    #[test]
    fn exits_mirror_bezier_curves() {
        let pop = preset("pop");
        let exit = expand_preset(
            pop,
            placement(AnimationPhase::Out, 1000),
            &text(),
            millis(2000),
        )
        .unwrap();
        let PropertyAnimation::Scale {
            value: Animatable::Keyframed { keyframes },
        } = &exit[0]
        else {
            panic!("expected scale keyframes");
        };
        let crate::Easing::CubicBezier { x1, y1, x2, y2 } = keyframes[0].easing else {
            panic!("expected a Bézier curve");
        };
        let expected = [0.36, 0.0, 0.66, -0.56];
        for (actual, expected) in [x1, y1, x2, y2].into_iter().zip(expected) {
            assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
        }
        assert_eq!(keyframes[1].value, Vec2::splat(0.0));
    }

    #[test]
    fn keeps_keyframes_outside_the_span_and_animates_to_the_held_value() {
        let mut kind = text();
        if let Some(transform) = kind.transform_mut() {
            transform.opacity = Animatable::Keyframed {
                keyframes: vec![
                    crate::Keyframe {
                        time: millis(0),
                        value: 0.8,
                        easing: crate::Easing::Linear,
                    },
                    crate::Keyframe {
                        time: millis(3000),
                        value: 0.4,
                        easing: crate::Easing::Linear,
                    },
                ],
            };
        }
        let exit = expand_preset(
            preset("fade"),
            placement(AnimationPhase::Out, 500),
            &kind,
            millis(4000),
        )
        .unwrap();
        let PropertyAnimation::Opacity {
            value: Animatable::Keyframed { keyframes },
        } = &exit[0]
        else {
            panic!("expected opacity keyframes");
        };
        let points: Vec<_> = keyframes.iter().map(|k| (k.time, k.value)).collect();
        assert_eq!(
            points,
            [
                (millis(0), 0.8),
                (millis(3000), 0.4),
                (millis(3500), 0.4),
                (millis(4000), 0.0)
            ]
        );
    }

    #[test]
    fn emphasis_is_centred_or_placed_and_clamped_into_the_item() {
        let pulse = preset("pulse");
        let times = |placement: PresetPlacement| {
            let animations = expand_preset(pulse, placement, &text(), millis(3000)).unwrap();
            let PropertyAnimation::Scale {
                value: Animatable::Keyframed { keyframes },
            } = &animations[0]
            else {
                panic!("expected scale keyframes");
            };
            keyframes.iter().map(|k| k.time).collect::<Vec<_>>()
        };
        assert_eq!(
            times(placement(AnimationPhase::Emphasis, 1000)),
            [millis(1000), millis(1500), millis(2000)]
        );
        let late = PresetPlacement {
            start: Some(millis(2800)),
            ..placement(AnimationPhase::Emphasis, 1000)
        };
        assert_eq!(times(late), [millis(2000), millis(2500), millis(3000)]);
        let long = placement(AnimationPhase::Emphasis, 9000);
        assert_eq!(times(long), [millis(0), millis(1500), millis(3000)]);
    }

    #[test]
    fn refuses_unsupported_phases_and_missing_properties() {
        let bounce = preset("bounce");
        let refusal = expand_preset(
            bounce,
            placement(AnimationPhase::In, 500),
            &text(),
            millis(2000),
        )
        .unwrap_err();
        assert_eq!(refusal, PresetRefusal::PhaseNotSupported);
        let error = refusal.into_error(
            bounce,
            AnimationPhase::In,
            ItemId::from_test_index(1),
            "text",
        );
        assert_eq!(
            error.to_string(),
            "preset 'bounce' cannot be applied as in; it supports: emphasis"
        );
        let shape = ItemKind::Shape(ShapeItem::new(ShapeKind::Ellipse));
        let refusal = expand_preset(
            preset("typewriter"),
            placement(AnimationPhase::In, 500),
            &shape,
            millis(2000),
        )
        .unwrap_err();
        assert_eq!(
            refusal,
            PresetRefusal::MissingProperty(AnimatableProperty::Reveal)
        );
        let refusal = expand_preset(
            preset("fade"),
            placement(AnimationPhase::In, 0),
            &text(),
            millis(2000),
        )
        .unwrap_err();
        assert_eq!(refusal, PresetRefusal::EmptyDuration);
    }

    #[test]
    fn spans_shorter_than_their_keyframe_spacing_drop_colliding_keyframes() {
        let animations = expand_preset(
            preset("bounce"),
            placement(AnimationPhase::Emphasis, 1),
            &text(),
            Flicks::new(1).unwrap(),
        )
        .unwrap();
        let PropertyAnimation::Position {
            value: Animatable::Keyframed { keyframes },
        } = &animations[0]
        else {
            panic!("expected position keyframes");
        };
        assert!(keyframes.windows(2).all(|pair| pair[0].time < pair[1].time));
    }
}
