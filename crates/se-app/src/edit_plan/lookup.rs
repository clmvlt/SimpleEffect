//! Finding items, tracks and sequences of a draft, and the checks every item edit shares (ids
//! listed once, one sequence per edit).

use std::{collections::BTreeSet, fmt::Display};

use se_domain::{DomainError, Item, ItemId, Project, Sequence, SequenceId, Track, TrackId};

use crate::AppError;

/// Where an item is.
pub(crate) struct ItemPlace<'a> {
    /// Its sequence.
    pub(crate) sequence: &'a Sequence,
    /// Its track.
    pub(crate) track: &'a Track,
    /// The item.
    pub(crate) item: &'a Item,
}

/// The item `id` and where it is.
///
/// # Errors
/// No item has this id.
pub(crate) fn find_item(project: &Project, id: ItemId) -> Result<ItemPlace<'_>, AppError> {
    project
        .sequences
        .iter()
        .flat_map(|sequence| sequence.tracks.iter().map(move |track| (sequence, track)))
        .find_map(|(sequence, track)| {
            let item = track.items.iter().find(|item| item.id == id)?;
            Some(ItemPlace {
                sequence,
                track,
                item,
            })
        })
        .ok_or_else(|| DomainError::ItemNotFound { item: id }.into())
}

/// The track `id` and its sequence.
///
/// # Errors
/// No track has this id.
pub(crate) fn find_track(project: &Project, id: TrackId) -> Result<(&Sequence, &Track), AppError> {
    project
        .sequences
        .iter()
        .find_map(|sequence| {
            let track = sequence.tracks.iter().find(|track| track.id == id)?;
            Some((sequence, track))
        })
        .ok_or_else(|| DomainError::TrackNotFound { track: id }.into())
}

/// The sequence `id`.
///
/// # Errors
/// No sequence has this id.
pub(crate) fn find_sequence(project: &Project, id: SequenceId) -> Result<&Sequence, AppError> {
    project
        .sequences
        .iter()
        .find(|sequence| sequence.id == id)
        .ok_or_else(|| DomainError::SequenceNotFound { sequence: id }.into())
}

/// The one sequence holding every listed item and track.
///
/// # Errors
/// An unknown id, nothing listed, or ids from several sequences.
pub(crate) fn sequence_of(
    project: &Project,
    items: impl IntoIterator<Item = ItemId>,
    tracks: impl IntoIterator<Item = TrackId>,
) -> Result<SequenceId, AppError> {
    let mut found: Option<SequenceId> = None;
    let item_sequences = items
        .into_iter()
        .map(|id| find_item(project, id).map(|place| place.sequence.id));
    let track_sequences = tracks
        .into_iter()
        .map(|id| find_track(project, id).map(|(sequence, _)| sequence.id));
    for sequence in item_sequences.chain(track_sequences) {
        let sequence = sequence?;
        if found.is_some_and(|first| first != sequence) {
            return Err(AppError::InvalidEdit {
                reason: "the items and tracks of one edit must belong to one sequence: edit each \
                         sequence separately"
                    .to_owned(),
            });
        }
        found = Some(sequence);
    }
    found.ok_or_else(|| AppError::InvalidEdit {
        reason: "nothing to edit: list at least one item".to_owned(),
    })
}

/// `ids` as a set, refused when empty or when an id is listed twice.
///
/// # Errors
/// [`AppError::InvalidEdit`] naming the repeated id.
pub(crate) fn distinct<T: Ord + Copy + Display>(
    ids: impl IntoIterator<Item = T>,
) -> Result<BTreeSet<T>, AppError> {
    let mut set = BTreeSet::new();
    for id in ids {
        if !set.insert(id) {
            return Err(AppError::InvalidEdit {
                reason: format!("{id} is listed twice: list each item once"),
            });
        }
    }
    if set.is_empty() {
        return Err(AppError::InvalidEdit {
            reason: "nothing to edit: list at least one item".to_owned(),
        });
    }
    Ok(set)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit_plan::testing::{V1, clip, item_id, project_with, track_id};

    #[test]
    fn finds_items_and_their_tracks() {
        let project = project_with(vec![clip(1, 0, 2, None)], Vec::new(), Vec::new());
        let place = find_item(&project, item_id(1)).unwrap();
        assert_eq!(place.track.id, track_id(V1));
        assert!(matches!(
            find_item(&project, item_id(9)),
            Err(AppError::Domain(DomainError::ItemNotFound { .. }))
        ));
    }

    #[test]
    fn an_edit_needs_one_sequence_and_distinct_ids() {
        let project = project_with(vec![clip(1, 0, 2, None)], Vec::new(), Vec::new());
        let sequence = sequence_of(&project, [item_id(1)], [track_id(V1)]).unwrap();
        assert_eq!(sequence, project.sequences[0].id);
        assert!(sequence_of(&project, [], []).is_err());
        assert!(distinct([item_id(1), item_id(1)]).is_err());
        assert!(distinct::<ItemId>([]).is_err());
        assert_eq!(distinct([item_id(2), item_id(1)]).unwrap().len(), 2);
    }
}
