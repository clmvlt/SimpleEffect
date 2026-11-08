//! [`RippleDeleteRanges`]: deletes several time ranges at once and closes them (smart cuts).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    AddItem, Edit, Effect, Operation, RemoveItem, RestoreItem,
    effect::run_steps,
    lookup::{edited_tracks, find_sequence},
    reshape::{Reshape, reshape},
    transition_upkeep::{TransitionSnapshot, restore_transitions},
};
use crate::{DomainError, Flicks, Item, ItemId, Project, SequenceId, TimeRange, Track, TrackId};

/// Deletes `ranges` from tracks of a sequence in one pass and shifts what follows left by the
/// time removed before it: the edit of text-based editing and smart cuts (prompt 22, ADR-0032).
///
/// The result equals [`super::RippleDeleteRange`] applied for each range, last range first, but
/// each item is reshaped once: an item keeps its id for its first remaining part and gets one new
/// part per range falling strictly inside it (ids from `split_ids`, in time order), with no link
/// group (the caller relinks the parts of linked items). The first part keeps the fade-in, the
/// last one the fade-out; edges made by a cut have none. Markers do not move. Transitions whose
/// items are adjacent again are kept (an outgoing transition follows the last part). Inverse:
/// the reverse steps, one snapshot per item changed (not one per range).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RippleDeleteRanges {
    /// The sequence edited.
    pub sequence: SequenceId,
    /// The times deleted: sorted by start, each positive, apart (a range ends before the next
    /// one starts).
    pub ranges: Vec<TimeRange>,
    /// The tracks edited (unlocked, of this sequence); `null` for every unlocked track.
    pub tracks: Option<Vec<TrackId>>,
    /// Ids of the new parts of each item that ranges fall strictly inside: one per such range, in
    /// time order; other entries are ignored.
    pub split_ids: BTreeMap<ItemId, Vec<ItemId>>,
}

impl Edit for RippleDeleteRanges {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let cuts = Cuts::new(&self.ranges)?;
        let sequence = &project.sequences[find_sequence(project, self.sequence)?];
        let mut steps = Vec::new();
        let mut renamed = Vec::new();
        for id in edited_tracks(sequence, self.tracks.as_deref())? {
            if let Some(track) = sequence.tracks.iter().find(|track| track.id == id) {
                steps.extend(self.track_steps(track, &cuts, &mut renamed)?);
            }
        }
        let before = TransitionSnapshot::of(project);
        run_steps(project, &steps, effect)?;
        restore_transitions(project, &before, &renamed, effect)
    }
}

impl RippleDeleteRanges {
    /// The steps editing `track`, ordered so that every intermediate state is valid: removals,
    /// then each item in start order (its first part, then its new parts). Items only move left,
    /// never past the new end of the item before them. `renamed` gets (item, its last part).
    fn track_steps(
        &self,
        track: &Track,
        cuts: &Cuts,
        renamed: &mut Vec<(ItemId, ItemId)>,
    ) -> Result<Vec<Operation>, DomainError> {
        let mut removals = Vec::new();
        let mut reshapes = Vec::new();
        for item in &track.items {
            let parts = cuts.remaining(item.timeline_range);
            let Some(first) = parts.first() else {
                removals.push(Operation::from(RemoveItem {
                    item: item.id,
                    expected_revision: None,
                }));
                continue;
            };
            if parts.len() == 1
                && *first == item.timeline_range
                && cuts.removed_before(first.start()) == Flicks::ZERO
            {
                continue;
            }
            let new_ids = self.new_ids(item, parts.len() - 1)?;
            for (index, part) in parts.iter().enumerate() {
                let piece = cuts.piece(item, *part, index == 0, index + 1 == parts.len())?;
                match index.checked_sub(1).and_then(|new| new_ids.get(new)) {
                    None => reshapes.push(Operation::from(RestoreItem {
                        track: track.id,
                        item: piece,
                    })),
                    Some(&id) => reshapes.push(Operation::from(AddItem {
                        track: track.id,
                        item: Item {
                            id,
                            revision: 0,
                            link_group: None,
                            ..piece
                        },
                    })),
                }
            }
            if let Some(&last) = new_ids.last() {
                renamed.push((item.id, last));
            }
        }
        removals.extend(reshapes);
        Ok(removals)
    }

