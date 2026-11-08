//! Planning a resolved cut on an [`EditPlan`]: the tracks it edits, one `RippleDeleteRanges` and
//! the links of the new parts, so linked picture and sound stay in sync (ADR-0032).

use std::collections::{BTreeMap, BTreeSet};

use se_domain::{
    Flicks, ItemId, LinkGroupId, LinkItems, RippleDeleteRanges, Sequence, SequenceId, TimeRange,
    TrackId, UnlinkItems, remaining_parts,
};

use crate::{
    AppError,
    edit_plan::{EditPlan, find_sequence},
};

/// The tracks a cut starting at `from` edits: every unlocked track with `all_tracks`; otherwise
/// `seeds` (the speech's tracks), then — until nothing is added — every unlocked track holding
/// an item linked to an item of a listed track that ends after `from` (a clip crossing the cut
/// drags its linked partner along). Locked tracks never change.
#[must_use]
pub fn cut_tracks(
    sequence: &Sequence,
    seeds: &BTreeSet<TrackId>,
    from: Flicks,
    all_tracks: bool,
) -> BTreeSet<TrackId> {
    let unlocked = sequence.tracks.iter().filter(|track| !track.locked);
    if all_tracks {
        return unlocked.map(|track| track.id).collect();
    }
    let mut tracks: BTreeSet<TrackId> = sequence
        .tracks
        .iter()
        .filter(|track| !track.locked && seeds.contains(&track.id))
        .map(|track| track.id)
        .collect();
    loop {
        let groups: BTreeSet<LinkGroupId> = sequence
            .tracks
            .iter()
            .filter(|track| tracks.contains(&track.id))
            .flat_map(|track| &track.items)
            .filter(|item| item.timeline_range.end() > from)
            .filter_map(|item| item.link_group)
            .collect();
        let added: Vec<TrackId> = sequence
            .tracks
            .iter()
            .filter(|track| !track.locked && !tracks.contains(&track.id))
            .filter(|track| {
                track
                    .items
                    .iter()
                    .any(|item| item.link_group.is_some_and(|group| groups.contains(&group)))
            })
            .map(|track| track.id)
            .collect();
        if added.is_empty() {
            return tracks;
        }
        tracks.extend(added);
    }
}

/// One part of an item after the cut: its id (the item's own for the first part) and how many
/// cuts lie before it (parts of linked items between the same two cuts belong together).
struct Part {
    id: ItemId,
    interval: usize,
    is_new: bool,
    has_group: bool,
}

/// Queues the removal of `cuts` (sorted, apart) from `tracks` of `sequence_id`: one
/// `RippleDeleteRanges` with ids for every new part, then the links of the parts of linked items
/// (the parts between the same two cuts are linked together; the earliest ones keep the group).
///
/// # Errors
/// Unknown sequence, id generation; the domain's refusals surface at the next read.
pub(crate) fn plan_cut(
    plan: &mut EditPlan<'_>,
    sequence_id: SequenceId,
    cuts: &[TimeRange],
    tracks: &BTreeSet<TrackId>,
) -> Result<(), AppError> {
    if cuts.is_empty() || tracks.is_empty() {
        return Ok(());
    }
    let sequence = find_sequence(plan.project()?, sequence_id)?.clone();
    let mut split_ids: BTreeMap<ItemId, Vec<ItemId>> = BTreeMap::new();
    let mut groups: BTreeMap<LinkGroupId, Vec<Part>> = BTreeMap::new();
    for track in &sequence.tracks {
        let edited = tracks.contains(&track.id);
        for item in &track.items {
            let parts = if edited {
                remaining_parts(cuts, item.timeline_range)
            } else {
                vec![item.timeline_range]
            };
            let mut new_ids = Vec::new();
            for _ in 1..parts.len() {
                new_ids.push(plan.ids().item()?);
            }
            if let Some(group) = item.link_group {
                let ids = std::iter::once(item.id).chain(new_ids.iter().copied());
                for (index, (part, id)) in parts.iter().zip(ids).enumerate() {
                    groups.entry(group).or_default().push(Part {
                        id,
                        interval: cuts.partition_point(|cut| cut.end() <= part.start()),
                        is_new: index > 0,
                        has_group: index == 0,
                    });
                }
            }
            if !new_ids.is_empty() {
                split_ids.insert(item.id, new_ids);
            }
        }
    }
    plan.push(RippleDeleteRanges {
        sequence: sequence_id,
        ranges: cuts.to_vec(),
        tracks: Some(tracks.iter().copied().collect()),
        split_ids,
    });
    for (group, parts) in groups {
        relink(plan, group, parts)?;
    }
    Ok(())
}

/// Links the parts of the items of `group` between the same two cuts together. Groups without a
/// new part are left alone (items linked across a cut on purpose stay linked).
fn relink(plan: &mut EditPlan<'_>, group: LinkGroupId, parts: Vec<Part>) -> Result<(), AppError> {
    if !parts.iter().any(|part| part.is_new) {
        return Ok(());
    }
    let Some(first_interval) = parts.iter().map(|part| part.interval).min() else {
        return Ok(());
    };
    let mut by_interval: BTreeMap<usize, Vec<Part>> = BTreeMap::new();
    for part in parts {
        by_interval.entry(part.interval).or_default().push(part);
    }
    for (interval, members) in by_interval {
        if interval == first_interval {
            let joining: Vec<ItemId> = members
                .iter()
                .filter(|part| part.is_new)
                .map(|part| part.id)
                .collect();
            if !joining.is_empty() {
                plan.push(LinkItems {
                    items: joining,
                    group,
                });
            }
        } else if members.len() > 1 {
            plan.push(LinkItems {
                items: members.iter().map(|part| part.id).collect(),
                group: plan.ids().link_group()?,
            });
        } else if let Some(alone) = members.iter().find(|part| part.has_group) {
            plan.push(UnlinkItems {
                items: vec![alone.id],
            });
        }
    }
    Ok(())
}
