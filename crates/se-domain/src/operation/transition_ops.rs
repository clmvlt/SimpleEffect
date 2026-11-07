//! Transition operations: [`AddTransition`], [`UpdateTransition`], [`RemoveTransition`].
//!
//! They only check what they need to find their target (ids, locked track, index): whether the
//! transition fits its cut is a document rule ([`crate::validate_project`], codes
//! `INVALID_TRANSITION` and `INSUFFICIENT_HANDLES`), checked once at the end of the
//! transaction, so a transaction may add a transition before the item edits that make it fit.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    Edit, Effect,
    checks::{ensure_unlocked, ensure_unused, insertion_index, swap_field},
    lookup::{TrackLocation, find_track},
};
use crate::{DomainError, Flicks, Project, TrackId, Transition, TransitionAlignment, TransitionId};

/// Adds a transition to a track. Inverse: [`RemoveTransition`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AddTransition {
    /// The track of the two items joined.
    pub track: TrackId,
    /// Position in the track's transitions (`null` appends).
    pub index: Option<usize>,
    /// The transition, with an unused id.
    pub transition: Transition,
}

impl Edit for AddTransition {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let location = find_track(project, self.track)?;
        ensure_unlocked(location.get(project))?;
        let id = self.transition.id;
        ensure_unused(find_transition(project, id).is_ok(), "transition", &id)?;
        let transitions = &mut location.get_mut(project).transitions;
        let index = insertion_index("transition", self.index, transitions.len())?;
        transitions.insert(index, self.transition.clone());
        effect.touched.tracks.insert(self.track);
        effect.undo_with(RemoveTransition { transition: id });
        Ok(())
    }
}

/// Removes a transition. Inverse: [`AddTransition`] at the same position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RemoveTransition {
    /// The transition removed.
    pub transition: TransitionId,
}

impl Edit for RemoveTransition {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let (location, index) = find_transition(project, self.transition)?;
        let track = location.get(project);
        ensure_unlocked(track)?;
        let track_id = track.id;
        let transition = location.get_mut(project).transitions.remove(index);
        effect.touched.tracks.insert(track_id);
        effect.undo_with(AddTransition {
            track: track_id,
            index: Some(index),
            transition,
        });
        Ok(())
    }
}

/// Changes of a [`Transition`]: its length and alignment. Its kind follows its track and its
/// items are fixed (remove it and add another to join other items).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(optional_fields)]
pub struct TransitionPatch {
    /// Another duration (> 0).
    pub duration: Option<Flicks>,
    /// Another alignment on the cut.
    pub alignment: Option<TransitionAlignment>,
}

impl TransitionPatch {
    /// Applies the patch and returns the one undoing it.
    fn apply(&self, transition: &mut Transition) -> Self {
        Self {
            duration: swap_field(&mut transition.duration, self.duration.as_ref()),
            alignment: swap_field(&mut transition.alignment, self.alignment.as_ref()),
        }
    }
}

/// Resizes or realigns a transition. Inverse: [`UpdateTransition`] with the previous values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTransition {
    /// The transition changed.
    pub transition: TransitionId,
    /// What changes.
    pub patch: TransitionPatch,
}

impl Edit for UpdateTransition {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let (location, index) = find_transition(project, self.transition)?;
        let track = location.get(project);
        ensure_unlocked(track)?;
        let track_id = track.id;
        let previous = self
            .patch
            .apply(&mut location.get_mut(project).transitions[index]);
        effect.touched.tracks.insert(track_id);
        effect.undo_with(Self {
            transition: self.transition,
            patch: previous,
        });
        Ok(())
    }
}

/// The track holding transition `id` and its index there.
pub(crate) fn find_transition(
    project: &Project,
    id: TransitionId,
) -> Result<(TrackLocation, usize), DomainError> {
    for (sequence_index, sequence) in project.sequences.iter().enumerate() {
        for (track_index, track) in sequence.tracks.iter().enumerate() {
            if let Some(index) = track.transitions.iter().position(|other| other.id == id) {
                let location = TrackLocation {
                    sequence: sequence_index,
                    track: track_index,
                };
                return Ok((location, index));
            }
        }
    }
    Err(DomainError::TransitionNotFound { transition: id })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DomainErrorCode, ItemId, TransitionKind, ViolationCode,
        operation::testing::{V1, apply_err, apply_ok, example, lock},
        test_support::millis,
    };

    const DISSOLVE: u64 = 1;

    fn transitions(project: &Project) -> &[Transition] {
        &project.sequences[0].tracks[0].transitions
    }

    fn remove() -> RemoveTransition {
        RemoveTransition {
            transition: TransitionId::from_test_index(DISSOLVE),
        }
    }

    #[test]
    fn removes_and_adds_back_at_the_same_place() {
        let removed = apply_ok(&example(), remove());
        assert_eq!(transitions(&removed.project), []);
        assert!(removed.touched.tracks.contains(&V1));
        let added = apply_ok(&removed.project, removed.inverse[0].clone());
        assert_eq!(added.project, example());
    }

    #[test]
    fn updates_the_length_and_alignment() {
        let patch = TransitionPatch {
            duration: Some(millis(2000)),
            alignment: Some(TransitionAlignment::Start),
        };
        let update = UpdateTransition {
            transition: TransitionId::from_test_index(DISSOLVE),
            patch,
        };
        let applied = apply_ok(&example(), update);
        assert_eq!(transitions(&applied.project)[0].duration, millis(2000));
        assert_eq!(
            transitions(&applied.project)[0].alignment,
            TransitionAlignment::Start
        );
    }

    #[test]
    fn the_document_rules_decide_whether_it_fits() {
        let mut transition = transitions(&example())[0].clone();
        transition.id = TransitionId::from_test_index(2);
        transition.left_item = ItemId::from_test_index(2);
        transition.right_item = ItemId::from_test_index(1);
        transition.kind = TransitionKind::CrossDissolve;
        let add = AddTransition {
            track: V1,
            index: None,
            transition,
        };
        let DomainError::InvalidDocument { violations } = apply_err(&example(), add) else {
            panic!("expected a validation error");
        };
        assert_eq!(violations[0].code, ViolationCode::InvalidTransition);
    }

    #[test]
    fn refuses_unknown_ids_reused_ids_and_locked_tracks() {
        let unknown = RemoveTransition {
            transition: TransitionId::from_test_index(99),
        };
        assert_eq!(
            apply_err(&example(), unknown).code(),
            DomainErrorCode::TransitionNotFound
        );
        let add = AddTransition {
            track: V1,
            index: None,
            transition: transitions(&example())[0].clone(),
        };
        assert_eq!(
            apply_err(&example(), add).code(),
            DomainErrorCode::IdAlreadyUsed
        );
        assert_eq!(
            apply_err(&lock(example(), V1), remove()).code(),
            DomainErrorCode::TrackLocked
        );
    }
}
