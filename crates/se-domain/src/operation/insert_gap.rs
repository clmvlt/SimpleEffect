//! [`InsertGap`]: opens empty time on tracks.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    AddItem, Edit, Effect, Operation, RestoreItem,
    effect::run_steps,
    lookup::{edited_tracks, find_sequence},
    reshape::{cut_pieces, shifted},
    transition_upkeep::{TransitionSnapshot, restore_transitions},
};
use crate::{DomainError, Flicks, ItemId, Project, SequenceId, Track, TrackId};

/// Opens `duration` of empty time at `at` on tracks of a sequence: items starting at or after
/// `at` move right; an item crossing `at` is split there (new right part id from `split_ids`,
/// like [`super::SplitItem`]) and its right part moves right. Markers do not move. A transition
/// on a cut at `at` is removed (its items no longer touch); others follow their items.
/// Inverse: the reverse steps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct InsertGap {
    /// The sequence edited.
    pub sequence: SequenceId,
    /// Where the gap opens.
    pub at: Flicks,
    /// Its length (positive).
    pub duration: Flicks,
    /// The tracks edited (unlocked, of this sequence); `null` for every unlocked track.
    pub tracks: Option<Vec<TrackId>>,
    /// Id of the new right part of each item crossing `at`; other entries are ignored.
    pub split_ids: BTreeMap<ItemId, ItemId>,
}

impl Edit for InsertGap {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        if self.duration <= Flicks::ZERO {
            return Err(DomainError::EmptyDuration {
                operation: "insertGap",
            });
        }
        let sequence = &project.sequences[find_sequence(project, self.sequence)?];
        let mut steps = Vec::new();
        for id in edited_tracks(sequence, self.tracks.as_deref())? {
            if let Some(track) = sequence.tracks.iter().find(|track| track.id == id) {
                steps.extend(self.track_steps(track)?);
            }
        }
        let before = TransitionSnapshot::of(project);
        run_steps(project, &steps, effect)?;
        let renamed: Vec<_> = self
            .split_ids
            .iter()
            .map(|(cut, part)| (*cut, *part))
            .collect();
        restore_transitions(project, &before, &renamed, effect)
    }
}

impl InsertGap {
    /// The primitive steps editing `track`, ordered so that every intermediate state is valid:
    /// right moves from the last item backwards, then the split of the crossing item.
    fn track_steps(&self, track: &Track) -> Result<Vec<Operation>, DomainError> {
        let mut moves = Vec::new();
        let mut splits = Vec::new();
        for item in track.items.iter().rev() {
            let range = item.timeline_range;
            let restore = |item| {
                Operation::from(RestoreItem {
                    track: track.id,
                    item,
                })
            };
            if range.start() >= self.at {
                moves.push(restore(shifted(item, self.duration)?));
            } else if range.end() > self.at {
                let new_id = self
                    .split_ids
                    .get(&item.id)
                    .copied()
                    .ok_or(DomainError::MissingSplitId { item: item.id })?;
                let right_at = self
                    .at
                    .checked_add(self.duration)
                    .ok_or(DomainError::TimeOutOfRange { item: item.id })?;
                let (left, right) = cut_pieces(item, self.at, self.at, right_at, new_id)?;
                splits.push(restore(left));
                splits.push(
                    AddItem {
                        track: track.id,
                        item: right,
                    }
                    .into(),
                );
            }
        }
        moves.append(&mut splits);
        Ok(moves)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        operation::testing::{A2, SEQUENCE, V2, V3, apply_err, apply_ok, clip, example, find},
        test_support::{millis, range_ms},
    };

    fn gap(at_ms: i64, duration_ms: i64, tracks: Option<Vec<TrackId>>) -> InsertGap {
        InsertGap {
            sequence: SEQUENCE,
            at: millis(at_ms),
            duration: millis(duration_ms),
            tracks,
            split_ids: BTreeMap::new(),
        }
    }

    #[test]
    fn pushes_later_items_right() {
        // V3: template [2 s, 5 s), shape [6 s, 8 s), caption [8 s, 10 s).
        let applied = apply_ok(&example(), gap(6000, 1000, Some(vec![V3])));
        assert_eq!(
            find(&applied.project, 5).timeline_range,
            range_ms(2000, 3000)
        );
        assert_eq!(
            find(&applied.project, 6).timeline_range,
            range_ms(7000, 2000)
        );
        assert_eq!(
            find(&applied.project, 7).timeline_range,
            range_ms(9000, 2000)
        );
    }

    #[test]
    fn splits_the_crossing_item() {
        // Music on A2: [0 s, 14 s); text 3 on V2: [1 s, 4 s) with opacity keyframes.
        let mut insert = gap(2000, 500, Some(vec![A2, V2]));
        assert!(matches!(
            apply_err(&example(), insert.clone()),
            DomainError::MissingSplitId { .. }
        ));
        insert.split_ids = [
            (ItemId::from_test_index(9), ItemId::from_test_index(100)),
            (ItemId::from_test_index(3), ItemId::from_test_index(101)),
        ]
        .into();
        let applied = apply_ok(&example(), insert);
        assert_eq!(find(&applied.project, 9).timeline_range, range_ms(0, 2000));
        let right = find(&applied.project, 100);
        assert_eq!(right.timeline_range, range_ms(2500, 12_000));
        assert_eq!(clip(right).source_start, millis(2000));
        assert_eq!(
            find(&applied.project, 101).timeline_range,
            range_ms(2500, 2000)
        );
        assert_eq!(
            find(&applied.project, 4).timeline_range,
            range_ms(5500, 2000)
        );
    }

    #[test]
    fn needs_a_positive_duration() {
        assert!(matches!(
            apply_err(&example(), gap(0, 0, None)),
            DomainError::EmptyDuration {
                operation: "insertGap"
            }
        ));
    }
}
