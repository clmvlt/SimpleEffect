//! Finding entities by id: positions in the document tree, or a "not found" error.

use crate::{
    AssetId, DomainError, Item, ItemId, MarkerId, Project, Sequence, SequenceId, Track, TrackId,
};

/// Where a track sits: indices into `project.sequences` and `sequence.tracks`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TrackLocation {
    /// Index of the sequence.
    pub(crate) sequence: usize,
    /// Index of the track in its sequence.
    pub(crate) track: usize,
}

impl TrackLocation {
    /// The track.
    pub(crate) fn get(self, project: &Project) -> &Track {
        &project.sequences[self.sequence].tracks[self.track]
    }

    /// The track, mutably.
    pub(crate) fn get_mut(self, project: &mut Project) -> &mut Track {
        &mut project.sequences[self.sequence].tracks[self.track]
    }
}

/// Where an item sits: its track and its index in the track.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ItemLocation {
    /// The track holding the item.
    pub(crate) track: TrackLocation,
    /// Index of the item in its track.
    pub(crate) index: usize,
}

impl ItemLocation {
    /// The item.
    pub(crate) fn get(self, project: &Project) -> &Item {
        &self.track.get(project).items[self.index]
    }
}

/// The location of item `id`.
pub(crate) fn find_item(project: &Project, id: ItemId) -> Result<ItemLocation, DomainError> {
    for (sequence_index, sequence) in project.sequences.iter().enumerate() {
        for (track_index, track) in sequence.tracks.iter().enumerate() {
            if let Some(index) = track.items.iter().position(|item| item.id == id) {
                let track = TrackLocation {
                    sequence: sequence_index,
                    track: track_index,
                };
                return Ok(ItemLocation { track, index });
            }
        }
    }
    Err(DomainError::ItemNotFound { item: id })
}

/// The location of track `id`.
pub(crate) fn find_track(project: &Project, id: TrackId) -> Result<TrackLocation, DomainError> {
    for (sequence_index, sequence) in project.sequences.iter().enumerate() {
        if let Some(track_index) = sequence.tracks.iter().position(|track| track.id == id) {
            return Ok(TrackLocation {
                sequence: sequence_index,
                track: track_index,
            });
        }
    }
    Err(DomainError::TrackNotFound { track: id })
}

/// The index of sequence `id`.
pub(crate) fn find_sequence(project: &Project, id: SequenceId) -> Result<usize, DomainError> {
    project
        .sequences
        .iter()
        .position(|sequence| sequence.id == id)
        .ok_or(DomainError::SequenceNotFound { sequence: id })
}

/// The indices of marker `id`: its sequence and its position in the sequence's markers.
pub(crate) fn find_marker(project: &Project, id: MarkerId) -> Result<(usize, usize), DomainError> {
    for (sequence_index, sequence) in project.sequences.iter().enumerate() {
        if let Some(marker_index) = sequence.markers.iter().position(|marker| marker.id == id) {
            return Ok((sequence_index, marker_index));
        }
    }
    Err(DomainError::MarkerNotFound { marker: id })
}

/// The index of asset `id`.
pub(crate) fn find_asset(project: &Project, id: AssetId) -> Result<usize, DomainError> {
    project
        .assets
        .iter()
        .position(|asset| asset.id == id)
        .ok_or(DomainError::AssetNotFound { asset: id })
}

/// Every item of the project.
pub(crate) fn all_items(project: &Project) -> impl Iterator<Item = &Item> {
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.items)
}

/// The tracks of `sequence` a sequence-wide edit (ripple delete, insert gap) works on, in
/// sequence order: the listed ones (each must belong to the sequence and be unlocked), or every
/// unlocked track when `tracks` is `None`.
pub(crate) fn edited_tracks(
    sequence: &Sequence,
    tracks: Option<&[TrackId]>,
) -> Result<Vec<TrackId>, DomainError> {
    let Some(listed) = tracks else {
        return Ok(sequence
            .tracks
            .iter()
            .filter(|track| !track.locked)
            .map(|track| track.id)
            .collect());
    };
    for id in listed {
        let track = sequence.tracks.iter().find(|track| track.id == *id).ok_or(
            DomainError::TrackNotInSequence {
                track: *id,
                sequence: sequence.id,
            },
        )?;
        super::checks::ensure_unlocked(track)?;
    }
    Ok(sequence
        .tracks
        .iter()
        .filter(|track| listed.contains(&track.id))
        .map(|track| track.id)
        .collect())
}
