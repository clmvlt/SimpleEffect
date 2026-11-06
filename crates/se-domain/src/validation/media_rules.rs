//! Rules of items referring to assets or templates: `UNKNOWN_REFERENCE`, `ASSET_KIND_MISMATCH`,
//! `INVALID_SPEED`, `NEGATIVE_TIME` (source start), `TIME_OUT_OF_RANGE` and
//! `SOURCE_RANGE_EXCEEDS_MEDIA` (source end), `VALUE_OUT_OF_RANGE` (fades).

use super::{Path, Report, ViolationCode, item_rules::ItemContext, report::seconds};
use crate::{Asset, AssetId, AssetKind, ImageItem, MediaClip, TemplateItem};

pub(super) fn check_media_clip(
    clip: &MediaClip,
    context: &ItemContext<'_>,
    path: &Path,
    report: &mut Report,
) {
    check_fades(clip, context, path, report);
    let Some(asset) = find_asset(clip.asset_id, context, path, report) else {
        return;
    };
    if asset.kind == AssetKind::Image {
        report.push(
            ViolationCode::AssetKindMismatch,
            &path.field("assetId"),
            format!(
                "asset {} is an image; show it with an image item, not a media clip",
                asset.id
            ),
        );
        return;
    }
    if clip.speed.is_nan() || clip.speed <= 0.0 {
        report.push(
            ViolationCode::InvalidSpeed,
            &path.field("speed"),
            format!(
                "speed must be a number greater than 0 (1 is normal speed), got {}",
                clip.speed
            ),
        );
        return;
    }
    check_source_range(clip, asset, context, path, report);
}

pub(super) fn check_image(
    image: &ImageItem,
    context: &ItemContext<'_>,
    path: &Path,
    report: &mut Report,
) {
    let Some(asset) = find_asset(image.asset_id, context, path, report) else {
        return;
    };
    if asset.kind != AssetKind::Image {
        report.push(
            ViolationCode::AssetKindMismatch,
            &path.field("assetId"),
            format!(
                "asset {} is not an image; play videos and sounds with a media clip",
                asset.id
            ),
        );
    }
}

pub(super) fn check_template(
    template: &TemplateItem,
    context: &ItemContext<'_>,
    path: &Path,
    report: &mut Report,
) {
    if !context.index.has_template(template.template_id) {
        report.push(
            ViolationCode::UnknownReference,
            &path.field("templateId"),
            format!(
                "template {} does not exist in this project; use the id of an existing template",
                template.template_id
            ),
        );
    }
}

/// The referenced asset, or `None` after reporting `UNKNOWN_REFERENCE`.
fn find_asset<'a>(
    id: AssetId,
    context: &ItemContext<'a>,
    path: &Path,
    report: &mut Report,
) -> Option<&'a Asset> {
    let asset = context.index.asset(id);
    if asset.is_none() {
        report.push(
            ViolationCode::UnknownReference,
            &path.field("assetId"),
            format!(
                "asset {id} does not exist in this project; import it or use an existing asset id"
            ),
        );
    }
    asset
}

/// The source range starts at or after zero, ends within the supported range and, when the
/// asset's duration is known, inside the asset.
fn check_source_range(
    clip: &MediaClip,
    asset: &Asset,
    context: &ItemContext<'_>,
    path: &Path,
    report: &mut Report,
) {
    if clip.source_start.is_negative() {
        report.push(
            ViolationCode::NegativeTime,
            &path.field("sourceStart"),
            format!(
                "source start is {}, before the start of the asset; use 0s or later",
                seconds(clip.source_start)
            ),
        );
    }
    let Some(source) = clip.source_range(context.item.timeline_range.duration()) else {
        report.push(
            ViolationCode::TimeOutOfRange,
            &path.field("sourceStart"),
            "the clip's source range ends beyond the supported time range (about 147 days); \
             lower the source start, the speed or the item duration"
                .to_owned(),
        );
        return;
    };
    let duration = asset.media_info.as_ref().and_then(|info| info.duration);
    if let Some(duration) = duration
        && source.end() > duration
    {
        report.push(
            ViolationCode::SourceRangeExceedsMedia,
            path,
            format!(
                "the clip plays its asset up to {} but the asset lasts {}; shorten the item, \
                 lower the speed or move the source start earlier",
                seconds(source.end()),
                seconds(duration)
            ),
        );
    }
}

