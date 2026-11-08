//! Cross-fades added by agents: on the cut between two adjacent items of one track, shortened to
//! what the cut allows (as for the user, `se_domain::transition_room`).

use se_domain::{
    AddTransition, Flicks, Transition, TransitionAlignment, TransitionKind, transition_room,
};

use super::{ItemRef, planner::AgentPlanner};
use crate::{AppError, edit_plan::find_item, project_service::asset_duration};

/// Queues the transition between `left` and `right`.
///
/// # Errors
/// Items on different tracks, [`AppError::TransitionNoRoom`] when the cut has no media to
/// cross-fade (or the items do not touch).
pub(super) fn plan_transition(
    planner: &mut AgentPlanner<'_>,
    left: ItemRef,
    right: ItemRef,
    duration: Flicks,
    alignment: TransitionAlignment,
) -> Result<(), AppError> {
    if duration <= Flicks::ZERO {
        return Err(AppError::InvalidEdit {
            reason: "a transition needs a positive duration".to_owned(),
        });
    }
    let (left, right) = (planner.resolve(left)?, planner.resolve(right)?);
    let id = planner.plan.ids().transition()?;
    let project = planner.plan.project()?;
    let left_place = find_item(project, left)?;
    let right_place = find_item(project, right)?;
    if left_place.track.id != right_place.track.id {
        return Err(AppError::InvalidEdit {
            reason: format!(
                "items {left} and {right} are on different tracks ({} and {}): a transition \
                 joins two adjacent items of one track",
                left_place.track.name, right_place.track.name
            ),
        });
    }
    let track = left_place.track;
    let room = transition_room(track, (left, right), alignment, None, &|asset| {
        asset_duration(project, asset)
    });
    if room <= Flicks::ZERO {
        return Err(AppError::TransitionNoRoom);
    }
    let addition = AddTransition {
        track: track.id,
        index: None,
        transition: Transition {
            id,
            kind: TransitionKind::for_track(track.kind),
            left_item: left,
            right_item: right,
            duration: duration.min(room),
            alignment,
        },
    };
    planner.plan.push(addition);
    Ok(())
}
