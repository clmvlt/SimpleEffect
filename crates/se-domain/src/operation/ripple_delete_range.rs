//! [`RippleDeleteRange`]: deletes a time range and closes the gap.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    AddItem, Edit, Effect, Operation, RemoveItem, RestoreItem,
    effect::run_steps,
    lookup::{edited_tracks, find_sequence},
    reshape::{Reshape, cut_pieces, reshape, shifted},
};
use crate::{DomainError, Flicks, Item, ItemId, Project, SequenceId, TimeRange, Track, TrackId};

/// Deletes `range` from tracks of a sequence and shifts what follows left by its duration.
///
/// On each edited track: items inside the range are removed; items crossing one of its edges are
/// trimmed to it (keeping their fades); an item spanning the whole range keeps its left part and
/// gets a new right part (id from `split_ids`), like [`super::SplitItem`]; later items move left.
/// Markers do not move. Inverse: the reverse steps (restore, re-add, remove the new parts).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RippleDeleteRange {
    /// The sequence edited.
    pub sequence: SequenceId,
    /// The time deleted (positive duration).
    pub range: TimeRange,
    /// The tracks edited (unlocked, of this sequence); `null` for every unlocked track.
    pub tracks: Option<Vec<TrackId>>,
    /// Id of the new right part of each item spanning the whole range; other entries are
    /// ignored.
    pub split_ids: BTreeMap<ItemId, ItemId>,
}

impl Edit for RippleDeleteRange {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        if self.range.is_empty() {
            return Err(DomainError::EmptyDuration {
                operation: "rippleDeleteRange",
            });
        }
        let sequence = &project.sequences[find_sequence(project, self.sequence)?];
        let mut steps = Vec::new();
        for id in edited_tracks(sequence, self.tracks.as_deref())? {
            if let Some(track) = sequence.tracks.iter().find(|track| track.id == id) {
                steps.extend(self.track_steps(track)?);
            }
        }
        run_steps(project, &steps, effect)
    }
}

impl RippleDeleteRange {
    /// The primitive steps editing `track`, ordered so that every intermediate state is valid:
    /// removals, end trims, splits, then left moves in start order.
    fn track_steps(&self, track: &Track) -> Result<Vec<Operation>, DomainError> {
        let (start, end) = (self.range.start(), self.range.end());
        let mut steps = Steps::default();
        for item in &track.items {
            let range = item.timeline_range;
            let restore = |item| {
                Operation::from(RestoreItem {
                    track: track.id,
                    item,
                })
            };
            if range.end() <= start {
                continue;
            }
            if range.start() >= end {
                steps.moves.push(restore(self.moved_left(item)?));
            } else if range.start() >= start && range.end() <= end {
                steps.removals.push(Operation::from(RemoveItem {
                    item: item.id,
                    expected_revision: None,
                }));
            } else if range.end() <= end {
                let content = content(item, range.start(), start)?;
                steps
                    .trims
                    .push(restore(reshape(item, &Reshape::trim(content))?));
            } else if range.start() >= start {
                let kept = Reshape {
                    start,
                    ..Reshape::trim(content(item, end, range.end())?)
                };
                steps.moves.push(restore(reshape(item, &kept)?));
            } else {
                let new_id = self.split_id(item.id)?;
                let (left, right) = cut_pieces(item, start, end, start, new_id)?;
                steps.splits.push(restore(left));
                steps.splits.push(
                    AddItem {
                        track: track.id,
                        item: right,
                    }
                    .into(),
                );
            }
        }
        Ok(steps.into_ordered())
    }

    fn moved_left(&self, item: &Item) -> Result<Item, DomainError> {
        let delta = Flicks::ZERO
            .checked_sub(self.range.duration())
            .ok_or(DomainError::TimeOutOfRange { item: item.id })?;
        shifted(item, delta)
    }

    fn split_id(&self, item: ItemId) -> Result<ItemId, DomainError> {
        self.split_ids
            .get(&item)
            .copied()
            .ok_or(DomainError::MissingSplitId { item })
    }
}

/// `[start, end)` as the content of `item`.
fn content(item: &Item, start: Flicks, end: Flicks) -> Result<TimeRange, DomainError> {
    TimeRange::from_start_end(start, end).map_err(|_| DomainError::TimeOutOfRange { item: item.id })
}

