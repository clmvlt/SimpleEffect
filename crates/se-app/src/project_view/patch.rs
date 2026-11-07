//! Turning what a commit touched into whole-entity upserts and deletes.

use std::collections::BTreeMap;

use se_domain::{
    Asset, AssetId, Item, ItemId, Marker, MarkerId, Project, Sequence, SequenceId, TouchedEntities,
    Track, TrackId,
};
use se_protocol::{EntityDeletes, EntityUpserts, ItemEntity, MarkerEntity, ProjectPatch};

use super::snapshot::{project_header, sequence_entity, track_entity};

/// The patch bringing a client from `before` to `after`, given what the commit `touched`.
///
/// Parents are refreshed with their children: a moved item also refreshes the item lists of its
/// old and new tracks, a touched track or marker refreshes its sequence (before and after).
#[must_use]
pub fn project_patch(before: &Project, after: &Project, touched: &TouchedEntities) -> ProjectPatch {
    let old = Index::of(before);
    let new = Index::of(after);
    let mut upserts = EntityUpserts::default();
    let mut deletes = EntityDeletes::default();
    let mut tracks = touched.tracks.clone();
    let mut sequences = touched.sequences.clone();
    for track in &touched.tracks {
        sequences.extend(old.track_sequence(*track));
        sequences.extend(new.track_sequence(*track));
    }
    for id in &touched.items {
        tracks.extend(old.items.get(id).map(|(track, _)| *track));
        match new.items.get(id) {
            Some(&(track_id, item)) => {
                tracks.insert(track_id);
                upserts.items.push(ItemEntity {
                    track_id,
                    item: item.clone(),
                });
            }
            None => deletes.items.push(*id),
        }
    }
    for id in &touched.markers {
        sequences.extend(old.markers.get(id).map(|(sequence, _)| sequence.id));
        match new.markers.get(id) {
            Some(&(sequence, marker)) => {
                sequences.insert(sequence.id);
                upserts.markers.push(MarkerEntity {
                    sequence_id: sequence.id,
                    marker: marker.clone(),
                });
            }
            None => deletes.markers.push(*id),
        }
    }
    for id in tracks {
        match new.tracks.get(&id) {
            Some(&(sequence, track)) => upserts.tracks.push(track_entity(sequence, track)),
            None => deletes.tracks.push(id),
        }
    }
    for id in sequences {
        match new.sequences.get(&id) {
            Some(sequence) => upserts.sequences.push(sequence_entity(sequence)),
            None => deletes.sequences.push(id),
        }
    }
    for id in &touched.assets {
        match new.assets.get(id) {
            Some(asset) => upserts.assets.push((*asset).clone()),
            None => deletes.assets.push(*id),
        }
    }
    if touched.project {
        upserts.project = Some(project_header(after));
    }
    ProjectPatch {
        project_id: after.id,
        revision: after.revision,
        upserts,
        deletes,
    }
}

/// Every entity of a project by id, with its parent.
struct Index<'a> {
    sequences: BTreeMap<SequenceId, &'a Sequence>,
    tracks: BTreeMap<TrackId, (&'a Sequence, &'a Track)>,
    items: BTreeMap<ItemId, (TrackId, &'a Item)>,
    markers: BTreeMap<MarkerId, (&'a Sequence, &'a Marker)>,
    assets: BTreeMap<AssetId, &'a Asset>,
}

impl<'a> Index<'a> {
    fn of(project: &'a Project) -> Self {
        let mut index = Self {
            sequences: BTreeMap::new(),
            tracks: BTreeMap::new(),
            items: BTreeMap::new(),
            markers: BTreeMap::new(),
            assets: project
                .assets
                .iter()
                .map(|asset| (asset.id, asset))
                .collect(),
        };
        for sequence in &project.sequences {
            index.sequences.insert(sequence.id, sequence);
            for marker in &sequence.markers {
                index.markers.insert(marker.id, (sequence, marker));
            }
            for track in &sequence.tracks {
                index.tracks.insert(track.id, (sequence, track));
                for item in &track.items {
                    index.items.insert(item.id, (track.id, item));
                }
            }
        }
        index
    }

    fn track_sequence(&self, track: TrackId) -> Option<SequenceId> {
        self.tracks.get(&track).map(|(sequence, _)| sequence.id)
    }
}

/// The ids of every entity in `project` (to check patches in tests).
#[cfg(test)]
pub(crate) fn entity_count(project: &Project) -> usize {
    let index = Index::of(project);
    index.sequences.len()
        + index.tracks.len()
        + index.items.len()
        + index.markers.len()
        + index.assets.len()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use se_domain::{
        Actor, MoveItem, Operation, RemoveSequence, RenameProject, Transaction, TransactionId,
        commit,
        test_support::{example_project, millis},
    };

    use super::*;

    fn patch_of(project: &Project, op: impl Into<Operation>) -> ProjectPatch {
        let transaction = Transaction {
            id: TransactionId::from_test_index(1),
            actor: Actor::User,
            label: "Edit".to_owned(),
            ops: vec![op.into()],
        };
        let committed = commit(project, &transaction).unwrap();
        project_patch(project, &committed.project, &committed.touched)
    }

    #[test]
    fn moving_an_item_to_another_track_refreshes_both_tracks() {
        let project = example_project();
        let text = &project.sequences[0].tracks[1];
        let target = &project.sequences[0].tracks[2];
        let item = text.items[0].id;
        let patch = patch_of(
            &project,
            MoveItem {
                item,
                track: target.id,
                start: millis(60_000),
                expected_revision: None,
            },
        );
        assert_eq!(patch.revision, project.revision + 1);
        assert_eq!(patch.upserts.items.len(), 1);
        assert_eq!(patch.upserts.items[0].track_id, target.id);
        assert_eq!(patch.upserts.items[0].item.revision, 2);
        let refreshed: BTreeSet<TrackId> = patch.upserts.tracks.iter().map(|t| t.id).collect();
        assert_eq!(refreshed, BTreeSet::from([text.id, target.id]));
        let new_text = patch
            .upserts
            .tracks
            .iter()
            .find(|t| t.id == text.id)
            .unwrap();
        assert!(!new_text.item_ids.contains(&item));
        assert_eq!(patch.deletes.items, []);
        assert!(patch.upserts.project.is_none());
    }

    #[test]
    fn removing_a_sequence_deletes_it_and_everything_inside() {
        let project = example_project();
        let sequence = project.sequences[0].id;
        let patch = patch_of(&project, RemoveSequence { sequence });
        let deleted = patch.deletes.sequences.len()
            + patch.deletes.tracks.len()
            + patch.deletes.items.len()
            + patch.deletes.markers.len();
        assert_eq!(deleted, entity_count(&project) - project.assets.len());
        assert_eq!(patch.upserts.tracks, []);
        assert_eq!(patch.upserts.items, []);
    }

    #[test]
    fn renaming_the_project_upserts_its_header() {
        let project = example_project();
        let patch = patch_of(
            &project,
            RenameProject {
                name: "Nouveau".to_owned(),
            },
        );
        assert_eq!(patch.upserts.project.unwrap().name, "Nouveau");
        assert_eq!(patch.upserts.items, []);
    }
}
