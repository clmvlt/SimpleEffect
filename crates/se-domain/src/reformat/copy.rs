//! [`copy_sequence`]: a sequence copied with fresh ids, independent of its original.

use std::collections::BTreeMap;

use crate::{
    DomainError, Item, ItemId, LinkGroupId, Marker, MarkerId, Sequence, SequenceId, Track, TrackId,
    Transition, TransitionId,
};

/// Hands out the ids of a copied sequence's entities (se-domain never generates ids: se-app
/// implements this over its `IdGenerator`, tests with [`CountingIds`]).
pub trait SequenceIds {
    /// Why an id could not be made (refusals of the domain convert into it).
    type Error: From<DomainError>;
    /// A new sequence id.
    ///
    /// # Errors
    /// The generator failed.
    fn sequence(&mut self) -> Result<SequenceId, Self::Error>;
    /// A new track id.
    ///
    /// # Errors
    /// The generator failed.
    fn track(&mut self) -> Result<TrackId, Self::Error>;
    /// A new item id.
    ///
    /// # Errors
    /// The generator failed.
    fn item(&mut self) -> Result<ItemId, Self::Error>;
    /// A new transition id.
    ///
    /// # Errors
    /// The generator failed.
    fn transition(&mut self) -> Result<TransitionId, Self::Error>;
    /// A new marker id.
    ///
    /// # Errors
    /// The generator failed.
    fn marker(&mut self) -> Result<MarkerId, Self::Error>;
    /// A new link group id.
    ///
    /// # Errors
    /// The generator failed.
    fn link_group(&mut self) -> Result<LinkGroupId, Self::Error>;
}

/// `source` copied under `name` with a fresh id for the sequence and for each of its tracks,
/// items, transitions, markers and link groups: items linked together stay linked together (in
/// a group of their own), transitions join the copies of their items, items start at revision 0.
/// Everything else (times, content, origins, track states, format) is the same.
///
/// # Errors
/// The id generator failed.
pub fn copy_sequence<I: SequenceIds>(
    source: &Sequence,
    name: String,
    ids: &mut I,
) -> Result<Sequence, I::Error> {
    let mut groups: BTreeMap<LinkGroupId, LinkGroupId> = BTreeMap::new();
    let mut tracks = Vec::with_capacity(source.tracks.len());
    for track in &source.tracks {
        tracks.push(copy_track(track, ids, &mut groups)?);
    }
    let mut markers = Vec::with_capacity(source.markers.len());
    for marker in &source.markers {
        markers.push(Marker {
            id: ids.marker()?,
            ..marker.clone()
        });
    }
    Ok(Sequence {
        id: ids.sequence()?,
        name,
        format: source.format,
        tracks,
        markers,
    })
}

fn copy_track<I: SequenceIds>(
    track: &Track,
    ids: &mut I,
    groups: &mut BTreeMap<LinkGroupId, LinkGroupId>,
) -> Result<Track, I::Error> {
    let mut item_ids: BTreeMap<ItemId, ItemId> = BTreeMap::new();
    let mut items = Vec::with_capacity(track.items.len());
    for item in &track.items {
        let id = ids.item()?;
        item_ids.insert(item.id, id);
        let link_group = match item.link_group {
            Some(group) => Some(copied_group(group, ids, groups)?),
            None => None,
        };
        items.push(Item {
            id,
            revision: 0,
            link_group,
            ..item.clone()
        });
    }
    let mut transitions = Vec::with_capacity(track.transitions.len());
    for transition in &track.transitions {
        transitions.push(Transition {
            id: ids.transition()?,
            left_item: renamed(&item_ids, transition.left_item),
            right_item: renamed(&item_ids, transition.right_item),
            ..transition.clone()
        });
    }
    Ok(Track {
        id: ids.track()?,
        items,
        transitions,
        ..track.clone()
    })
}

/// The group of the copies of `group`'s items (made the first time it is met).
fn copied_group<I: SequenceIds>(
    group: LinkGroupId,
    ids: &mut I,
    groups: &mut BTreeMap<LinkGroupId, LinkGroupId>,
) -> Result<LinkGroupId, I::Error> {
    if let Some(copied) = groups.get(&group) {
        return Ok(*copied);
    }
    let copied = ids.link_group()?;
    groups.insert(group, copied);
    Ok(copied)
}