    /// The `count` ids of the new parts of `item`.
    fn new_ids(&self, item: &Item, count: usize) -> Result<&[ItemId], DomainError> {
        if count == 0 {
            return Ok(&[]);
        }
        self.split_ids
            .get(&item.id)
            .and_then(|ids| ids.get(..count))
            .ok_or(DomainError::MissingSplitId { item: item.id })
    }
}

/// The parts of `range` that `ranges` (sorted by start, apart) do not cover, in order: what
/// [`RippleDeleteRanges`] keeps of an item over `range`, so callers know how many new ids it
/// needs (one per part after the first).
#[must_use]
pub fn remaining_parts(ranges: &[TimeRange], range: TimeRange) -> Vec<TimeRange> {
    let first = ranges.partition_point(|cut| cut.end() <= range.start());
    let mut parts = Vec::new();
    let mut cursor = range.start();
    for cut in &ranges[first..] {
        if cut.start() >= range.end() {
            break;
        }
        if cut.start() > cursor
            && let Ok(part) = TimeRange::from_start_end(cursor, cut.start())
        {
            parts.push(part);
        }
        cursor = cursor.max(cut.end());
    }
    if cursor < range.end()
        && let Ok(part) = TimeRange::from_start_end(cursor, range.end())
    {
        parts.push(part);
    }
    parts
}

/// The ranges to delete, checked, with the time removed before each range's end.
struct Cuts<'a> {
    ranges: &'a [TimeRange],
    /// `removed[i]`: total duration of `ranges[..=i]`.
    removed: Vec<Flicks>,
}

impl<'a> Cuts<'a> {
    fn new(ranges: &'a [TimeRange]) -> Result<Self, DomainError> {
        if ranges.is_empty() || ranges.iter().any(|range| range.is_empty()) {
            return Err(DomainError::EmptyDuration {
                operation: "rippleDeleteRanges",
            });
        }
        if ranges
            .windows(2)
            .any(|pair| pair[0].end() >= pair[1].start())
        {
            return Err(DomainError::RangesNotInOrder {
                operation: "rippleDeleteRanges",
            });
        }
        let mut removed = Vec::with_capacity(ranges.len());
        let mut total = Flicks::ZERO;
        for range in ranges {
            total = total
                .checked_add(range.duration())
                .ok_or(DomainError::RangesNotInOrder {
                    operation: "rippleDeleteRanges",
                })?;
            removed.push(total);
        }
        Ok(Self { ranges, removed })
    }

    /// The time deleted before `time` (ranges ending at or before it).
    fn removed_before(&self, time: Flicks) -> Flicks {
        let count = self.ranges.partition_point(|range| range.end() <= time);
        count
            .checked_sub(1)
            .and_then(|last| self.removed.get(last))
            .copied()
            .unwrap_or(Flicks::ZERO)
    }

    /// The parts of `range` no deleted range covers, in order.
    fn remaining(&self, range: TimeRange) -> Vec<TimeRange> {
        remaining_parts(self.ranges, range)
    }