/// The steps of one track, by phase.
#[derive(Default)]
struct Steps {
    removals: Vec<Operation>,
    trims: Vec<Operation>,
    splits: Vec<Operation>,
    moves: Vec<Operation>,
}

impl Steps {
    fn into_ordered(self) -> Vec<Operation> {
        [self.removals, self.trims, self.splits, self.moves].concat()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        operation::testing::{
            A2, SEQUENCE, V1, V2, V3, apply_err, apply_ok, clip, example, find, item_ids, lock,
        },
        test_support::{millis, range_ms},
    };

    fn ripple(start_ms: i64, duration_ms: i64, tracks: Option<Vec<TrackId>>) -> RippleDeleteRange {
        RippleDeleteRange {
            sequence: SEQUENCE,
            range: range_ms(start_ms, duration_ms),
            tracks,
            split_ids: BTreeMap::new(),
        }
    }

    #[test]
    fn removes_trims_and_closes_the_gap() {
        // V3: template [2 s, 5 s), shape [6 s, 8 s), caption [8 s, 10 s). Delete [4.5 s, 7 s).
        let applied = apply_ok(&example(), ripple(4500, 2500, Some(vec![V3])));
        assert_eq!(
            find(&applied.project, 5).timeline_range,
            range_ms(2000, 2500)
        );
        assert_eq!(
            find(&applied.project, 6).timeline_range,
            range_ms(4500, 1000)
        );
        assert_eq!(
            find(&applied.project, 7).timeline_range,
            range_ms(5500, 2000)
        );
        // Other tracks are untouched.
        assert_eq!(
            find(&applied.project, 4).timeline_range,
            range_ms(5000, 2000)
        );
    }

    #[test]
    fn items_inside_the_range_are_removed_and_edges_are_exact() {
        // V2: text [1 s, 4 s), image [5 s, 7 s). Delete exactly the image.
        let applied = apply_ok(&example(), ripple(5000, 2000, Some(vec![V2])));
        assert_eq!(item_ids(&applied.project, V2), [ItemId::from_test_index(3)]);
        let applied = apply_ok(&example(), ripple(4000, 1000, Some(vec![V2])));
        assert_eq!(
            find(&applied.project, 3).timeline_range,
            range_ms(1000, 3000)
        );
        assert_eq!(
            find(&applied.project, 4).timeline_range,
            range_ms(4000, 2000)
        );
    }

    #[test]
    fn spanning_items_are_split_with_the_given_id() {
        // Music on A2: [0 s, 14 s) from 0 s. Delete [2 s, 5 s).
        let mut delete = ripple(2000, 3000, Some(vec![A2]));
        let error = apply_err(&example(), delete.clone());
        assert_eq!(
            error,
            DomainError::MissingSplitId {
                item: ItemId::from_test_index(9)
            }
        );
        delete
            .split_ids
            .insert(ItemId::from_test_index(9), ItemId::from_test_index(100));
        let applied = apply_ok(&example(), delete);
        assert_eq!(find(&applied.project, 9).timeline_range, range_ms(0, 2000));
        let right = find(&applied.project, 100);
        assert_eq!(right.timeline_range, range_ms(2000, 9000));
        assert_eq!(clip(right).source_start, millis(5000));
    }

    #[test]
    fn every_unlocked_track_by_default() {
        let project = lock(example(), V1);
        let mut delete = ripple(10_000, 1000, None);
        delete
            .split_ids
            .insert(ItemId::from_test_index(9), ItemId::from_test_index(100));
        let applied = apply_ok(&project, delete);
        assert_eq!(
            find(&applied.project, 2).timeline_range,
            range_ms(10_000, 4000)
        );
        assert_eq!(
            find(&applied.project, 100).timeline_range,
            range_ms(10_000, 3000)
        );
    }

    #[test]
    fn listed_tracks_must_be_unlocked_and_in_the_sequence() {
        let error = apply_err(&lock(example(), V2), ripple(0, 1000, Some(vec![V2])));
        assert!(matches!(error, DomainError::TrackLocked { .. }));
        let error = apply_err(
            &example(),
            ripple(0, 1000, Some(vec![TrackId::from_test_index(99)])),
        );
        assert!(matches!(error, DomainError::TrackNotInSequence { .. }));
        let error = apply_err(&example(), ripple(0, 0, None));
        assert!(matches!(error, DomainError::EmptyDuration { .. }));
    }
}
