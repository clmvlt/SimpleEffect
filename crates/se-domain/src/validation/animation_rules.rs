//! Rules of animatable properties: `EMPTY_KEYFRAMES`, `KEYFRAMES_NOT_SORTED`,
//! `KEYFRAME_OUTSIDE_ITEM`, and `VALUE_OUT_OF_RANGE` for the values of numeric properties
//! (`AnimatableProperty::allowed_range`: opacity, volume, sizes…) and Bézier control points.

use super::{Path, Report, ViolationCode, report::seconds};
use crate::{Animatable, AnimatableProperty, Easing, Flicks, Item, ItemKind, PropertyRef};

/// Checks every animatable property of `item` (paths are relative to `kind_path`).
pub(super) fn check(item: &Item, kind_path: &Path, report: &mut Report) {
    let duration = item.timeline_range.duration();
    for property in AnimatableProperty::ALL {
        let Some(slot) = item.kind.property(property) else {
            continue;
        };
        let path = property_path(&item.kind, property, kind_path);
        match slot {
            PropertyRef::Number(values) => check_keyframes(values, &path, duration, report),
            PropertyRef::Pixels(values) => check_keyframes(values, &path, duration, report),
            PropertyRef::Point(values) => check_keyframes(values, &path, duration, report),
            PropertyRef::Color(values) => check_keyframes(values, &path, duration, report),
        }
        if let Some((allowed, expected)) = property.allowed_range() {
            for (value_path, value) in value_paths(slot, &path) {
                if !allowed.contains(&value) {
                    report.push(
                        ViolationCode::ValueOutOfRange,
                        &value_path,
                        format!("{property} must be {expected}, got {value}"),
                    );
                }
            }
        }
    }
}

/// Where `property` lives in an item of `kind` (JSON field names under `kind_path`).
fn property_path(kind: &ItemKind, property: AnimatableProperty, kind_path: &Path) -> Path {
    use AnimatableProperty as P;
    let in_style = matches!(kind, ItemKind::Text(_));
    let fields: &[&str] = match property {
        P::Position => &["transform", "position"],
        P::Scale => &["transform", "scale"],
        P::Rotation => &["transform", "rotation"],
        P::Anchor => &["transform", "anchor"],
        P::Opacity => &["transform", "opacity"],
        P::Blur => &["transform", "blur"],
        P::VolumeDb => &["volumeDb"],
        P::Reveal => &["reveal"],
        P::FontSize => &["style", "fontSize"],
        P::LetterSpacing => &["style", "letterSpacing"],
        P::ShadowColor => &["style", "shadow", "color"],
        P::ShadowBlur => &["style", "shadow", "blur"],
        P::ShadowOffset => &["style", "shadow", "offset"],
        P::BackgroundColor => &["style", "background", "color"],
        P::FillColor if in_style => &["style", "color"],
        P::FillColor => &["fill"],
        P::StrokeColor if in_style => &["style", "stroke", "color"],
        P::StrokeColor => &["stroke", "color"],
        P::StrokeWidth if in_style => &["style", "stroke", "width"],
        P::StrokeWidth => &["stroke", "width"],
    };
    fields
        .iter()
        .fold(kind_path.clone(), |path, field| path.field(field))
}

/// The numbers of a numeric property with the path of each (`value`, or `keyframes[i].value`).
fn value_paths(slot: PropertyRef<'_>, path: &Path) -> Vec<(Path, f64)> {
    let numbers = slot.numbers();
    if slot.is_keyframed() {
        numbers
            .into_iter()
            .enumerate()
            .map(|(index, value)| (path.field("keyframes").key(index).field("value"), value))
            .collect()
    } else {
        numbers
            .into_iter()
            .map(|value| (path.field("value"), value))
            .collect()
    }
}

/// Keyframes exist, have strictly increasing times within `[0, duration]` and valid easings.
fn check_keyframes<T>(
    property: &Animatable<T>,
    path: &Path,
    duration: Flicks,
    report: &mut Report,
) {
    let Animatable::Keyframed { keyframes } = property else {
        return;
    };
    let keyframes_path = path.field("keyframes");
    if keyframes.is_empty() {
        report.push(
            ViolationCode::EmptyKeyframes,
            &keyframes_path,
            "a keyframed property needs at least one keyframe; add one or make it static"
                .to_owned(),
        );
    }
    let mut previous_time: Option<Flicks> = None;
    for (index, keyframe) in keyframes.iter().enumerate() {
        let keyframe_path = keyframes_path.key(index);
        let time_path = keyframe_path.field("time");
        if previous_time.is_some_and(|previous| keyframe.time <= previous) {
            report.push(
                ViolationCode::KeyframesNotSorted,
                &time_path,
                format!(
                    "keyframe {index} at {} does not come after the previous keyframe; keyframe \
                     times must strictly increase",
                    seconds(keyframe.time)
                ),
            );
        }
        if keyframe.time.is_negative() || keyframe.time > duration {
            report.push(
                ViolationCode::KeyframeOutsideItem,
                &time_path,
                format!(
                    "keyframe {index} is at {} but the item lasts {}; keyframe times are relative \
                     to the item start and must be between 0s and its duration",
                    seconds(keyframe.time),
                    seconds(duration)
                ),
            );
        }
        check_easing(keyframe.easing, &keyframe_path.field("easing"), report);
        previous_time = Some(keyframe.time);
    }
}

/// A cubic Bézier's control-point times must stay in `[0, 1]` so time never runs backwards.
fn check_easing(easing: Easing, path: &Path, report: &mut Report) {
    if let Easing::CubicBezier { x1, x2, .. } = easing
        && !(is_unit(x1) && is_unit(x2))
    {
        report.push(
            ViolationCode::ValueOutOfRange,
            path,
            format!("cubic Bézier x1 and x2 must be between 0 and 1, got {x1} and {x2}"),
        );
    }
}

