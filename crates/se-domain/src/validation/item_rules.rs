//! Rules of one item: `NEGATIVE_TIME` and `EMPTY_ITEM` for its timeline range,
//! `ITEM_NOT_ALLOWED_ON_TRACK`, then the kind-specific and animation rules.

use super::{
    Path, ProjectIndex, Report, ViolationCode, animation_rules, media_rules, report::seconds,
};
use crate::{AssetKind, Item, ItemKind, TrackKind};

/// An item with what its rules need to know about its surroundings.
pub(super) struct ItemContext<'a> {
    /// The item checked.
    pub(super) item: &'a Item,
    /// Kind of the track holding it.
    pub(super) track_kind: TrackKind,
    /// Assets and templates of the project.
    pub(super) index: &'a ProjectIndex<'a>,
    /// Path of the item.
    pub(super) path: Path,
}

pub(super) fn check(context: &ItemContext<'_>, report: &mut Report) {
    check_timeline_range(context, report);
    check_track_kind(context, report);
    let kind_path = context.path.field("kind");
    match &context.item.kind {
        ItemKind::MediaClip(clip) => {
            media_rules::check_media_clip(clip, context, &kind_path, report);
        }
        ItemKind::Image(image) => media_rules::check_image(image, context, &kind_path, report),
        ItemKind::MotionTemplate(template) => {
            media_rules::check_template(template, context, &kind_path, report);
        }
        ItemKind::Text(_) | ItemKind::Shape(_) | ItemKind::Caption(_) => {}
    }
    animation_rules::check(context.item, &kind_path, report);
}

fn check_timeline_range(context: &ItemContext<'_>, report: &mut Report) {
    let range = context.item.timeline_range;
    let path = context.path.field("timelineRange");
    if range.start().is_negative() {
        report.push(
            ViolationCode::NegativeTime,
            &path.field("start"),
            format!(
                "item starts at {}, before the start of the sequence; move it to 0s or later",
                seconds(range.start())
            ),
        );
    }
    if range.is_empty() {
        report.push(
            ViolationCode::EmptyItem,
            &path.field("duration"),
            "item has a zero duration; give it a positive duration or remove it".to_owned(),
        );
    }
}

/// Audio tracks hold only media clips of assets with sound; video tracks hold every kind except
/// clips of audio-only assets. Unknown assets are left to the reference rule.
fn check_track_kind(context: &ItemContext<'_>, report: &mut Report) {
    let kind = &context.item.kind;
    let clip_asset_kind = match kind {
        ItemKind::MediaClip(clip) => context.index.asset(clip.asset_id).map(|asset| asset.kind),
        _ => None,
    };
    let message = match (context.track_kind, kind) {
        (TrackKind::Audio, ItemKind::MediaClip(_)) => None,
        (TrackKind::Audio, _) => Some(format!(
            "a {} item cannot sit on an audio track; move it to a video track",
            kind.label()
        )),
        (TrackKind::Video, ItemKind::MediaClip(_)) if clip_asset_kind == Some(AssetKind::Audio) => {
            Some(
                "a media clip of an audio-only asset cannot sit on a video track; move it to an \
                 audio track"
                    .to_owned(),
            )
        }
        (TrackKind::Video, _) => None,
    };
    if let Some(message) = message {
        report.push(
            ViolationCode::ItemNotAllowedOnTrack,
            &context.path.field("kind"),
            message,
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ViolationCode,
        test_support::{example_project, range_ms},
        validate_project,
    };

    fn codes(project: &crate::Project) -> Vec<ViolationCode> {
        validate_project(project)
            .into_iter()
            .map(|violation| violation.code)
            .collect()
    }

    #[test]
    fn reports_items_starting_before_zero() {
        let mut project = example_project();
        // V2 holds a text [1 s, 4 s): start it at -1 s, keeping it clear of the logo.
        project.sequences[0].tracks[1].items[0].timeline_range = range_ms(-1000, 3000);
        assert_eq!(codes(&project), [ViolationCode::NegativeTime]);
    }

    #[test]
    fn reports_empty_items() {
        let mut project = example_project();
        // The shape on V3 has no keyframes, so only the empty range is reported.
        project.sequences[0].tracks[2].items[1].timeline_range = range_ms(6000, 0);
        assert_eq!(codes(&project), [ViolationCode::EmptyItem]);
    }

    #[test]
    fn reports_visual_items_on_audio_tracks() {
        let mut project = example_project();
        let text = project.sequences[0].tracks[1].items.remove(0);
        // Make room on A2: the music [0 s, 14 s) now starts after the text [1 s, 4 s).
        project.sequences[0].tracks[4].items[0].timeline_range = range_ms(5000, 9000);
        project.sequences[0].tracks[4].items.insert(0, text);
        let violations = validate_project(&project);
        assert_eq!(violations.len(), 1, "{violations:#?}");
        assert_eq!(violations[0].code, ViolationCode::ItemNotAllowedOnTrack);
        assert!(violations[0].message.contains("a text item"));
    }

    #[test]
    fn reports_audio_only_clips_on_video_tracks() {
        let mut project = example_project();
        let music = project.sequences[0].tracks[4].items.remove(0);
        project.sequences[0].tracks[0].items = vec![music];
        assert_eq!(codes(&project), [ViolationCode::ItemNotAllowedOnTrack]);
    }
}
