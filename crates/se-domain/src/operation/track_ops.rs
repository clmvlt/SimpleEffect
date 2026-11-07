//! Track operations: [`AddTrack`], [`RemoveTrack`], [`UpdateTrack`], [`ReorderTracks`].

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    Edit, Effect,
    checks::{ensure_name, ensure_unlocked, ensure_unused, insertion_index, swap_field},
    lookup::{find_sequence, find_track},
};
use crate::{DomainError, Project, SequenceId, Track, TrackId};

/// Adds a track (with its items, if any) to a sequence. Video tracks stack in list order, the
/// first one at the bottom. Inverse: [`RemoveTrack`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AddTrack {
    /// The sequence receiving the track.
    pub sequence: SequenceId,
    /// Position in the sequence's tracks (`null` appends, i.e. on top).
    pub index: Option<usize>,
    /// The track, with an unused id and a non-empty name.
    pub track: Track,
}

impl Edit for AddTrack {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let sequence_index = find_sequence(project, self.sequence)?;
        ensure_unused(
            find_track(project, self.track.id).is_ok(),
            "track",
            &self.track.id,
        )?;
        ensure_name("track", &self.track.name)?;
        let tracks = &mut project.sequences[sequence_index].tracks;
        let index = insertion_index("track", self.index, tracks.len())?;
        tracks.insert(index, self.track.clone());
        effect.touched.sequences.insert(self.sequence);
        effect.touched.tracks.insert(self.track.id);
        effect
            .touched
            .items
            .extend(self.track.items.iter().map(|item| item.id));
        effect.undo_with(RemoveTrack {
            track: self.track.id,
        });
        Ok(())
    }
}

/// Removes an unlocked track and its items. Inverse: [`AddTrack`] at the same position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RemoveTrack {
    /// The track removed.
    pub track: TrackId,
}

impl Edit for RemoveTrack {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let location = find_track(project, self.track)?;
        ensure_unlocked(location.get(project))?;
        let sequence = &mut project.sequences[location.sequence];
        let track = sequence.tracks.remove(location.track);
        effect.touched.sequences.insert(sequence.id);
        effect.touched.tracks.insert(track.id);
        effect
            .touched
            .items
            .extend(track.items.iter().map(|item| item.id));
        effect.undo_with(AddTrack {
            sequence: sequence.id,
            index: Some(location.track),
            track,
        });
        Ok(())
    }
}

/// Changes of a track header; `null` (or absent) fields are unchanged. The kind of a track
/// cannot change (its items depend on it).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(optional_fields)]
pub struct TrackPatch {
    /// Another name (not empty).
    pub name: Option<String>,
    /// Mute or unmute.
    pub muted: Option<bool>,
    /// Solo or unsolo.
    pub solo: Option<bool>,
    /// Lock or unlock.
    pub locked: Option<bool>,
    /// Hide or show.
    pub hidden: Option<bool>,
    /// Another gain in decibels.
    pub volume_db: Option<f64>,
    /// Another stereo balance (−1 to 1).
    pub pan: Option<f64>,
}

impl TrackPatch {
    /// Applies the patch and returns the one undoing it.
    fn apply(&self, track: &mut Track) -> Self {
        Self {
            name: swap_field(&mut track.name, self.name.as_ref()),
            muted: swap_field(&mut track.muted, self.muted.as_ref()),
            solo: swap_field(&mut track.solo, self.solo.as_ref()),
            locked: swap_field(&mut track.locked, self.locked.as_ref()),
            hidden: swap_field(&mut track.hidden, self.hidden.as_ref()),
            volume_db: swap_field(&mut track.volume_db, self.volume_db.as_ref()),
            pan: swap_field(&mut track.pan, self.pan.as_ref()),
        }
    }
}

/// Changes the header of a track, locked or not (this is how a track gets unlocked).
/// Inverse: [`UpdateTrack`] with the previous values of the changed fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTrack {
    /// The track changed.
    pub track: TrackId,
    /// What changes.
    pub patch: TrackPatch,
}

impl Edit for UpdateTrack {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let location = find_track(project, self.track)?;
        if let Some(name) = &self.patch.name {
            ensure_name("track", name)?;
        }
        let previous = self.patch.apply(location.get_mut(project));
        effect.touched.tracks.insert(self.track);
        effect.undo_with(Self {
            track: self.track,
            patch: previous,
        });
        Ok(())
    }
}