/// Whether `value` is in `[0, 1]` (false for NaN).
fn is_unit(value: f64) -> bool {
    (0.0..=1.0).contains(&value)
}

#[cfg(test)]
mod tests {
    use crate::{
        Animatable, Easing, ItemKind, Project, ViolationCode,
        test_support::{example_project, millis},
        validate_project,
    };

    fn codes(project: &Project) -> Vec<ViolationCode> {
        validate_project(project)
            .into_iter()
            .map(|violation| violation.code)
            .collect()
    }

    /// The title text of V2 (`[1 s, 4 s)`), whose opacity has keyframes at 0 and 0.5 s.
    fn title_opacity(project: &mut Project) -> &mut Animatable<f64> {
        match &mut project.sequences[0].tracks[1].items[0].kind {
            ItemKind::Text(text) => &mut text.transform.opacity,
            _ => unreachable!("the example's first V2 item is a text"),
        }
    }

    fn title_keyframes(project: &mut Project) -> &mut Vec<crate::Keyframe<f64>> {
        match title_opacity(project) {
            Animatable::Keyframed { keyframes } => keyframes,
            Animatable::Static { .. } => unreachable!("the example's title opacity is keyframed"),
        }
    }

    #[test]
    fn reports_keyframed_properties_without_keyframes() {
        let mut project = example_project();
        title_keyframes(&mut project).clear();
        assert_eq!(codes(&project), [ViolationCode::EmptyKeyframes]);
    }

    #[test]
    fn reports_unsorted_and_duplicate_keyframe_times() {
        let mut project = example_project();
        title_keyframes(&mut project).reverse();
        assert_eq!(codes(&project), [ViolationCode::KeyframesNotSorted]);

        let mut project = example_project();
        title_keyframes(&mut project)[1].time = millis(0);
        let violations = validate_project(&project);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].path.ends_with("opacity.keyframes[1].time"));
    }

    #[test]
    fn reports_keyframes_outside_the_item() {
        let mut project = example_project();
        title_keyframes(&mut project)[1].time = millis(3001);
        assert_eq!(codes(&project), [ViolationCode::KeyframeOutsideItem]);

        let mut project = example_project();
        title_keyframes(&mut project)[0].time = millis(-1);
        assert_eq!(codes(&project), [ViolationCode::KeyframeOutsideItem]);

        // The last instant of the item is allowed.
        let mut project = example_project();
        title_keyframes(&mut project)[1].time = millis(3000);
        assert_eq!(codes(&project), []);
    }

    #[test]
    fn reports_opacity_outside_zero_to_one() {
        let mut project = example_project();
        title_keyframes(&mut project)[1].value = 1.5;
        assert_eq!(codes(&project), [ViolationCode::ValueOutOfRange]);

        let mut project = example_project();
        *title_opacity(&mut project) = Animatable::constant(-0.1);
        let violations = validate_project(&project);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].path.ends_with("opacity.value"));
    }

    #[test]
    fn reports_volumes_outside_their_range() {
        let mut project = example_project();
        let ItemKind::MediaClip(clip) = &mut project.sequences[0].tracks[4].items[0].kind else {
            unreachable!("the example's A2 item is the music clip")
        };
        clip.volume_db = Animatable::constant(f64::NAN);
        let violations = validate_project(&project);
        assert_eq!(violations.len(), 1, "{violations:#?}");
        assert_eq!(violations[0].code, ViolationCode::ValueOutOfRange);
        assert!(violations[0].path.ends_with("volumeDb.value"));
    }

    #[test]
    fn reports_style_values_outside_their_range() {
        let mut project = example_project();
        let ItemKind::Text(text) = &mut project.sequences[0].tracks[1].items[0].kind else {
            unreachable!("the example's first V2 item is a text")
        };
        text.style.font_size = Animatable::constant(crate::DesignPixels(0.0));
        text.reveal = Animatable::constant(1.5);
        text.style.stroke.width = Animatable::constant(crate::DesignPixels(-1.0));
        let violations = validate_project(&project);
        let paths: Vec<_> = violations
            .iter()
            .map(|violation| violation.path.as_str())
            .collect();
        assert_eq!(violations.len(), 3, "{violations:#?}");
        assert!(paths[0].ends_with("kind.style.fontSize.value"), "{paths:?}");
        assert!(paths[1].ends_with("kind.reveal.value"), "{paths:?}");
        assert!(
            paths[2].ends_with("kind.style.stroke.width.value"),
            "{paths:?}"
        );
        assert_eq!(
            violations[0].message,
            "fontSize must be a size above 0 px, got 0"
        );
    }

    #[test]
    fn checks_keyframes_of_colour_and_point_properties() {
        let mut project = example_project();
        let ItemKind::Shape(shape) = &mut project.sequences[0].tracks[2].items[1].kind else {
            unreachable!("the example's second V3 item is a shape")
        };
        shape.fill = Animatable::Keyframed {
            keyframes: Vec::new(),
        };
        let violations = validate_project(&project);
        assert_eq!(codes(&project), [ViolationCode::EmptyKeyframes]);
        assert!(violations[0].path.ends_with("kind.fill.keyframes"));
    }

    #[test]
    fn reports_bezier_control_points_running_backwards() {
        let mut project = example_project();
        title_keyframes(&mut project)[0].easing = Easing::CubicBezier {
            x1: 1.2,
            y1: 0.0,
            x2: 0.5,
            y2: 2.0,
        };
        assert_eq!(codes(&project), [ViolationCode::ValueOutOfRange]);
    }
}