/// The copy of item `id`; an item missing from the track (an invalid transition) keeps its id,
/// so validation reports it rather than the copy hiding it.
fn renamed(item_ids: &BTreeMap<ItemId, ItemId>, id: ItemId) -> ItemId {
    item_ids.get(&id).copied().unwrap_or(id)
}

/// Ids counting up from a base, for tests (`from_test_index`).
#[cfg(any(test, feature = "test-support"))]
#[derive(Debug, Clone)]
pub struct CountingIds {
    next: u64,
}

#[cfg(any(test, feature = "test-support"))]
impl CountingIds {
    /// Ids starting after `base` (pick one above the indices the project already uses).
    #[must_use]
    pub const fn after(base: u64) -> Self {
        Self { next: base }
    }

    const fn next(&mut self) -> u64 {
        self.next += 1;
        self.next
    }
}

#[cfg(any(test, feature = "test-support"))]
impl SequenceIds for CountingIds {
    type Error = DomainError;

    fn sequence(&mut self) -> Result<SequenceId, DomainError> {
        Ok(SequenceId::from_test_index(self.next()))
    }

    fn track(&mut self) -> Result<TrackId, DomainError> {
        Ok(TrackId::from_test_index(self.next()))
    }

    fn item(&mut self) -> Result<ItemId, DomainError> {
        Ok(ItemId::from_test_index(self.next()))
    }

    fn transition(&mut self) -> Result<TransitionId, DomainError> {
        Ok(TransitionId::from_test_index(self.next()))
    }

    fn marker(&mut self) -> Result<MarkerId, DomainError> {
        Ok(MarkerId::from_test_index(self.next()))
    }

    fn link_group(&mut self) -> Result<LinkGroupId, DomainError> {
        Ok(LinkGroupId::from_test_index(self.next()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AddSequence, apply, test_support::example_project, validate_project};

    fn copy_of_example() -> (crate::Project, Sequence) {
        let project = example_project();
        let copy = copy_sequence(
            &project.sequences[0],
            "Copie".to_owned(),
            &mut CountingIds::after(5000),
        )
        .unwrap();
        (project, copy)
    }

    #[test]
    fn a_copy_is_valid_next_to_its_original_and_shares_no_id() {
        let (project, copy) = copy_of_example();
        let source = &project.sequences[0];
        assert_eq!(copy.name, "Copie");
        assert_eq!(copy.tracks.len(), source.tracks.len());
        assert_ne!(copy.id, source.id);
        for (copied, original) in copy.tracks.iter().zip(&source.tracks) {
            assert_ne!(copied.id, original.id);
            assert_eq!(copied.items.len(), original.items.len());
            for (item, original_item) in copied.items.iter().zip(&original.items) {
                assert_ne!(item.id, original_item.id);
                assert_eq!(item.kind, original_item.kind);
                assert_eq!(item.timeline_range, original_item.timeline_range);
                assert_eq!(item.revision, 0);
                assert_eq!(
                    item.link_group.is_some(),
                    original_item.link_group.is_some()
                );
                if let (Some(copied), Some(original)) = (item.link_group, original_item.link_group)
                {
                    assert_ne!(copied, original);
                }
            }
        }
        let applied = apply(
            &project,
            &AddSequence {
                index: None,
                sequence: copy,
            }
            .into(),
        )
        .unwrap();
        assert_eq!(validate_project(&applied.project), []);
    }

    #[test]
    fn linked_items_stay_linked_and_transitions_follow_their_items() {
        let (_, copy) = copy_of_example();
        let groups: Vec<_> = copy
            .tracks
            .iter()
            .flat_map(|track| &track.items)
            .filter_map(|item| item.link_group)
            .collect();
        assert!(groups.len() >= 2);
        assert!(groups.windows(2).all(|pair| pair[0] == pair[1]));
        let video = &copy.tracks[0];
        let transition = &video.transitions[0];
        assert_eq!(transition.left_item, video.items[0].id);
        assert_eq!(transition.right_item, video.items[1].id);
    }
}
