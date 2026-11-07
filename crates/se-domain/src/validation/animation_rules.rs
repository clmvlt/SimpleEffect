//! Rules of animatable properties: `EMPTY_KEYFRAMES`, `KEYFRAMES_NOT_SORTED`,
//! `KEYFRAME_OUTSIDE_ITEM`, and `VALUE_OUT_OF_RANGE` for opacity, volume and Bézier control
//! points.

use std::ops::RangeInclusive;

use super::{Path, Report, ViolationCode, report::seconds};
use crate::{Animatable, Easing, Flicks, Item, ItemKind, MediaClip, Transform2D};

/// Checks every animatable property of `item` (paths are relative to `kind_path`).
pub(super) fn check(item: &Item, kind_path: &Path, report: &mut Report) {
    let duration = item.timeline_range.duration();
    if let Some(transform) = item.kind.transform() {
        check_transform(transform, &kind_path.field("transform"), duration, report);
    }
    if let ItemKind::MediaClip(clip) = &item.kind {
        let volume = kind_path.field("volumeDb");
        check_keyframes(&clip.volume_db, &volume, duration, report);
        check_values(
            &clip.volume_db,
            &volume,
            MediaClip::MIN_VOLUME_DB..=MediaClip::MAX_VOLUME_DB,
            report,
            |value| {
                format!(
                    "volume must be between {} dB and {} dB, got {value}",
                    MediaClip::MIN_VOLUME_DB,
                    MediaClip::MAX_VOLUME_DB
                )
            },
        );
    }
}

fn check_transform(transform: &Transform2D, path: &Path, duration: Flicks, report: &mut Report) {
    check_keyframes(
        &transform.position,
        &path.field("position"),
        duration,
        report,
    );
    check_keyframes(&transform.scale, &path.field("scale"), duration, report);
    check_keyframes(
        &transform.rotation,
        &path.field("rotation"),
        duration,
        report,
    );
    check_keyframes(&transform.anchor, &path.field("anchor"), duration, report);
    let opacity = path.field("opacity");
    check_keyframes(&transform.opacity, &opacity, duration, report);
    check_values(&transform.opacity, &opacity, 0.0..=1.0, report, |value| {
        format!("opacity must be between 0 (invisible) and 1 (opaque), got {value}")
    });
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

/// Values of `property` (static or keyframed) are in `allowed` (NaN never is).
fn check_values(
    property: &Animatable<f64>,
    path: &Path,
    allowed: RangeInclusive<f64>,
    report: &mut Report,
    message: impl Fn(f64) -> String,
) {
    let values: Vec<(Path, f64)> = match property {
        Animatable::Static { value } => vec![(path.field("value"), *value)],
        Animatable::Keyframed { keyframes } => keyframes
            .iter()
            .enumerate()
            .map(|(index, keyframe)| {
                (
                    path.field("keyframes").key(index).field("value"),
                    keyframe.value,
                )
            })
            .collect(),
    };
    for (value_path, value) in values {
        if !allowed.contains(&value) {
            report.push(ViolationCode::ValueOutOfRange, &value_path, message(value));
        }
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
