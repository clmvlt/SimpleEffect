//! Lifting items off their tracks and placing items with a [`PlacementMode`].

use std::collections::BTreeSet;

use se_domain::{AddItem, Item, ItemId, RemoveItem, SequenceId, TimeRange, TrackId};
use se_protocol::PlacementMode;

use super::{
    EditPlan, concerned_tracks, find_item, find_sequence, open_time, overwrite::clear_range,
};
use crate::AppError;

/// Removes `item_ids` from their tracks and returns them (in the same order) with the track
/// each was on, to be placed again.
///
/// # Errors
/// Unknown items, items on locked tracks.
pub(crate) fn lift(
    plan: &mut EditPlan<'_>,
    item_ids: &[ItemId],
) -> Result<Vec<(TrackId, Item)>, AppError> {
    let project = plan.project()?;
    let lifted = item_ids
        .iter()
        .map(|&id| find_item(project, id).map(|place| (place.track.id, place.item.clone())))
        .collect::<Result<Vec<_>, _>>()?;
    for &item in item_ids {
        plan.push(RemoveItem {
            item,
            expected_revision: None,
        });
    }
    Ok(lifted)
}

/// Puts `placed` items (lifted, copied or new, each with its target track of sequence
/// `sequence_id`) on their tracks. [`PlacementMode::Overwrite`] first empties the range each
/// covers on its track; [`PlacementMode::Insert`] opens time as long as the placed block at its
/// start, on the target tracks and their concerned tracks.
///
/// # Errors
/// Placed items overlapping each other, locked tracks, items a track does not accept.
pub(crate) fn place(
    plan: &mut EditPlan<'_>,
    sequence_id: SequenceId,
    placed: Vec<(TrackId, Item)>,
    mode: PlacementMode,
) -> Result<(), AppError> {
    let Some(block) = block_of(&placed) else {
        return Ok(());
    };
    match mode {
        PlacementMode::Overwrite => {
            for (track, item) in &placed {
                clear_range(plan, *track, item.timeline_range)?;
            }
        }
        PlacementMode::Insert => {
            let seeds: BTreeSet<TrackId> = placed.iter().map(|(track, _)| *track).collect();
            let sequence = find_sequence(plan.project()?, sequence_id)?;
            let tracks = concerned_tracks(sequence, &seeds, block.start());
            open_time(plan, sequence_id, block.start(), block.duration(), &tracks)?;
        }
    }
    for (track, item) in placed {
        plan.push(AddItem { track, item });
    }
    Ok(())
}

/// From the earliest start to the latest end of `placed`, `None` when empty.
fn block_of(placed: &[(TrackId, Item)]) -> Option<TimeRange> {
    let start = placed
        .iter()
        .map(|(_, item)| item.timeline_range.start())
        .min()?;
    let end = placed
        .iter()
        .map(|(_, item)| item.timeline_range.end())
        .max()?;
    TimeRange::from_start_end(start, end).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        edit_plan::{
            starting_at,
            testing::{A1, V1, V2, clip, item_id, project_with, secs, spans, track_id},
        },
        new_ids::NewIds,
        test_support::SequentialIds,
    };

    #[test]
    fn a_lifted_item_overwrites_where_it_lands() {
        let project = project_with(
            vec![clip(1, 0, 2, None), clip(2, 3, 9, None)],
            Vec::new(),
            Vec::new(),
        );
        let ids = SequentialIds::default();
        let mut plan = EditPlan::new(&project, NewIds(&ids));
        let sequence = project.sequences[0].id;
        let lifted = lift(&mut plan, &[item_id(1)]).unwrap();
        let moved = lifted
            .into_iter()
            .map(|(track, item)| (track, starting_at(item, secs(4)).unwrap()))
            .collect();
        place(&mut plan, sequence, moved, PlacementMode::Overwrite).unwrap();
        assert_eq!(spans(plan.project().unwrap(), V1), [(3, 4), (4, 6), (6, 9)]);
    }

    #[test]
    fn insert_pushes_later_items_on_target_and_linked_tracks() {
        let project = project_with(
            vec![clip(1, 0, 2, Some(1)), clip(2, 2, 4, Some(2))],
            vec![clip(3, 0, 1, None)],
            vec![clip(4, 0, 2, Some(1)), clip(5, 2, 4, Some(2))],
        );
        let ids = SequentialIds::default();
        let mut plan = EditPlan::new(&project, NewIds(&ids));
        let sequence = project.sequences[0].id;
        let new_clip = clip(9, 2, 5, None);
        place(
            &mut plan,
            sequence,
            vec![(track_id(V1), new_clip)],
            PlacementMode::Insert,
        )
        .unwrap();
        let project = plan.project().unwrap();
        assert_eq!(spans(project, V1), [(0, 2), (2, 5), (5, 7)]);
        assert_eq!(spans(project, A1), [(0, 2), (5, 7)]);
        assert_eq!(spans(project, V2), [(0, 1)]);
    }
}
