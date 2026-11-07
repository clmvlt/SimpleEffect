//! Overwrite: emptying a range of a track so an item can be placed there.

use se_domain::{Item, ItemEdge, RemoveItem, SplitItem, TimeRange, TrackId, TrimItem};

use super::{EditPlan, find_track};
use crate::AppError;

/// Empties `range` on track `track_id`: items inside it are removed, items crossing one of its
/// edges are trimmed to that edge, an item spanning it is cut around it (its right part gets a
/// new id and no link group, like any split). Items linked to them on other tracks stay as they
/// are: an overwrite only changes its own tracks.
///
/// # Errors
/// Unknown or locked track (the domain refuses the edits).
pub(crate) fn clear_range(
    plan: &mut EditPlan<'_>,
    track_id: TrackId,
    range: TimeRange,
) -> Result<(), AppError> {
    if range.is_empty() {
        return Ok(());
    }
    let (_, track) = find_track(plan.project()?, track_id)?;
    let covered: Vec<Item> = track
        .items
        .iter()
        .filter(|item| item.timeline_range.intersects(range))
        .cloned()
        .collect();
    for item in &covered {
        clear_item(plan, item, range)?;
    }
    Ok(())
}

/// Removes, trims or cuts `item` so it leaves `range`.
fn clear_item(plan: &mut EditPlan<'_>, item: &Item, range: TimeRange) -> Result<(), AppError> {
    let (start, end) = (item.timeline_range.start(), item.timeline_range.end());
    let trim = |edge, to| TrimItem {
        item: item.id,
        edge,
        to,
        expected_revision: None,
    };
    match (start < range.start(), end > range.end()) {
        (false, false) => plan.push(RemoveItem {
            item: item.id,
            expected_revision: None,
        }),
        (true, true) => {
            let new_item_id = plan.ids().item()?;
            plan.push(SplitItem {
                item: item.id,
                at: range.end(),
                new_item_id,
                expected_revision: None,
            });
            plan.push(trim(ItemEdge::End, range.start()));
        }
        (true, false) => plan.push(trim(ItemEdge::End, range.start())),
        (false, true) => plan.push(trim(ItemEdge::Start, range.end())),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use se_domain::TimeRange;

    use super::*;
    use crate::{
        edit_plan::testing::{V1, clip, project_with, secs, spans, track_id},
        new_ids::NewIds,
        test_support::SequentialIds,
    };

    #[test]
    fn removes_trims_and_cuts_what_the_range_covers() {
        let project = project_with(
            vec![
                clip(1, 0, 4, None),
                clip(2, 5, 6, None),
                clip(3, 7, 20, None),
            ],
            Vec::new(),
            Vec::new(),
        );
        let ids = SequentialIds::default();
        let mut plan = EditPlan::new(&project, NewIds(&ids));
        let range = TimeRange::from_start_end(secs(3), secs(8)).unwrap();
        clear_range(&mut plan, track_id(V1), range).unwrap();
        assert_eq!(spans(plan.project().unwrap(), V1), [(0, 3), (8, 20)]);

        let mut spanning = EditPlan::new(&project, NewIds(&ids));
        let inside = TimeRange::from_start_end(secs(10), secs(12)).unwrap();
        clear_range(&mut spanning, track_id(V1), inside).unwrap();
        assert_eq!(
            spans(spanning.project().unwrap(), V1),
            [(0, 4), (5, 6), (7, 10), (12, 20)]
        );
    }
}
