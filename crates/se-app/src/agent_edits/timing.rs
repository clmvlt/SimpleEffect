//! Timing edits of agents: moves, trims, splits and deletes that keep linked items together (as
//! the timeline does), ripple deletes and inserted gaps (which shift what follows, so the scope
//! check refuses them when that reaches outside a time-limited scope).

use std::collections::{BTreeMap, BTreeSet};

use se_domain::{
    Flicks, InsertGap, Item, ItemEdge, ItemId, LinkGroupId, LinkItems, MoveItem, RemoveItem,
    RippleDeleteRange, Sequence, SplitItem, TimeRange, TrackId, TrimItem, format_clock,
};

use super::{ItemRef, planner::AgentPlanner, scope::track_allowed};
use crate::{
    AppError,
    edit_plan::{concerned_tracks, distinct, find_item, find_sequence},
};

/// The other items of `item`'s link group in `sequence`, with their tracks.
fn partners<'s>(sequence: &'s Sequence, item: &Item) -> Vec<(TrackId, &'s Item)> {
    let Some(group) = item.link_group else {
        return Vec::new();
    };
    sequence
        .tracks
        .iter()
        .flat_map(|track| track.items.iter().map(move |other| (track.id, other)))
        .filter(|(_, other)| other.id != item.id && other.link_group == Some(group))
        .collect()
}

fn out_of_range() -> AppError {
    AppError::InvalidEdit {
        reason: "the edit would leave the supported time range".to_owned(),
    }
}

/// Moves `item` to `start` (and `track`), its linked partners by the same amount.
pub(super) fn plan_move(
    planner: &mut AgentPlanner<'_>,
    item: ItemRef,
    track: Option<TrackId>,
    start: Flicks,
    expected_revision: Option<u64>,
) -> Result<(), AppError> {
    let id = planner.resolve(item)?;
    let project = planner.plan.project()?;
    let place = find_item(project, id)?;
    let delta = start
        .checked_sub(place.item.timeline_range.start())
        .ok_or_else(out_of_range)?;
    let mut moves = vec![MoveItem {
        item: id,
        track: track.unwrap_or(place.track.id),
        start,
        expected_revision,
    }];
    for (partner_track, partner) in partners(place.sequence, place.item) {
        moves.push(MoveItem {
            item: partner.id,
            track: partner_track,
            start: partner
                .timeline_range
                .start()
                .checked_add(delta)
                .ok_or_else(out_of_range)?,
            expected_revision: None,
        });
    }
    for operation in moves {
        planner.plan.push(operation);
    }
    Ok(())
}

/// Moves `edge` of `item` to `to`, and the same edge of linked partners sitting at the same
/// instant.
pub(super) fn plan_trim(
    planner: &mut AgentPlanner<'_>,
    item: ItemRef,
    edge: ItemEdge,
    to: Flicks,
    expected_revision: Option<u64>,
) -> Result<(), AppError> {
    let id = planner.resolve(item)?;
    let project = planner.plan.project()?;
    let place = find_item(project, id)?;
    let edge_at = |item: &Item| match edge {
        ItemEdge::Start => item.timeline_range.start(),
        ItemEdge::End => item.timeline_range.end(),
    };
    let at = edge_at(place.item);
    let mut trims = vec![TrimItem {
        item: id,
        edge,
        to,
        expected_revision,
    }];
    trims.extend(
        partners(place.sequence, place.item)
            .into_iter()
            .filter(|(_, partner)| edge_at(partner) == at)
            .map(|(_, partner)| TrimItem {
                item: partner.id,
                edge,
                to,
                expected_revision: None,
            }),
    );
    for operation in trims {
        planner.plan.push(operation);
    }
    Ok(())
}

/// Whether `item` strictly contains instant `at` (a cut there leaves two parts).
fn crosses(item: &Item, at: Flicks) -> bool {
    item.timeline_range.start() < at && item.timeline_range.end() > at
}

/// Cuts items at `at` (every item crossing it on the task's unlocked tracks when `items` is
/// `None`), with their linked partners crossing it; right parts of linked items stay linked.
/// Returns the right parts, in order.
pub(super) fn plan_split(
    planner: &mut AgentPlanner<'_>,
    items: Option<&[ItemRef]>,
    at: Flicks,
) -> Result<Vec<ItemId>, AppError> {
    let listed = match items {
        Some(refs) => refs
            .iter()
            .map(|item| planner.resolve(*item))
            .collect::<Result<Vec<_>, _>>()?,
        None => Vec::new(),
    };
    let agent = planner.agent;
    let project = planner.plan.project()?;
    let sequence = find_sequence(project, agent.sequence_id())?;
    let seeds: Vec<ItemId> = if items.is_some() {
        listed
    } else {
        sequence
            .tracks
            .iter()
            .filter(|track| !track.locked && track_allowed(&agent.scope, track.id))
            .flat_map(|track| &track.items)
            .filter(|item| crosses(item, at))
            .map(|item| item.id)
            .collect()
    };
    let mut targets: Vec<(ItemId, Option<LinkGroupId>)> = Vec::new();
    for id in seeds {
        let place = find_item(project, id)?;
        let candidates = std::iter::once(place.item).chain(
            partners(place.sequence, place.item)
                .into_iter()
                .map(|(_, partner)| partner)
                .filter(|partner| crosses(partner, at)),
        );
        for candidate in candidates {
            if !targets.iter().any(|(known, _)| *known == candidate.id) {
                targets.push((candidate.id, candidate.link_group));
            }
        }
    }
    if targets.is_empty() {
        return Err(AppError::InvalidEdit {
            reason: format!(
                "no item of this task crosses {}: list items with get_timeline and cut inside one",
                format_clock(at)
            ),
        });
    }
    let mut right_parts = Vec::with_capacity(targets.len());
    let mut groups: BTreeMap<LinkGroupId, Vec<ItemId>> = BTreeMap::new();
    for (item, link_group) in targets {
        let new_item_id = planner.plan.ids().item()?;
        planner.plan.push(SplitItem {
            item,
            at,
            new_item_id,
            expected_revision: None,
        });
        right_parts.push(new_item_id);
        if let Some(group) = link_group {
            groups.entry(group).or_default().push(new_item_id);
        }
    }
    for items in groups.into_values().filter(|items| items.len() > 1) {
        let group = planner.plan.ids().link_group()?;
        planner.plan.push(LinkItems { items, group });
    }
    Ok(right_parts)
}

