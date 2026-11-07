//! Transitions of open projects (prompt 14): cross-fades added on cuts, resized, realigned and
//! removed. Each command is one user transaction, so one undo step. Durations are shortened to
//! what the cut allows (`se_domain::transition_room`), so a gesture or an agent asking for too
//! long a cross-fade gets the longest that fits.

use se_domain::{
    AddTransition, Flicks, Project, ProjectId, RemoveTransition, Track, Transition, TransitionId,
    TransitionKind, TransitionPatch, UpdateTransition, transition_room,
};
use se_protocol::NewTransition;

use super::ProjectService;
use crate::{
    AppError,
    edit_plan::{distinct, find_track},
};

/// Labels of the transition transactions (undo menu). Stable identifiers: the renderer
/// translates them (`apps/desktop/src/app/editor/history-label.ts`).
pub mod labels {
    /// Cross-fades were added on cuts.
    pub const ADD: &str = "Add transitions";
    /// A transition was resized or realigned.
    pub const CHANGE: &str = "Change transition";
    /// Transitions were removed.
    pub const REMOVE: &str = "Remove transitions";
}

impl ProjectService {
    /// Adds a cross-fade on each listed cut, its kind from the track and its duration shortened
    /// to what the cut allows; cuts without room are skipped. Returns the ids of the transitions
    /// added, in request order.
    ///
    /// # Errors
    /// Not open, nothing listed, unknown tracks, [`AppError::TransitionNoRoom`] when no cut has
    /// room, refusals of the document (locked track…), storage errors.
    pub async fn add_transitions(
        &self,
        project_id: ProjectId,
        wanted: &[NewTransition],
    ) -> Result<Vec<TransitionId>, AppError> {
        if wanted.is_empty() {
            return Err(AppError::InvalidEdit {
                reason: "no cut was given; list the cuts to cross-fade".to_owned(),
            });
        }
        let handle = self.handle(project_id).await?;
        let mut draft = handle.view().await?.project.as_ref().clone();
        let mut ops = Vec::new();
        let mut ids = Vec::new();
        for cut in wanted {
            let Some(addition) = self.fitting_transition(&draft, cut)? else {
                continue;
            };
            ids.push(addition.transition.id);
            draft = se_domain::apply(&draft, &addition.clone().into())?.project;
            ops.push(addition.into());
        }
        if ops.is_empty() {
            return Err(AppError::TransitionNoRoom);
        }
        self.commit_user_edit(&handle, labels::ADD, ops).await?;
        Ok(ids)
    }

    /// Resizes or realigns a transition; the duration is shortened to what its cut allows with
    /// the new alignment.
    ///
    /// # Errors
    /// Not open, unknown transition, [`AppError::TransitionNoRoom`] when the new alignment
    /// leaves no room, refusals of the document, storage errors.
    pub async fn update_transition(
        &self,
        project_id: ProjectId,
        transition_id: TransitionId,
        patch: TransitionPatch,
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let (track, transition) = find_transition(&view.project, transition_id)?;
        let alignment = patch.alignment.unwrap_or(transition.alignment);
        let room = transition_room(
            track,
            (transition.left_item, transition.right_item),
            alignment,
            Some(transition_id),
            &|asset| asset_duration(&view.project, asset),
        );
        if room <= Flicks::ZERO {
            return Err(AppError::TransitionNoRoom);
        }
        let duration = patch.duration.unwrap_or(transition.duration).min(room);
        let patch = TransitionPatch {
            duration: (duration != transition.duration).then_some(duration),
            alignment: patch
                .alignment
                .filter(|wanted| *wanted != transition.alignment),
        };
        if patch == TransitionPatch::default() {
            return Ok(());
        }
        let update = UpdateTransition {
            transition: transition_id,
            patch,
        };
        self.commit_user_edit(&handle, labels::CHANGE, vec![update.into()])
            .await
    }

    /// Removes transitions.
    ///
    /// # Errors
    /// Not open, nothing listed or a transition twice, unknown transitions, locked tracks,
    /// storage errors.
    pub async fn remove_transitions(
        &self,
        project_id: ProjectId,
        transition_ids: &[TransitionId],
    ) -> Result<(), AppError> {
        distinct(transition_ids.iter().copied())?;
        let handle = self.handle(project_id).await?;
        let ops = transition_ids
            .iter()
            .map(|transition| {
                RemoveTransition {
                    transition: *transition,
                }
                .into()
            })
            .collect();
        self.commit_user_edit(&handle, labels::REMOVE, ops).await
    }

    /// The transition to add on `cut` in `project`, shortened to the room the cut leaves, or
    /// `None` when it has none.
    fn fitting_transition(
        &self,
        project: &Project,
        cut: &NewTransition,
    ) -> Result<Option<AddTransition>, AppError> {
        let (_, track) = find_track(project, cut.track_id)?;
        let room = transition_room(
            track,
            (cut.left_item_id, cut.right_item_id),
            cut.alignment,
            None,
            &|asset| asset_duration(project, asset),
        );
        if room <= Flicks::ZERO || cut.duration <= Flicks::ZERO {
            return Ok(None);
        }
        let transition = Transition {
            id: self.new_ids().transition()?,
            kind: TransitionKind::for_track(track.kind),
            left_item: cut.left_item_id,
            right_item: cut.right_item_id,
            duration: cut.duration.min(room),
            alignment: cut.alignment,
        };
        Ok(Some(AddTransition {
            track: track.id,
            index: None,
            transition,
        }))
    }
}

/// The track holding transition `id`, and the transition.
fn find_transition(project: &Project, id: TransitionId) -> Result<(&Track, &Transition), AppError> {
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .find_map(|track| {
            track
                .transitions
                .iter()
                .find(|transition| transition.id == id)
                .map(|transition| (track, transition))
        })
        .ok_or(AppError::Domain(
            se_domain::DomainError::TransitionNotFound { transition: id },
        ))
}

/// The duration of asset `id` in `project`, when known.
fn asset_duration(project: &Project, id: se_domain::AssetId) -> Option<Flicks> {
    project
        .assets
        .iter()
        .find(|asset| asset.id == id)
        .and_then(|asset| asset.media_info.as_ref())
        .and_then(|info| info.duration)
}
