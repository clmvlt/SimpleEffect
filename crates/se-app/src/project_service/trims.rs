//! Edits of item edges and of the time between items: trims (with or without ripple), splits and
//! gaps (ADR-0021). Each is one user transaction, so one undo step.

use std::{cmp::Ordering, collections::BTreeMap, collections::BTreeSet};

use se_domain::{
    Flicks, ItemEdge, ItemId, LinkGroupId, LinkItems, Project, ProjectId, SplitItem, TimeRange,
    TrackId, TrimItem,
};
use se_protocol::ItemTrim;

use super::{ProjectService, items::labels};
use crate::{
    AppError,
    edit_plan::{
        EditPlan, concerned_tracks, distinct, find_item, find_sequence, find_track, open_time,
        remove_time, sequence_of,
    },
};

impl ProjectService {
    /// Moves item edges. With `ripple`, what follows shifts by the change on the items' tracks
    /// and their concerned tracks: every listed edge must then sit at the same instant (an item
    /// and its linked partners) and move to the same place.
    ///
    /// # Errors
    /// Not open, nothing listed or an item twice, unknown items, several sequences, unaligned
    /// ripple trims, trims the document refuses (beyond the media, collapsing the item, onto a
    /// neighbour…), [`AppError::RippleBlocked`], storage errors.
    pub async fn trim_items(
        &self,
        project_id: ProjectId,
        trims: &[ItemTrim],
        ripple: bool,
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let mut plan = EditPlan::new(&view.project, self.new_ids());
        distinct(trims.iter().map(|trim| trim.item_id))?;
        if ripple {
            ripple_trim(&mut plan, trims)?;
            self.commit_plan(&handle, labels::RIPPLE_TRIM, plan).await
        } else {
            plain_trims(&mut plan, trims)?;
            self.commit_plan(&handle, labels::TRIM, plan).await
        }
    }

