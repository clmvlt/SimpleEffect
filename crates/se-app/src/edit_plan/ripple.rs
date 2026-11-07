//! Ripple edits: the tracks they shift, opening and removing time, ripple deletes (ADR-0021).
//!
//! A ripple edit shifts what follows an edit point on the **concerned tracks**: the tracks it
//! edits, plus every unlocked track holding an item linked to an item that moves, so linked
//! picture and sound never drift apart. Other tracks stay where they are. Removing time is
//! refused when a concerned track holds other content there, rather than cutting it silently.

use std::collections::{BTreeMap, BTreeSet};

use se_domain::{
    Flicks, InsertGap, ItemId, LinkGroupId, RippleDeleteRange, Sequence, SequenceId, TimeRange,
    TrackId,
};

use super::{EditPlan, find_item, find_sequence};
use crate::AppError;

/// The tracks a ripple edit of `seeds` at `from` shifts: `seeds`, then — until nothing is added
/// — every unlocked track holding an item linked to an item starting at or after `from` on a
/// track already listed. Locked tracks never move: a locked seed is refused by the domain, a
/// locked partner track is left out (the user locked it).
pub(crate) fn concerned_tracks(
    sequence: &Sequence,
    seeds: &BTreeSet<TrackId>,
    from: Flicks,
) -> BTreeSet<TrackId> {
    let mut tracks = seeds.clone();
    loop {
        let moving_groups: BTreeSet<LinkGroupId> = sequence
            .tracks
            .iter()
            .filter(|track| tracks.contains(&track.id))
            .flat_map(|track| &track.items)
            .filter(|item| item.timeline_range.start() >= from)
            .filter_map(|item| item.link_group)
            .collect();
        let added: Vec<TrackId> = sequence
            .tracks
            .iter()
            .filter(|track| !track.locked && !tracks.contains(&track.id))
            .filter(|track| {
                track.items.iter().any(|item| {
                    item.link_group
                        .is_some_and(|group| moving_groups.contains(&group))
                })
            })
            .map(|track| track.id)
            .collect();
        if added.is_empty() {
            return tracks;
        }
        tracks.extend(added);
    }
}

/// Opens `duration` of empty time at `at` on `tracks` (later items move right; an item crossing
/// `at` is cut there, its right part getting a new id).
///
/// # Errors
/// Unknown sequence, a locked or foreign track, times out of range.
pub(crate) fn open_time(
    plan: &mut EditPlan<'_>,
    sequence_id: SequenceId,
    at: Flicks,
    duration: Flicks,
    tracks: &BTreeSet<TrackId>,
) -> Result<(), AppError> {
    if duration <= Flicks::ZERO || tracks.is_empty() {
        return Ok(());
    }
    let sequence = find_sequence(plan.project()?, sequence_id)?;
    let crossing: Vec<ItemId> = sequence
        .tracks
        .iter()
        .filter(|track| tracks.contains(&track.id))
        .flat_map(|track| &track.items)
        .filter(|item| item.timeline_range.start() < at && item.timeline_range.end() > at)
        .map(|item| item.id)
        .collect();
    let mut split_ids = BTreeMap::new();
    for item in crossing {
        split_ids.insert(item, plan.ids().item()?);
    }
    plan.push(InsertGap {
        sequence: sequence_id,
        at,
        duration,
        tracks: Some(tracks.iter().copied().collect()),
        split_ids,
    });
    Ok(())
}

/// Removes `range` from `tracks` and closes it (later items move left). Only items of `allowed`
/// may be there (they are removed, or trimmed when they cross an edge).
///
/// # Errors
/// [`AppError::RippleBlocked`] naming the first track holding anything else in `range`; a locked
/// or foreign track.
pub(crate) fn remove_time(
    plan: &mut EditPlan<'_>,
    sequence_id: SequenceId,
    range: TimeRange,
    tracks: &BTreeSet<TrackId>,
    allowed: &BTreeSet<ItemId>,
) -> Result<(), AppError> {
    if range.is_empty() || tracks.is_empty() {
        return Ok(());
    }
    let sequence = find_sequence(plan.project()?, sequence_id)?;
    for track in sequence.tracks.iter().filter(|t| tracks.contains(&t.id)) {
        let blocking = track
            .items
            .iter()
            .filter(|item| !allowed.contains(&item.id))
            .find_map(|item| item.timeline_range.intersection(range));
        if let Some(overlap) = blocking {
            return Err(AppError::RippleBlocked {
                track_name: track.name.clone(),
                start: overlap.start(),
                end: overlap.end(),
            });
        }
    }
    plan.push(RippleDeleteRange {
        sequence: sequence_id,
        range,
        tracks: Some(tracks.iter().copied().collect()),
        split_ids: BTreeMap::new(),
    });
    Ok(())
}

