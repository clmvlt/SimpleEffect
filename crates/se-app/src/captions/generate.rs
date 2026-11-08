//! Placing caption drafts on the caption track: generation from spoken words and SRT imports
//! share it. Captions of the track crossing the range are replaced (the range grows to cover
//! them whole), in the same transaction.

use se_domain::{
    AddItem, CaptionItem, CaptionStyle, CaptionStyleId, Flicks, FrameRate, Item, ItemId, ItemKind,
    Origin, Project, RemoveItem, Sequence, SequenceId, TimeRange,
    captions::{CaptionDraft, SegmentationParams, TimedWord, segment_words},
};
use se_protocol::{CaptionSegmentation, CaptionStyleChoice};

use super::{
    styles::{drop_unused_styles, style_to_use},
    track::{caption_track, ensure_caption_track},
};
use crate::{
    AppError,
    edit_plan::{EditPlan, find_sequence},
};

/// Where and how new captions go.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CaptionPlacement<'a> {
    /// The sequence.
    pub(crate) sequence_id: SequenceId,
    /// The range to fill (grown over the captions it crosses).
    pub(crate) range: TimeRange,
    /// The style asked for (`None`: the replaced captions' style, else the default preset).
    pub(crate) style: Option<&'a CaptionStyleChoice>,
    /// Name of a caption track created for them.
    pub(crate) track_name: Option<&'a str>,
    /// Who makes them.
    pub(crate) origin: Origin,
}

/// What [`plan_captions`] planned (nothing when no draft came out).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct PlacedCaptions {
    /// The new captions, in time order.
    pub(crate) item_ids: Vec<ItemId>,
    /// The caption track.
    pub(crate) track_id: Option<se_domain::TrackId>,
    /// Their style.
    pub(crate) style_id: Option<CaptionStyleId>,
    /// Captions replaced.
    pub(crate) replaced: u32,
}

/// The extent of a sequence's content: from 0 to its last item's end (`None` when empty).
pub(crate) fn sequence_extent(sequence: &Sequence) -> Option<TimeRange> {
    let end = sequence
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .map(|item| item.timeline_range.end())
        .max()?;
    TimeRange::from_start_end(Flicks::ZERO, end).ok()
}

/// Segmentation limits for `style` with the request's overrides and the sequence's frames.
pub(crate) fn segmentation_params(
    style: &CaptionStyle,
    overrides: Option<&CaptionSegmentation>,
    frame_rate: FrameRate,
) -> SegmentationParams {
    let defaults = SegmentationParams::for_style(style, Some(frame_rate));
    let Some(overrides) = overrides else {
        return defaults;
    };
    SegmentationParams {
        max_chars_per_line: overrides
            .max_chars_per_line
            .unwrap_or(defaults.max_chars_per_line),
        max_lines: overrides.max_lines.unwrap_or(defaults.max_lines),
        min_duration: overrides.min_duration.unwrap_or(defaults.min_duration),
        max_duration: overrides.max_duration.unwrap_or(defaults.max_duration),
        pause: overrides.pause.unwrap_or(defaults.pause),
        frame_rate: defaults.frame_rate,
    }
}

/// Plans captions from `words` (sequence time) over the placement's range.
///
/// # Errors
/// See [`plan_captions`]; segmentation overrides out of range.
pub(crate) fn plan_generation(
    plan: &mut EditPlan<'_>,
    placement: &CaptionPlacement<'_>,
    words: &[TimedWord],
    overrides: Option<&CaptionSegmentation>,
) -> Result<PlacedCaptions, AppError> {
    check_overrides(overrides)?;
    let frame_rate = find_sequence(plan.project()?, placement.sequence_id)?
        .format
        .frame_rate;
    plan_captions(plan, placement, |style, bounds| {
        segment_words(
            words,
            &segmentation_params(style, overrides, frame_rate),
            bounds,
        )
    })
}