    /// `item` reshaped to `part`, moved left by the time removed before it.
    fn piece(
        &self,
        item: &Item,
        part: TimeRange,
        first: bool,
        last: bool,
    ) -> Result<Item, DomainError> {
        let start = part
            .start()
            .checked_sub(self.removed_before(part.start()))
            .ok_or(DomainError::TimeOutOfRange { item: item.id })?;
        reshape(
            item,
            &Reshape {
                content: part,
                start,
                keep_fade_in: first,
                keep_fade_out: last,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ItemKind, RippleDeleteRange,
        operation::{
            apply_all,
            testing::{A2, SEQUENCE, V1, V3, apply_err, apply_ok, example, find, lock},
        },
        test_support::{millis, range_ms},
    };

    fn ranges(spans: &[(i64, i64)]) -> Vec<TimeRange> {
        spans
            .iter()
            .map(|&(start, end)| range_ms(start, end - start))
            .collect()
    }

    fn cut(spans: &[(i64, i64)], tracks: Option<Vec<TrackId>>) -> RippleDeleteRanges {
        RippleDeleteRanges {
            sequence: SEQUENCE,
            ranges: ranges(spans),
            tracks,
            split_ids: BTreeMap::new(),
        }
    }

    fn id(index: u64) -> ItemId {
        ItemId::from_test_index(index)
    }

    fn source_start(item: &Item) -> Flicks {
        match &item.kind {
            ItemKind::MediaClip(clip) => clip.source_start,
            _ => panic!("not a clip"),
        }
    }

    #[test]
    fn several_ranges_split_one_item_into_parts_in_time_order() {
        // Music on A2: [0 s, 14 s) from 0 s. Delete [2 s, 3 s) and [5 s, 7 s).
        let mut delete = cut(&[(2000, 3000), (5000, 7000)], Some(vec![A2]));
        assert_eq!(
            apply_err(&example(), delete.clone()),
            DomainError::MissingSplitId { item: id(9) }
        );
        delete.split_ids.insert(id(9), vec![id(100), id(101)]);
        let applied = apply_ok(&example(), delete);
        let project = &applied.project;
        assert_eq!(find(project, 9).timeline_range, range_ms(0, 2000));
        assert_eq!(find(project, 100).timeline_range, range_ms(2000, 2000));
        assert_eq!(source_start(find(project, 100)), millis(3000));
        assert_eq!(find(project, 101).timeline_range, range_ms(4000, 7000));
        assert_eq!(source_start(find(project, 101)), millis(7000));
        assert_eq!(find(project, 100).link_group, None);
    }

    #[test]
    fn removes_trims_and_shifts_by_the_time_removed_before() {
        // V3: template [2 s, 5 s), shape [6 s, 8 s), caption [8 s, 10 s).
        let applied = apply_ok(
            &example(),
            cut(&[(1000, 1500), (4500, 6500)], Some(vec![V3])),
        );
        let project = &applied.project;
        assert_eq!(find(project, 5).timeline_range, range_ms(1500, 2500));
        assert_eq!(find(project, 6).timeline_range, range_ms(4000, 1500));
        assert_eq!(find(project, 7).timeline_range, range_ms(5500, 2000));
        assert_eq!(find(project, 4).timeline_range, range_ms(5000, 2000));
    }

    /// Whether `cut` lies strictly inside `range` (the item must be split there).
    fn strictly_inside(cut: &TimeRange, range: TimeRange) -> bool {
        cut.start() > range.start() && cut.end() < range.end()
    }

    /// The same cuts as single ripple deletes, last range first, each item's k-th new part
    /// getting the k-th id of `together`.
    fn one_by_one(project: &Project, together: &RippleDeleteRanges) -> Vec<Operation> {
        let items: Vec<&Item> = project.items().collect();
        let mut operations = Vec::new();
        for (index, range) in together.ranges.iter().enumerate().rev() {
            let mut split_ids = BTreeMap::new();
            for item in items
                .iter()
                .filter(|item| strictly_inside(range, item.timeline_range))
            {
                let earlier = together.ranges[..index]
                    .iter()
                    .filter(|cut| strictly_inside(cut, item.timeline_range))
                    .count();
                if let Some(new) = together
                    .split_ids
                    .get(&item.id)
                    .and_then(|ids| ids.get(earlier))
                {
                    split_ids.insert(item.id, *new);
                }
            }
            operations.push(Operation::from(RippleDeleteRange {
                sequence: SEQUENCE,
                range: *range,
                tracks: together.tracks.clone(),
                split_ids,
            }));
        }
        operations
    }

    #[test]
    fn equals_single_ripple_deletes_from_the_last_range() {
        let spans = [(1000, 1500), (2000, 3000), (5000, 7000), (11_000, 12_500)];
        let mut together = cut(&spans, None);
        let mut next = 100;
        for item in example().items() {
            let ids = (0..4).map(|_| {
                next += 1;
                id(next)
            });
            together.split_ids.insert(item.id, ids.collect());
        }
        let at_once = apply_ok(&example(), together.clone()).project;
        let sequential = apply_all(&example(), &one_by_one(&example(), &together))
            .unwrap()
            .project;
        assert_eq!(at_once, sequential);
        assert_eq!(
            at_once.sequences[0].tracks[0].transitions.len(),
            1,
            "the dissolve between the interview clips is kept"
        );
    }

    #[test]
    fn refuses_unordered_overlapping_or_empty_ranges() {
        let error = apply_err(&example(), cut(&[(3000, 4000), (1000, 2000)], None));
        assert!(matches!(error, DomainError::RangesNotInOrder { .. }));
        let error = apply_err(&example(), cut(&[(1000, 2000), (2000, 3000)], None));
        assert!(matches!(error, DomainError::RangesNotInOrder { .. }));
        let error = apply_err(&example(), cut(&[], None));
        assert!(matches!(error, DomainError::EmptyDuration { .. }));
        let error = apply_err(&lock(example(), V1), cut(&[(0, 500)], Some(vec![V1])));
        assert!(matches!(error, DomainError::TrackLocked { .. }));
    }
}