/// Tracks shifted together by a ripple delete and the time they lose.
#[derive(Debug, Clone)]
struct RippleGroup {
    tracks: BTreeSet<TrackId>,
    ranges: Vec<TimeRange>,
}

impl RippleGroup {
    /// The earliest removed instant.
    fn from(&self) -> Flicks {
        self.ranges
            .iter()
            .map(|range| range.start())
            .min()
            .unwrap_or(Flicks::ZERO)
    }

    fn absorb(&mut self, other: Self) {
        self.tracks.extend(other.tracks);
        self.ranges.extend(other.ranges);
    }
}

/// Removes `items` and closes the time they took. Tracks are grouped: the tracks of the items
/// plus their concerned tracks, groups sharing a track merged, so every track of a group loses
/// the same time (the union of its items' ranges) and linked items stay in sync. A group track
/// holding other content in that time blocks the edit.
///
/// # Errors
/// Unknown items, [`AppError::RippleBlocked`], locked tracks.
pub(crate) fn ripple_delete(
    plan: &mut EditPlan<'_>,
    sequence_id: SequenceId,
    items: &BTreeSet<ItemId>,
) -> Result<(), AppError> {
    let project = plan.project()?;
    let mut by_track: BTreeMap<TrackId, Vec<TimeRange>> = BTreeMap::new();
    for &id in items {
        let place = find_item(project, id)?;
        by_track
            .entry(place.track.id)
            .or_default()
            .push(place.item.timeline_range);
    }
    let initial = by_track
        .into_iter()
        .map(|(track, ranges)| RippleGroup {
            tracks: BTreeSet::from([track]),
            ranges,
        })
        .collect();
    let groups = ripple_groups(find_sequence(project, sequence_id)?, initial);
    for group in groups {
        for range in merged(group.ranges).into_iter().rev() {
            remove_time(plan, sequence_id, range, &group.tracks, items)?;
        }
    }
    Ok(())
}

/// `groups` extended to their concerned tracks and merged until no two share a track.
fn ripple_groups(sequence: &Sequence, mut groups: Vec<RippleGroup>) -> Vec<RippleGroup> {
    loop {
        for group in &mut groups {
            group.tracks = concerned_tracks(sequence, &group.tracks, group.from());
        }
        let count = groups.len();
        groups = merge_sharing_tracks(groups);
        if groups.len() == count {
            return groups;
        }
    }
}

fn merge_sharing_tracks(groups: Vec<RippleGroup>) -> Vec<RippleGroup> {
    let mut merged: Vec<RippleGroup> = Vec::new();
    for mut group in groups {
        let (sharing, apart): (Vec<_>, Vec<_>) = merged
            .into_iter()
            .partition(|other| !other.tracks.is_disjoint(&group.tracks));
        for other in sharing {
            group.absorb(other);
        }
        merged = apart;
        merged.push(group);
    }
    merged
}

/// `ranges` sorted, overlapping or touching ones joined.
fn merged(mut ranges: Vec<TimeRange>) -> Vec<TimeRange> {
    ranges.sort_by_key(|range| range.start());
    let mut joined: Vec<TimeRange> = Vec::with_capacity(ranges.len());
    for range in ranges {
        match joined.last_mut() {
            Some(last) if range.start() <= last.end() => {
                if range.end() > last.end()
                    && let Ok(longer) = TimeRange::from_start_end(last.start(), range.end())
                {
                    *last = longer;
                }
            }
            _ => joined.push(range),
        }
    }
    joined
}

#[cfg(test)]
mod tests {
    use se_domain::TrackPatch;

    use super::*;
    use crate::{
        edit_plan::testing::{A1, V1, V2, clip, item_id, project_with, secs, spans, track_id},
        new_ids::NewIds,
        test_support::SequentialIds,
    };

    fn set(indices: &[u64]) -> BTreeSet<TrackId> {
        indices.iter().copied().map(track_id).collect()
    }

