//! Putting items on tracks: the two primitive item moves every item operation is built on.

use super::{
    AddItem, Effect, RemoveItem, RestoreItem,
    checks::{ensure_free, ensure_item_id_unused, ensure_unlocked},
    lookup::{ItemLocation, TrackLocation},
    transition_upkeep::drop_broken_transitions,
};
use crate::{DomainError, Item, Project, Track};

/// Inserts `item` into `track`, keeping the items sorted by start (a validation rule).
pub(crate) fn insert_sorted(track: &mut Track, item: Item) {
    let start = item.timeline_range.start();
    let index = track
        .items
        .partition_point(|other| other.timeline_range.start() <= start);
    track.items.insert(index, item);
}

/// Replaces the item at `location` by `replacement` (same id) on track `target` (its own track
/// or another one), keeping the item's current revision. Records a [`RestoreItem`] of the old
/// state as the inverse, then removes the transitions of both tracks that the change broke.
///
/// # Errors
/// [`DomainError::TrackLocked`] if either track is locked, [`DomainError::Overlap`] if the
/// replacement overlaps another item of `target`.
pub(crate) fn place_item(
    project: &mut Project,
    location: ItemLocation,
    target: TrackLocation,
    mut replacement: Item,
    effect: &mut Effect,
) -> Result<(), DomainError> {
    let source = location.track.get(project);
    ensure_unlocked(source)?;
    let previous_track = source.id;
    let target_track = target.get(project);
    ensure_unlocked(target_track)?;
    ensure_free(target_track, replacement.id, replacement.timeline_range)?;
    let previous = location.track.get_mut(project).items.remove(location.index);
    replacement.revision = previous.revision;
    effect.touched.items.insert(previous.id);
    let id = replacement.id;
    insert_sorted(target.get_mut(project), replacement);
    effect.undo_with(RestoreItem {
        track: previous_track,
        item: previous,
    });
    drop_broken_transitions(project, &[location.track, target], id, effect);
    Ok(())
}

/// Puts the new `item` on track `target`. Records a [`RemoveItem`] as the inverse.
///
/// # Errors
/// [`DomainError::IdAlreadyUsed`], [`DomainError::TrackLocked`] or [`DomainError::Overlap`].
pub(crate) fn insert_new_item(
    project: &mut Project,
    target: TrackLocation,
    item: Item,
    effect: &mut Effect,
) -> Result<(), DomainError> {
    ensure_item_id_unused(project, item.id)?;
    let track = target.get(project);
    ensure_unlocked(track)?;
    ensure_free(track, item.id, item.timeline_range)?;
    effect.touched.items.insert(item.id);
    effect.undo_with(RemoveItem {
        item: item.id,
        expected_revision: None,
    });
    insert_sorted(target.get_mut(project), item);
    Ok(())
}

/// Takes the item at `location` off its track. Records an [`AddItem`] of it as the inverse, then
/// removes the transitions joining it.
///
/// # Errors
/// [`DomainError::TrackLocked`].
pub(crate) fn remove_item(
    project: &mut Project,
    location: ItemLocation,
    effect: &mut Effect,
) -> Result<(), DomainError> {
    let track = location.track.get(project);
    ensure_unlocked(track)?;
    let track_id = track.id;
    let item = location.track.get_mut(project).items.remove(location.index);
    effect.touched.items.insert(item.id);
    let id = item.id;
    effect.undo_with(AddItem {
        track: track_id,
        item,
    });
    drop_broken_transitions(project, &[location.track], id, effect);
    Ok(())
}