    /// Cuts items at `at`; the right parts of items that were linked together are linked
    /// together. Returns the right parts' ids in the order of `item_ids`.
    ///
    /// # Errors
    /// Not open, nothing listed or an item twice, unknown items, `at` outside an item, locked
    /// tracks, storage errors.
    pub async fn split_items(
        &self,
        project_id: ProjectId,
        item_ids: &[ItemId],
        at: Flicks,
    ) -> Result<Vec<ItemId>, AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let mut plan = EditPlan::new(&view.project, self.new_ids());
        distinct(item_ids.iter().copied())?;
        let project = plan.project()?;
        let link_groups = item_ids
            .iter()
            .map(|&item| find_item(project, item).map(|place| place.item.link_group))
            .collect::<Result<Vec<_>, _>>()?;
        let mut right_parts = Vec::with_capacity(item_ids.len());
        let mut groups: BTreeMap<LinkGroupId, Vec<ItemId>> = BTreeMap::new();
        for (&item, link_group) in item_ids.iter().zip(link_groups) {
            let new_item_id = plan.ids().item()?;
            plan.push(SplitItem {
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
            let group = plan.ids().link_group()?;
            plan.push(LinkItems { items, group });
        }
        self.commit_plan(&handle, labels::SPLIT, plan).await?;
        Ok(right_parts)
    }

    /// Gives the gap `[start, end)` of a track (empty, followed by an item starting at `end`) the
    /// length `duration`: time is removed or opened at its end, on the track and its concerned
    /// tracks, so what follows moves. `0` closes it.
    ///
    /// # Errors
    /// Not open, unknown or locked track, no such gap any more, a negative length,
    /// [`AppError::RippleBlocked`], storage errors.
    pub async fn resize_gap(
        &self,
        project_id: ProjectId,
        track_id: TrackId,
        gap: TimeRange,
        duration: Flicks,
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let mut plan = EditPlan::new(&view.project, self.new_ids());
        if duration.is_negative() {
            return Err(AppError::InvalidEdit {
                reason: "a gap cannot be shorter than 0: close it with a length of 0".to_owned(),
            });
        }
        let (sequence, track) = find_track(plan.project()?, track_id)?;
        check_gap(track, gap)?;
        let sequence_id = sequence.id;
        let tracks = concerned_tracks(sequence, &BTreeSet::from([track_id]), gap.end());
        let change = duration
            .checked_sub(gap.duration())
            .ok_or_else(out_of_range)?;
        match change.cmp(&Flicks::ZERO) {
            Ordering::Equal => return Ok(()),
            Ordering::Less => {
                let start = gap
                    .end()
                    .checked_sub(change.abs())
                    .ok_or_else(out_of_range)?;
                let removed =
                    TimeRange::from_start_end(start, gap.end()).map_err(|_| out_of_range())?;
                remove_time(&mut plan, sequence_id, removed, &tracks, &BTreeSet::new())?;
            }
            Ordering::Greater => open_time(&mut plan, sequence_id, gap.end(), change, &tracks)?,
        }
        let label = if duration == Flicks::ZERO {
            labels::CLOSE_GAP
        } else {
            labels::RESIZE_GAP
        };
        self.commit_plan(&handle, label, plan).await
    }
}

/// Refuses a range that is not a gap of `track` any more (someone edited it since).
fn check_gap(track: &se_domain::Track, gap: TimeRange) -> Result<(), AppError> {
    let not_a_gap = |why: &str| AppError::InvalidEdit {
        reason: format!(
            "there is no gap from {:.3} s to {:.3} s on track {} ({why}): select the gap again",
            gap.start().to_seconds_f64(),
            gap.end().to_seconds_f64(),
            track.name
        ),
    };
    if gap.is_empty() {
        return Err(not_a_gap("it is empty"));
    }
    if track
        .items
        .iter()
        .any(|item| item.timeline_range.intersects(gap))
    {
        return Err(not_a_gap("an item is there"));
    }
    if !track
        .items
        .iter()
        .any(|item| item.timeline_range.start() == gap.end())
    {
        return Err(not_a_gap("no item starts at its end"));
    }
    Ok(())
}

fn out_of_range() -> AppError {
    AppError::InvalidEdit {
        reason: "the edit would leave the supported time range".to_owned(),
    }
}

/// Plain trims, those shortening an item first so that an item may grow into the room another
/// one leaves in the same edit.
fn plain_trims(plan: &mut EditPlan<'_>, trims: &[ItemTrim]) -> Result<(), AppError> {
    let project = plan.project()?;
    let mut ordered = Vec::with_capacity(trims.len());
    for trim in trims {
        let range = find_item(project, trim.item_id)?.item.timeline_range;
        let shortens = match trim.edge {
            ItemEdge::Start => trim.to > range.start(),
            ItemEdge::End => trim.to < range.end(),
        };
        ordered.push((!shortens, *trim));
    }
    ordered.sort_by_key(|(grows, _)| *grows);
    for (_, trim) in ordered {
        plan.push(trim_operation(trim.item_id, trim.edge, trim.to));
    }
    Ok(())
}

/// A ripple trim: edges sitting at one instant `p` move to `q`, and what follows on the concerned
/// tracks shifts by the change, so the trimmed items keep their place on the timeline.
fn ripple_trim(plan: &mut EditPlan<'_>, trims: &[ItemTrim]) -> Result<(), AppError> {
    let project = plan.project()?;
    let (edge, from, to) = aligned_edges(project, trims)?;
    let items: Vec<ItemId> = trims.iter().map(|trim| trim.item_id).collect();
    let sequence_id = sequence_of(project, items.iter().copied(), [])?;
    let seeds = items
        .iter()
        .map(|&id| find_item(project, id).map(|place| place.track.id))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let tracks = concerned_tracks(find_sequence(project, sequence_id)?, &seeds, from);
    let allowed: BTreeSet<ItemId> = items.iter().copied().collect();
    let trim_all = |plan: &mut EditPlan<'_>, position: Flicks| {
        for &item in &items {
            plan.push(trim_operation(item, edge, position));
        }
    };
    let span = |a: Flicks, b: Flicks| {
        TimeRange::from_start_end(a.min(b), a.max(b)).map_err(|_| out_of_range())
    };
    match (edge, to.cmp(&from)) {
        (_, Ordering::Equal) => Ok(()),
        (ItemEdge::End, Ordering::Less) | (ItemEdge::Start, Ordering::Greater) => {
            trim_all(plan, to);
            remove_time(plan, sequence_id, span(from, to)?, &tracks, &allowed)
        }
        (ItemEdge::End, Ordering::Greater) => {
            open_time(plan, sequence_id, from, span(from, to)?.duration(), &tracks)?;
            trim_all(plan, to);
            Ok(())
        }
        (ItemEdge::Start, Ordering::Less) => {
            open_time(plan, sequence_id, from, span(from, to)?.duration(), &tracks)?;
            trim_all(plan, from);
            Ok(())
        }
    }
}

/// The common edge, its current instant and its target of ripple trims; refused unless every
/// trim moves the same edge from the same instant to the same place.
fn aligned_edges(
    project: &Project,
    trims: &[ItemTrim],
) -> Result<(ItemEdge, Flicks, Flicks), AppError> {
    let unaligned = || AppError::InvalidEdit {
        reason: "a ripple trim moves edges that sit at the same instant to the same place: trim \
                 the other items separately"
            .to_owned(),
    };
    let mut common: Option<(ItemEdge, Flicks, Flicks)> = None;
    for trim in trims {
        let range = find_item(project, trim.item_id)?.item.timeline_range;
        let at = match trim.edge {
            ItemEdge::Start => range.start(),
            ItemEdge::End => range.end(),
        };
        let this = (trim.edge, at, trim.to);
        if common.is_some_and(|first| first != this) {
            return Err(unaligned());
        }
        common = Some(this);
    }
    common.ok_or_else(unaligned)
}

const fn trim_operation(item: ItemId, edge: ItemEdge, to: Flicks) -> TrimItem {
    TrimItem {
        item,
        edge,
        to,
        expected_revision: None,
    }
}