    #[test]
    fn linked_partners_of_moving_items_join_the_concerned_tracks() {
        let project = project_with(
            vec![clip(1, 0, 2, Some(1)), clip(2, 4, 6, Some(2))],
            vec![clip(3, 5, 7, None)],
            vec![clip(4, 0, 2, Some(1)), clip(5, 4, 6, Some(2))],
        );
        let sequence = &project.sequences[0];
        assert_eq!(
            concerned_tracks(sequence, &set(&[V1]), secs(3)),
            set(&[V1, A1])
        );
        assert_eq!(concerned_tracks(sequence, &set(&[V1]), secs(7)), set(&[V1]));
        let mut locked = project.clone();
        locked.sequences[0].tracks[2].locked = true;
        assert_eq!(
            concerned_tracks(&locked.sequences[0], &set(&[V1]), secs(3)),
            set(&[V1])
        );
    }

    #[test]
    fn ripple_delete_closes_the_time_on_linked_tracks_only() {
        let project = project_with(
            vec![clip(1, 0, 2, Some(1)), clip(2, 2, 5, Some(2))],
            vec![clip(3, 3, 4, None)],
            vec![clip(4, 0, 2, Some(1)), clip(5, 2, 5, Some(2))],
        );
        let ids = SequentialIds::default();
        let mut plan = EditPlan::new(&project, NewIds(&ids));
        let deleted = BTreeSet::from([item_id(1), item_id(4)]);
        ripple_delete(&mut plan, project.sequences[0].id, &deleted).unwrap();
        let project = plan.project().unwrap();
        assert_eq!(spans(project, V1), [(0, 3)]);
        assert_eq!(spans(project, A1), [(0, 3)]);
        assert_eq!(spans(project, V2), [(3, 4)]);
    }

    #[test]
    fn ripple_delete_refuses_to_cut_other_content_of_a_concerned_track() {
        let project = project_with(
            vec![clip(1, 0, 2, None), clip(2, 2, 5, Some(2))],
            Vec::new(),
            vec![clip(4, 0, 3, None), clip(5, 3, 5, Some(2))],
        );
        let ids = SequentialIds::default();
        let mut plan = EditPlan::new(&project, NewIds(&ids));
        let error = ripple_delete(
            &mut plan,
            project.sequences[0].id,
            &BTreeSet::from([item_id(1)]),
        )
        .unwrap_err();
        assert_eq!(
            error,
            AppError::RippleBlocked {
                track_name: "A1".to_owned(),
                start: secs(0),
                end: secs(2),
            }
        );
    }

    #[test]
    fn opening_time_cuts_crossing_items_and_pushes_later_ones() {
        let project = project_with(
            vec![clip(1, 0, 4, None), clip(2, 6, 8, None)],
            vec![clip(3, 1, 2, None)],
            Vec::new(),
        );
        let ids = SequentialIds::default();
        let mut plan = EditPlan::new(&project, NewIds(&ids));
        let sequence = project.sequences[0].id;
        open_time(&mut plan, sequence, secs(3), secs(2), &set(&[V1])).unwrap();
        let project = plan.project().unwrap();
        assert_eq!(spans(project, V1), [(0, 3), (5, 6), (8, 10)]);
        assert_eq!(spans(project, V2), [(1, 2)]);
    }

    #[test]
    fn locked_seed_tracks_are_refused_by_the_domain() {
        let project = project_with(vec![clip(1, 0, 4, None)], Vec::new(), Vec::new());
        let ids = SequentialIds::default();
        let mut plan = EditPlan::new(&project, NewIds(&ids));
        plan.push(se_domain::UpdateTrack {
            track: track_id(V1),
            patch: TrackPatch {
                locked: Some(true),
                ..TrackPatch::default()
            },
        });
        let sequence = project.sequences[0].id;
        let range = TimeRange::from_start_end(secs(0), secs(1)).unwrap();
        let allowed = BTreeSet::from([item_id(1)]);
        remove_time(&mut plan, sequence, range, &set(&[V1]), &allowed).unwrap();
        assert!(
            matches!(plan.project(), Err(AppError::Domain(_))),
            "the queued ripple fails at the next read"
        );
    }

    #[test]
    fn touching_ranges_are_merged() {
        let range = |start, end| TimeRange::from_start_end(secs(start), secs(end)).unwrap();
        assert_eq!(
            merged(vec![range(4, 6), range(0, 2), range(2, 3), range(5, 7)]),
            [range(0, 3), range(4, 7)]
        );
    }
}
