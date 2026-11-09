//! New texts, shapes, images and motion template instances placed by agents: on the given video track, or on the highest
//! free track the task may use (a new track on top when the task may use every track), with
//! their animations in the same edit. Created items carry `origin = AiTask(task)`.

use se_domain::{
    AddItem, AddTrack, Item, ItemId, ItemKind, Origin, Sequence, TimeRange, TrackId, TrackKind,
    format_clock,
};
use se_protocol::TaskScope;

use super::{NewAgentItem, planner::AgentPlanner, scope::track_allowed};
use crate::{
    AppError,
    edit_plan::find_sequence,
    new_project::empty_track,
    project_service::{graphic_track, next_track_name, top_video_index},
};

/// Queues the item (and its track when a new one is needed) and its presets; returns its id.
///
/// # Errors
/// A kind other than text, shape, image or template instance, an empty range, an audio or foreign track, no free
/// track the task may use.
pub(super) fn plan_new_item(
    planner: &mut AgentPlanner<'_>,
    new: &NewAgentItem,
) -> Result<ItemId, AppError> {
    if !matches!(
        new.kind,
        ItemKind::Text(_) | ItemKind::Shape(_) | ItemKind::Image(_) | ItemKind::MotionTemplate(_)
    ) {
        return Err(AppError::InvalidEdit {
            reason: format!(
                "agents add texts, shapes, images and template instances, not a {}",
                new.kind.label()
            ),
        });
    }
    if new.range.is_empty() {
        return Err(AppError::InvalidEdit {
            reason: "a new item needs a positive duration: give an end after its start".to_owned(),
        });
    }
    let agent = planner.agent;
    let scope = &agent.scope;
    let sequence = find_sequence(planner.plan.project()?, scope.sequence_id)?.clone();
    let track_id = match new.track {
        Some(track) => checked_track(&sequence, track)?,
        None => {
            if let Some(track) = free_track(&sequence, new.range, scope)? {
                track
            } else {
                let name = next_track_name(&sequence, TrackKind::Video);
                let track = empty_track(planner.plan.ids(), &name, TrackKind::Video)?;
                let track_id = track.id;
                planner.plan.push(AddTrack {
                    sequence: sequence.id,
                    index: Some(top_video_index(&sequence)),
                    track,
                });
                track_id
            }
        }
    };
    let item = Item {
        id: planner.plan.ids().item()?,
        revision: 0,
        origin: Origin::AiTask {
            task_id: agent.task_id,
        },
        timeline_range: new.range,
        link_group: None,
        kind: new.kind.clone(),
    };
    let item_id = item.id;
    planner.plan.push(AddItem {
        track: track_id,
        item,
    });
    for preset in &new.presets {
        planner.push_preset(item_id, preset, None, None)?;
    }
    Ok(item_id)
}

/// `track`, refused unless it is a video track of the task's sequence.
fn checked_track(sequence: &Sequence, track: TrackId) -> Result<TrackId, AppError> {
    let found = sequence
        .tracks
        .iter()
        .find(|candidate| candidate.id == track)
        .ok_or_else(|| AppError::OutOfScope {
            reason: format!(
                "track {track} is not a track of the task's sequence « {} »: pick one from \
                 get_project_overview",
                sequence.name
            ),
        })?;
    if found.kind != TrackKind::Video {
        return Err(AppError::InvalidEdit {
            reason: format!(
                "track {} holds sound: texts, shapes and images go on video tracks",
                found.name
            ),
        });
    }
    Ok(found.id)
}

/// The track a new graphic over `range` goes on when none is given: as for the user (the highest
/// free track with nothing above it, `None` for a new track on top) when the task may use every
/// track; otherwise the highest unlocked, visible video track of the task free over `range`.
///
/// # Errors
/// [`AppError::OutOfScope`] when the task's tracks have no room.
fn free_track(
    sequence: &Sequence,
    range: TimeRange,
    scope: &TaskScope,
) -> Result<Option<TrackId>, AppError> {
    if scope.tracks.is_none() {
        return Ok(graphic_track(sequence, range));
    }
    sequence
        .tracks
        .iter()
        .rev()
        .filter(|track| track.kind == TrackKind::Video && track_allowed(scope, track.id))
        .filter(|track| !track.locked && !track.hidden)
        .find(|track| {
            !track
                .items
                .iter()
                .any(|item| item.timeline_range.intersects(range))
        })
        .map(|track| Some(track.id))
        .ok_or_else(|| AppError::OutOfScope {
            reason: format!(
                "no video track of this task is free between {} and {}: give a track with \
                 `track`, move or shorten what is there, or pick another time",
                format_clock(range.start()),
                format_clock(range.end())
            ),
        })
}
