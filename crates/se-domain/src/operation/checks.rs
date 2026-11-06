//! Preconditions shared by operations, each failing with an actionable [`DomainError`].

use super::lookup::all_items;
use crate::{DomainError, Item, ItemId, Project, TimeRange, Track};

/// Fails if `track` is locked: its items cannot be added, changed or removed, and it cannot be
/// removed (its header can still change, to unlock it).
pub(crate) fn ensure_unlocked(track: &Track) -> Result<(), DomainError> {
    if track.locked {
        Err(DomainError::TrackLocked {
            track: track.id,
            track_name: track.name.clone(),
        })
    } else {
        Ok(())
    }
}

/// Fails if `expected` is set and differs from the item's revision.
pub(crate) fn check_revision(item: &Item, expected: Option<u64>) -> Result<(), DomainError> {
    match expected {
        Some(expected) if expected != item.revision => Err(DomainError::RevisionConflict {
            item: item.id,
            expected,
            actual: item.revision,
        }),
        _ => Ok(()),
    }
}

/// Fails if `range` overlaps an item of `track` other than `item` itself.
pub(crate) fn ensure_free(
    track: &Track,
    item: ItemId,
    range: TimeRange,
) -> Result<(), DomainError> {
    let overlap = track.items.iter().find_map(|other| {
        if other.id == item {
            return None;
        }
        range
            .intersection(other.timeline_range)
            .map(|shared| (other.id, shared))
    });
    match overlap {
        Some((other, shared)) => Err(DomainError::Overlap {
            item,
            other,
            track_name: track.name.clone(),
            start: shared.start(),
            end: shared.end(),
        }),
        None => Ok(()),
    }
}

/// Fails if an item of the project already has id `id`.
pub(crate) fn ensure_item_id_unused(project: &Project, id: ItemId) -> Result<(), DomainError> {
    ensure_unused(all_items(project).any(|item| item.id == id), "item", &id)
}

/// Fails with [`DomainError::IdAlreadyUsed`] when `used`.
pub(crate) fn ensure_unused(
    used: bool,
    kind: &'static str,
    id: &dyn std::fmt::Display,
) -> Result<(), DomainError> {
    if used {
        Err(DomainError::IdAlreadyUsed {
            kind,
            id: id.to_string(),
        })
    } else {
        Ok(())
    }
}

/// Fails if `name` is empty or only whitespace.
pub(crate) fn ensure_name(entity: &'static str, name: &str) -> Result<(), DomainError> {
    if name.trim().is_empty() {
        Err(DomainError::EmptyName { entity })
    } else {
        Ok(())
    }
}

/// Where to insert into a list of `len` entries: `index` (at most `len`), or the end.
pub(crate) fn insertion_index(
    entity: &'static str,
    index: Option<usize>,
    len: usize,
) -> Result<usize, DomainError> {
    match index {
        None => Ok(len),
        Some(index) if index <= len => Ok(index),
        Some(index) => Err(DomainError::IndexOutOfBounds { entity, index, len }),
    }
}

/// Writes `value` (if any) into `slot` and returns the value it replaced: the matching field of
/// an inverse patch.
pub(crate) fn swap_field<T: Clone>(slot: &mut T, value: Option<&T>) -> Option<T> {
    value.map(|value| std::mem::replace(slot, value.clone()))
}