/// Removes items and their linked partners, leaving gaps.
pub(super) fn plan_delete(
    planner: &mut AgentPlanner<'_>,
    items: &[(ItemRef, Option<u64>)],
) -> Result<(), AppError> {
    let listed = items
        .iter()
        .map(|(item, revision)| Ok((planner.resolve(*item)?, *revision)))
        .collect::<Result<Vec<_>, AppError>>()?;
    distinct(listed.iter().map(|(id, _)| *id))?;
    let project = planner.plan.project()?;
    let mut removals: Vec<RemoveItem> = listed
        .iter()
        .map(|&(item, expected_revision)| RemoveItem {
            item,
            expected_revision,
        })
        .collect();
    for &(id, _) in &listed {
        let place = find_item(project, id)?;
        for (_, partner) in partners(place.sequence, place.item) {
            if !removals.iter().any(|removal| removal.item == partner.id) {
                removals.push(RemoveItem {
                    item: partner.id,
                    expected_revision: None,
                });
            }
        }
    }
    for removal in removals {
        planner.plan.push(removal);
    }
    Ok(())
}

/// The tracks a ripple edit at `from` works on: the given ones, else the task's unlocked tracks
/// (`None`: every unlocked track), extended to the tracks of linked partners so picture and
/// sound stay in sync.
fn ripple_tracks(
    planner: &mut AgentPlanner<'_>,
    tracks: Option<&[TrackId]>,
    from: Flicks,
) -> Result<Option<BTreeSet<TrackId>>, AppError> {
    let agent = planner.agent;
    let sequence = find_sequence(planner.plan.project()?, agent.sequence_id())?;
    let seeds: Option<BTreeSet<TrackId>> = match (tracks, &agent.scope.tracks) {
        (Some(listed), _) => Some(listed.iter().copied().collect()),
        (None, Some(scope_tracks)) => Some(
            sequence
                .tracks
                .iter()
                .filter(|track| !track.locked && scope_tracks.contains(&track.id))
                .map(|track| track.id)
                .collect(),
        ),
        (None, None) => None,
    };
    Ok(seeds.map(|seeds| concerned_tracks(sequence, &seeds, from)))
}

/// Ids for the items of `tracks` (every unlocked track when `None`) that `needs_cut` selects.
fn split_ids(
    planner: &mut AgentPlanner<'_>,
    tracks: Option<&BTreeSet<TrackId>>,
    needs_cut: impl Fn(&Item) -> bool,
) -> Result<BTreeMap<ItemId, ItemId>, AppError> {
    let agent = planner.agent;
    let sequence = find_sequence(planner.plan.project()?, agent.sequence_id())?;
    let cut: Vec<ItemId> = sequence
        .tracks
        .iter()
        .filter(|track| tracks.map_or(!track.locked, |tracks| tracks.contains(&track.id)))
        .flat_map(|track| &track.items)
        .filter(|item| needs_cut(item))
        .map(|item| item.id)
        .collect();
    let mut ids = BTreeMap::new();
    for item in cut {
        ids.insert(item, planner.plan.ids().item()?);
    }
    Ok(ids)
}

/// Removes `range` from the tracks and closes it.
pub(super) fn plan_ripple_delete(
    planner: &mut AgentPlanner<'_>,
    range: TimeRange,
    tracks: Option<&[TrackId]>,
) -> Result<(), AppError> {
    if range.is_empty() {
        return Err(AppError::InvalidEdit {
            reason: "the range to delete is empty: give an end after its start".to_owned(),
        });
    }
    let tracks = ripple_tracks(planner, tracks, range.start())?;
    let split_ids = split_ids(planner, tracks.as_ref(), |item| {
        item.timeline_range.start() < range.start() && item.timeline_range.end() > range.end()
    })?;
    planner.plan.push(RippleDeleteRange {
        sequence: planner.agent.sequence_id(),
        range,
        tracks: tracks.map(|tracks| tracks.into_iter().collect()),
        split_ids,
    });
    Ok(())
}

/// Opens `duration` of empty time at `at` on the tracks.
pub(super) fn plan_gap(
    planner: &mut AgentPlanner<'_>,
    at: Flicks,
    duration: Flicks,
    tracks: Option<&[TrackId]>,
) -> Result<(), AppError> {
    if duration <= Flicks::ZERO {
        return Err(AppError::InvalidEdit {
            reason: "a gap needs a positive duration".to_owned(),
        });
    }
    let tracks = ripple_tracks(planner, tracks, at)?;
    let split_ids = split_ids(planner, tracks.as_ref(), |item| crosses(item, at))?;
    planner.plan.push(InsertGap {
        sequence: planner.agent.sequence_id(),
        at,
        duration,
        tracks: tracks.map(|tracks| tracks.into_iter().collect()),
        split_ids,
    });
    Ok(())
}