/// Reorders the tracks of a sequence (video tracks stack in list order).
/// Inverse: [`ReorderTracks`] with the previous order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ReorderTracks {
    /// The sequence.
    pub sequence: SequenceId,
    /// Every track of the sequence, once each, in the new order.
    pub order: Vec<TrackId>,
}

impl Edit for ReorderTracks {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let sequence_index = find_sequence(project, self.sequence)?;
        let sequence = &mut project.sequences[sequence_index];
        let previous: Vec<TrackId> = sequence.tracks.iter().map(|track| track.id).collect();
        let mut sorted_new = self.order.clone();
        sorted_new.sort();
        let mut sorted_previous = previous.clone();
        sorted_previous.sort();
        if sorted_new != sorted_previous {
            return Err(DomainError::InvalidTrackOrder {
                sequence: self.sequence,
                track_count: previous.len(),
            });
        }
        sequence
            .tracks
            .sort_by_key(|track| self.order.iter().position(|id| *id == track.id));
        effect.touched.sequences.insert(self.sequence);
        effect.undo_with(Self {
            sequence: self.sequence,
            order: previous,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ItemId, TrackKind,
        operation::testing::{A1, A2, SEQUENCE, V1, V2, V3, apply_err, apply_ok, example, lock},
        test_support::track,
    };

    fn track_ids(project: &Project) -> Vec<TrackId> {
        project.sequences[0]
            .tracks
            .iter()
            .map(|track| track.id)
            .collect()
    }

    #[test]
    fn adds_a_track_at_a_position_and_removes_it_back() {
        let new = track(10, TrackKind::Video, "V4", Vec::new());
        let add = AddTrack {
            sequence: SEQUENCE,
            index: Some(3),
            track: new,
        };
        let applied = apply_ok(&example(), add.clone());
        let new_id = TrackId::from_test_index(10);
        assert_eq!(track_ids(&applied.project), [V1, V2, V3, new_id, A1, A2]);
        assert!(matches!(
            apply_err(&applied.project, add),
            DomainError::IdAlreadyUsed { kind: "track", .. }
        ));
        let mut far = AddTrack {
            sequence: SEQUENCE,
            index: Some(9),
            track: track(11, TrackKind::Video, "V5", Vec::new()),
        };
        assert!(matches!(
            apply_err(&example(), far.clone()),
            DomainError::IndexOutOfBounds { len: 5, .. }
        ));
        far.track.name = " ".to_owned();
        far.index = None;
        assert!(matches!(
            apply_err(&example(), far),
            DomainError::EmptyName { entity: "track" }
        ));
    }

    #[test]
    fn removing_a_track_removes_its_items_unless_locked() {
        let applied = apply_ok(&example(), RemoveTrack { track: V2 });
        assert_eq!(track_ids(&applied.project), [V1, V3, A1, A2]);
        assert!(applied.touched.items.contains(&ItemId::from_test_index(3)));
        assert!(matches!(
            apply_err(&lock(example(), V2), RemoveTrack { track: V2 }),
            DomainError::TrackLocked { .. }
        ));
    }

    #[test]
    fn updates_only_the_given_fields_even_when_locked() {
        let patch = TrackPatch {
            locked: Some(false),
            name: Some("Titles".to_owned()),
            ..TrackPatch::default()
        };
        let applied = apply_ok(&lock(example(), V2), UpdateTrack { track: V2, patch });
        let header = &applied.project.sequences[0].tracks[1];
        assert_eq!((header.name.as_str(), header.locked), ("Titles", false));
        assert!(!header.muted);
        let crate::Operation::UpdateTrack(inverse) = &applied.inverse[0] else {
            panic!("expected an inverse update");
        };
        assert_eq!(inverse.patch.locked, Some(true));
        assert_eq!(inverse.patch.muted, None);
    }

    #[test]
    fn reorders_with_a_permutation_only() {
        let order = vec![V3, V1, V2, A2, A1];
        let applied = apply_ok(
            &example(),
            ReorderTracks {
                sequence: SEQUENCE,
                order: order.clone(),
            },
        );
        assert_eq!(track_ids(&applied.project), order);
        for wrong in [vec![V1, V2], vec![V1, V1, V2, A1, A2]] {
            assert!(matches!(
                apply_err(
                    &example(),
                    ReorderTracks {
                        sequence: SEQUENCE,
                        order: wrong
                    }
                ),
                DomainError::InvalidTrackOrder { track_count: 5, .. }
            ));
        }
    }
}