/// Replaces the captions of the caption track crossing the range by the drafts `make` gives for
/// the grown range and the style to use; nothing is planned when it gives none.
///
/// # Errors
/// Unknown sequence or style, an unknown preset, the domain's refusals, id generator failures.
pub(crate) fn plan_captions(
    plan: &mut EditPlan<'_>,
    placement: &CaptionPlacement<'_>,
    make: impl FnOnce(&CaptionStyle, TimeRange) -> Vec<CaptionDraft>,
) -> Result<PlacedCaptions, AppError> {
    let before: Project = plan.project()?.clone();
    let sequence = find_sequence(&before, placement.sequence_id)?;
    let (replaced, range) = crossed_captions(sequence, placement.range);
    let fallback = replaced.first().map(|(_, style)| *style);
    let style = style_to_use(plan, &before, &sequence.format, placement.style, fallback)?;
    let drafts = make(style.style(), range);
    if drafts.is_empty() {
        return Ok(PlacedCaptions::default());
    }
    let (track_id, _) = ensure_caption_track(plan, placement.sequence_id, placement.track_name)?;
    let style_id = style.commit(plan);
    for (item, _) in &replaced {
        plan.push(RemoveItem {
            item: *item,
            expected_revision: None,
        });
    }
    let mut item_ids = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let id = plan.ids().item()?;
        item_ids.push(id);
        plan.push(AddItem {
            track: track_id,
            item: Item {
                id,
                revision: 0,
                origin: placement.origin,
                timeline_range: draft.range,
                link_group: None,
                kind: ItemKind::Caption(CaptionItem {
                    words: draft.words,
                    style_id,
                }),
            },
        });
    }
    drop_unused_styles(plan, &before)?;
    plan.project()?;
    Ok(PlacedCaptions {
        item_ids,
        track_id: Some(track_id),
        style_id: Some(style_id),
        replaced: u32::try_from(replaced.len()).unwrap_or(u32::MAX),
    })
}

/// The captions of the sequence's caption track crossing `range` (with their style), and
/// `range` grown to cover them whole.
fn crossed_captions(
    sequence: &Sequence,
    range: TimeRange,
) -> (Vec<(ItemId, CaptionStyleId)>, TimeRange) {
    let Some(track) = caption_track(sequence) else {
        return (Vec::new(), range);
    };
    let mut start = range.start();
    let mut end = range.end();
    let mut crossed = Vec::new();
    for item in &track.items {
        let ItemKind::Caption(caption) = &item.kind else {
            continue;
        };
        if item.timeline_range.intersects(range) {
            start = start.min(item.timeline_range.start());
            end = end.max(item.timeline_range.end());
            crossed.push((item.id, caption.style_id));
        }
    }
    let grown = TimeRange::from_start_end(start, end).unwrap_or(range);
    (crossed, grown)
}

/// Refuses segmentation overrides outside the style limits.
fn check_overrides(overrides: Option<&CaptionSegmentation>) -> Result<(), AppError> {
    let Some(overrides) = overrides else {
        return Ok(());
    };
    let invalid = |reason: &str| {
        Err(AppError::InvalidEdit {
            reason: reason.to_owned(),
        })
    };
    if overrides
        .max_chars_per_line
        .is_some_and(|chars| !se_domain::CAPTION_LINE_CHARS.contains(&chars))
    {
        return invalid("maxCharsPerLine must be between 4 and 80");
    }
    if overrides
        .max_lines
        .is_some_and(|lines| !se_domain::CAPTION_LINES.contains(&lines))
    {
        return invalid("maxLines must be between 1 and 4");
    }
    let positive = |time: Option<Flicks>| time.is_none_or(|time| time > Flicks::ZERO);
    if !positive(overrides.min_duration)
        || !positive(overrides.max_duration)
        || !positive(overrides.pause)
    {
        return invalid("durations and pauses must be positive");
    }
    if let (Some(min), Some(max)) = (overrides.min_duration, overrides.max_duration)
        && min > max
    {
        return invalid("the shortest caption must not be longer than the longest");
    }
    Ok(())
}
