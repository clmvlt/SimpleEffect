//! Edits of existing captions in plans: text, emphasis, merging, splitting, shifting.

use se_domain::{
    CaptionItem, CaptionPatch, Flicks, ItemEdge, ItemId, ItemKind, ItemPatch, MoveItem, RemoveItem,
    SplitItem, TrimItem, UpdateItem,
    captions::{merged_words, retimed_words, split_point},
};

use super::styles::not_a_caption;
use crate::{
    AppError,
    edit_plan::{EditPlan, ItemPlace, find_item},
};

/// The caption `item` of the draft and where it is.
fn caption<'p>(place: &'p ItemPlace<'_>) -> Result<&'p CaptionItem, AppError> {
    match &place.item.kind {
        ItemKind::Caption(caption) => Ok(caption),
        other => Err(not_a_caption(place.item.id, other.label())),
    }
}

fn set_words(plan: &mut EditPlan<'_>, item: ItemId, words: Vec<se_domain::CaptionWord>) {
    plan.push(UpdateItem {
        item,
        patch: ItemPatch::Caption(CaptionPatch {
            words: Some(words),
            style_id: None,
        }),
        expected_revision: None,
    });
}

/// Gives caption `item` the text `text` (unchanged words keep their timing:
/// `se_domain::captions::retimed_words`).
///
/// # Errors
/// Not a caption, an empty text.
pub(crate) fn plan_text(plan: &mut EditPlan<'_>, item: ItemId, text: &str) -> Result<(), AppError> {
    if text.trim().is_empty() {
        return Err(AppError::InvalidEdit {
            reason: "a caption needs text; delete the caption instead of emptying it".to_owned(),
        });
    }
    let draft = plan.project()?;
    let place = find_item(draft, item)?;
    let duration = place.item.timeline_range.duration();
    let words = retimed_words(&caption(&place)?.words, text, duration);
    set_words(plan, item, words);
    plan.project()?;
    Ok(())
}

/// Puts the words `indices` of caption `item` forward (or back).
///
/// # Errors
/// Not a caption, an index beyond its words.
pub(crate) fn plan_emphasis(
    plan: &mut EditPlan<'_>,
    item: ItemId,
    indices: &[u32],
    emphasis: bool,
) -> Result<(), AppError> {
    let draft = plan.project()?;
    let place = find_item(draft, item)?;
    let mut words = caption(&place)?.words.clone();
    for &index in indices {
        let word = usize::try_from(index)
            .ok()
            .and_then(|index| words.get_mut(index))
            .ok_or_else(|| AppError::InvalidEdit {
                reason: format!(
                    "caption {item} has {} words (indices 0 to {}); word {index} does not exist",
                    caption(&place).map_or(0, |caption| caption.words.len()),
                    caption(&place).map_or(0, |caption| caption.words.len().saturating_sub(1)),
                ),
            })?;
        word.emphasis = emphasis;
    }
    set_words(plan, item, words);
    plan.project()?;
    Ok(())
}

/// Merges `items` (two or more captions following each other on one track) into the first,
/// which then spans them all with their words; returns the first.
///
/// # Errors
/// Fewer than two captions, captions of several tracks, other items between them.
pub(crate) fn plan_merge(plan: &mut EditPlan<'_>, items: &[ItemId]) -> Result<ItemId, AppError> {
    let invalid = |reason: &str| AppError::InvalidEdit {
        reason: reason.to_owned(),
    };
    let draft = plan.project()?.clone();
    let mut places = items
        .iter()
        .map(|&id| find_item(&draft, id))
        .collect::<Result<Vec<_>, _>>()?;
    if places.len() < 2 {
        return Err(invalid("merging needs two or more captions"));
    }
    let track = places[0].track.id;
    if places.iter().any(|place| place.track.id != track) {
        return Err(invalid("only captions of one track can be merged"));
    }
    places.sort_by_key(|place| place.item.timeline_range.start());
    let positions: Vec<usize> = places
        .iter()
        .filter_map(|place| {
            place
                .track
                .items
                .iter()
                .position(|item| item.id == place.item.id)
        })
        .collect();
    if positions.windows(2).any(|pair| pair[1] != pair[0] + 1) {
        return Err(invalid(
            "the captions must follow each other on their track, with no other item between",
        ));
    }
    let first = places[0].item;
    let mut words = caption(&places[0])?.words.clone();
    for place in &places[1..] {
        let offset = place
            .item
            .timeline_range
            .start()
            .checked_sub(first.timeline_range.start())
            .unwrap_or(Flicks::ZERO);
        words = merged_words(&words, &caption(place)?.words, offset);
    }
    let last_end = places.last().map_or(first.timeline_range.end(), |place| {
        place.item.timeline_range.end()
    });
    for place in &places[1..] {
        plan.push(RemoveItem {
            item: place.item.id,
            expected_revision: None,
        });
    }
    plan.push(TrimItem {
        item: first.id,
        edge: ItemEdge::End,
        to: last_end,
        expected_revision: None,
    });
    set_words(plan, first.id, words);
    plan.project()?;
    Ok(first.id)
}

/// Splits caption `item` halfway through the pause before word `word_index`, which starts the
/// new right part; returns the right part.
///
/// # Errors
/// Not a caption, an index that leaves no word on one side.
pub(crate) fn plan_split(
    plan: &mut EditPlan<'_>,
    item: ItemId,
    word_index: u32,
) -> Result<ItemId, AppError> {
    let draft = plan.project()?;
    let place = find_item(draft, item)?;
    let words = &caption(&place)?.words;
    let range = place.item.timeline_range;
    let at = usize::try_from(word_index)
        .ok()
        .and_then(|index| split_point(words, index))
        .and_then(|local| range.start().checked_add(local))
        .filter(|&at| at > range.start() && at < range.end())
        .ok_or_else(|| AppError::InvalidEdit {
            reason: format!(
                "caption {item} has {} words: split before a word from 1 to {}",
                words.len(),
                words.len().saturating_sub(1)
            ),
        })?;
    let new_item_id = plan.ids().item()?;
    plan.push(SplitItem {
        item,
        at,
        new_item_id,
        expected_revision: None,
    });
    plan.project()?;
    Ok(new_item_id)
}

/// Moves `items` by `delta` on their tracks, all together.
///
/// # Errors
/// No item, an item starting before 0 afterwards, overlaps the domain refuses.
pub(crate) fn plan_shift(
    plan: &mut EditPlan<'_>,
    items: &[ItemId],
    delta: Flicks,
) -> Result<(), AppError> {
    if items.is_empty() {
        return Err(AppError::InvalidEdit {
            reason: "no caption given: list the captions to shift".to_owned(),
        });
    }
    let draft = plan.project()?;
    let mut moves = Vec::with_capacity(items.len());
    for &id in items {
        let place = find_item(draft, id)?;
        let start = place
            .item
            .timeline_range
            .start()
            .checked_add(delta)
            .filter(|start| !start.is_negative())
            .ok_or_else(|| AppError::InvalidEdit {
                reason: format!(
                    "shifting by {} s would start item {id} before the sequence's start",
                    delta.to_seconds_f64()
                ),
            })?;
        moves.push((place.item.timeline_range.start(), id, place.track.id, start));
    }
    // Items moved later go last first (and earlier ones first first), so moved items never
    // overlap each other on the way.
    moves.sort_by_key(|&(start, ..)| start);
    if delta > Flicks::ZERO {
        moves.reverse();
    }
    for (_, item, track, start) in moves {
        plan.push(MoveItem {
            item,
            track,
            start,
            expected_revision: None,
        });
    }
    plan.project()?;
    Ok(())
}