/// Fades are non-negative and fit in the item together.
fn check_fades(clip: &MediaClip, context: &ItemContext<'_>, path: &Path, report: &mut Report) {
    let mut both_positive = true;
    for (name, fade) in [("fadeIn", clip.fade_in), ("fadeOut", clip.fade_out)] {
        if fade.is_negative() {
            both_positive = false;
            report.push(
                ViolationCode::ValueOutOfRange,
                &path.field(name),
                format!("{name} is {}; a fade lasts 0s or more", seconds(fade)),
            );
        }
    }
    let duration = context.item.timeline_range.duration();
    let total = clip.fade_in.checked_add(clip.fade_out);
    if both_positive && total.is_none_or(|total| total > duration) {
        report.push(
            ViolationCode::ValueOutOfRange,
            path,
            format!(
                "fades ({} in + {} out) are longer than the item ({}); shorten them",
                seconds(clip.fade_in),
                seconds(clip.fade_out),
                seconds(duration)
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        AssetId, Flicks, ItemKind, Project, TemplateId, ViolationCode,
        test_support::{example_project, millis, range_ms},
        validate_project,
    };

    fn codes(project: &Project) -> Vec<ViolationCode> {
        validate_project(project)
            .into_iter()
            .map(|violation| violation.code)
            .collect()
    }

    /// The fast interview clip of V1: `[10 s, 14 s)` at 2× from 20 s (source 20 s → 28 s of 60 s).
    fn fast_clip(project: &mut Project) -> &mut crate::MediaClip {
        match &mut project.sequences[0].tracks[0].items[1].kind {
            ItemKind::MediaClip(clip) => clip,
            _ => unreachable!("the example's second V1 item is a media clip"),
        }
    }

    #[test]
    fn reports_unknown_assets_and_templates() {
        let mut project = example_project();
        fast_clip(&mut project).asset_id = AssetId::from_test_index(99);
        if let ItemKind::MotionTemplate(template) =
            &mut project.sequences[0].tracks[2].items[0].kind
        {
            template.template_id = TemplateId::from_test_index(99);
        }
        assert_eq!(codes(&project), [ViolationCode::UnknownReference; 2]);
    }

    #[test]
    fn reports_assets_of_the_wrong_kind() {
        let mut project = example_project();
        fast_clip(&mut project).asset_id = project.assets[2].id;
        if let ItemKind::Image(image) = &mut project.sequences[0].tracks[1].items[1].kind {
            image.asset_id = project.assets[0].id;
        }
        assert_eq!(codes(&project), [ViolationCode::AssetKindMismatch; 2]);
    }

    #[test]
    fn reports_speeds_that_are_not_positive() {
        for speed in [0.0, -1.0, f64::NAN] {
            let mut project = example_project();
            fast_clip(&mut project).speed = speed;
            assert_eq!(codes(&project), [ViolationCode::InvalidSpeed], "{speed}");
        }
    }

    #[test]
    fn reports_negative_source_starts() {
        let mut project = example_project();
        fast_clip(&mut project).source_start = millis(-1);
        assert_eq!(codes(&project), [ViolationCode::NegativeTime]);
    }

    #[test]
    fn reports_source_ranges_beyond_the_asset() {
        let mut project = example_project();
        // 4 s at 2× from 53 s needs the asset up to 61 s; it lasts 60 s.
        fast_clip(&mut project).source_start = millis(53_000);
        let violations = validate_project(&project);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, ViolationCode::SourceRangeExceedsMedia);
        assert!(
            violations[0].message.contains("61.000s"),
            "{}",
            violations[0].message
        );
    }

    #[test]
    fn reports_source_ranges_beyond_the_supported_range() {
        let mut project = example_project();
        fast_clip(&mut project).source_start = Flicks::MAX;
        assert_eq!(codes(&project), [ViolationCode::TimeOutOfRange]);
    }

    #[test]
    fn source_range_is_unchecked_when_the_duration_is_unknown() {
        let mut project = example_project();
        project.assets[0].media_info = None;
        fast_clip(&mut project).source_start = millis(600_000);
        assert_eq!(validate_project(&project), []);
    }

    #[test]
    fn reports_negative_fades_and_fades_longer_than_the_item() {
        let mut project = example_project();
        fast_clip(&mut project).fade_out = millis(-1);
        assert_eq!(codes(&project), [ViolationCode::ValueOutOfRange]);

        let mut project = example_project();
        project.sequences[0].tracks[0].items[1].timeline_range = range_ms(10_000, 400);
        // fade in 500 ms > 400 ms item.
        assert_eq!(codes(&project), [ViolationCode::ValueOutOfRange]);
    }
}
