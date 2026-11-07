//! Keeping transitions in step with their items.
//!
//! Rule (`docs/architecture.md` §5.4): an item edit that leaves a transition without its two
//! adjacent items, or without enough media, removes it; the inverse puts it back. Edits made of
//! several steps (split, ripple delete, insert gap; the edit plans of se-app) then put back the
//! transitions whose items are adjacent again at the end, shortened to fit when needed, so that
//! moving both items of a transition, or cutting the outgoing item, keeps it.
//!
//! Removal happens in the primitive item moves (`placement`), so every step records its own
//! exact inverse and the inverses can be replayed one by one.

use super::{AddTransition, Edit, Effect, lookup::TrackLocation};
use crate::{
    AssetId, DomainError, Flicks, ItemId, Project, TrackId, Transition, fitted_transition,
    transition_problem,
};

/// Duration of asset `id` in `project`, when known.
pub(crate) fn asset_duration(project: &Project, id: AssetId) -> Option<Flicks> {
    project
        .assets
        .iter()
        .find(|asset| asset.id == id)
        .and_then(|asset| asset.media_info.as_ref())
        .and_then(|info| info.duration)
}

/// Removes the transitions of the tracks at `locations` that join `item` and no longer fit,
/// in list order; records an [`AddTransition`] at the same position for each.
pub(crate) fn drop_broken_transitions(
    project: &mut Project,
    locations: &[TrackLocation],
    item: ItemId,
    effect: &mut Effect,
) {
    for (position, location) in locations.iter().enumerate() {
        if locations[..position].contains(location) {
            continue;
        }
        while let Some(index) = first_broken(project, *location, item) {
            let track = location.get_mut(project);
            let transition = track.transitions.remove(index);
            effect.touched.tracks.insert(track.id);
            effect.undo_with(AddTransition {
                track: track.id,
                index: Some(index),
                transition,
            });
        }
    }
}

/// Index of the first transition of the track at `location` joining `item` that does not fit.
fn first_broken(project: &Project, location: TrackLocation, item: ItemId) -> Option<usize> {
    let track = location.get(project);
    let durations = |asset| asset_duration(project, asset);
    track.transitions.iter().position(|transition| {
        (transition.left_item == item || transition.right_item == item)
            && transition_problem(track, transition, &track.transitions, &durations).is_some()
    })
}

/// The transitions of a project with their tracks and positions, recorded before an edit so
/// that those it removed can be put back when they fit again ([`Self::restorations`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransitionSnapshot(Vec<(TrackId, usize, Transition)>);

impl TransitionSnapshot {
    /// Every transition of `project`.
    #[must_use]
    pub fn of(project: &Project) -> Self {
        Self(
            project
                .sequences
                .iter()
                .flat_map(|sequence| &sequence.tracks)
                .flat_map(|track| {
                    track
                        .transitions
                        .iter()
                        .enumerate()
                        .map(|(index, transition)| (track.id, index, transition.clone()))
                })
                .collect(),
        )
    }

    /// The transitions recorded here, gone from `project`, whose items are adjacent again on
    /// their track: each as an [`AddTransition`] at (at most) its old position, shortened to
    /// what the cut allows. `renamed` maps a cut item to the new item holding its end (the right
    /// part of a split), so a transition after the cut follows that part.
    #[must_use]
    pub fn restorations(
        &self,
        project: &Project,
        renamed: &[(ItemId, ItemId)],
    ) -> Vec<AddTransition> {
        let durations = |asset| asset_duration(project, asset);
        let mut additions = Vec::new();
        for (track_id, index, transition) in &self.0 {
            let Some(track) = project
                .sequences
                .iter()
                .flat_map(|sequence| &sequence.tracks)
                .find(|track| track.id == *track_id)
            else {
                continue;
            };
            if track.locked
                || track
                    .transitions
                    .iter()
                    .any(|other| other.id == transition.id)
            {
                continue;
            }
            let mut candidate = transition.clone();
            if let Some((_, right_part)) =
                renamed.iter().find(|(cut, _)| *cut == candidate.left_item)
            {
                candidate.left_item = *right_part;
            }
            if let Some(fitted) = fitted_transition(track, &candidate, &durations) {
                additions.push(AddTransition {
                    track: *track_id,
                    index: Some((*index).min(track.transitions.len())),
                    transition: fitted,
                });
            }
        }
        additions
    }
}

/// Ends a composite edit: puts back, one at a time (each may change the room left for the
/// next), the transitions of `before` that its steps removed and that fit again. Their inverses
/// go first, so undoing removes them before undoing the steps.
pub(crate) fn restore_transitions(
    project: &mut Project,
    before: &TransitionSnapshot,
    renamed: &[(ItemId, ItemId)],
    effect: &mut Effect,
) -> Result<(), DomainError> {
    let mut inverses = Vec::new();
    for _ in 0..before.0.len() {
        let Some(addition) = before.restorations(project, renamed).into_iter().next() else {
            break;
        };
        let mut step = Effect::default();
        addition.apply_to(project, &mut step)?;
        effect.touched.merge(step.touched);
        inverses.push(step.inverse);
    }
    effect
        .inverse
        .splice(0..0, inverses.into_iter().rev().flatten());
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::{
        ItemEdge, MoveItem, RemoveItem, RippleDeleteRange, SplitItem, TrimItem,
        operation::testing::{SEQUENCE, V1, V2, apply_ok, example},
        test_support::{millis, range_ms},
    };

    /// The transitions of V1 (the example has a centred 1 s dissolve between clips 1 and 2).
    fn dissolves(project: &Project) -> &[Transition] {
        &project.sequences[0].tracks[0].transitions
    }

    fn item(index: u64) -> ItemId {
        ItemId::from_test_index(index)
    }

    #[test]
    fn moving_the_incoming_clip_away_removes_the_dissolve() {
        let move_away = MoveItem {
            item: item(2),
            track: V2,
            start: millis(20_000),
            expected_revision: None,
        };
        let applied = apply_ok(&example(), move_away);
        assert_eq!(dissolves(&applied.project), []);
        assert!(applied.touched.tracks.contains(&V1));
    }

    #[test]
    fn removing_or_trimming_an_item_off_the_cut_removes_the_dissolve() {
        let removal = RemoveItem {
            item: item(1),
            expected_revision: None,
        };
        assert_eq!(dissolves(&apply_ok(&example(), removal).project), []);
        let trim = TrimItem {
            item: item(1),
            edge: ItemEdge::End,
            to: millis(9000),
            expected_revision: None,
        };
        assert_eq!(dissolves(&apply_ok(&example(), trim).project), []);
    }

    #[test]
    fn cutting_the_outgoing_clip_hands_the_dissolve_to_its_right_part() {
        let split = SplitItem {
            item: item(1),
            at: millis(4000),
            new_item_id: item(100),
            expected_revision: None,
        };
        let applied = apply_ok(&example(), split);
        let [dissolve] = dissolves(&applied.project) else {
            panic!("the dissolve should stay");
        };
        assert_eq!(dissolve.left_item, item(100));
        assert_eq!(dissolve.right_item, item(2));
    }

    #[test]
    fn cutting_the_incoming_clip_close_to_the_cut_shortens_the_dissolve() {
        // Its left part lasts 0.3 s: a centred dissolve fits in 0.6 s.
        let split = SplitItem {
            item: item(2),
            at: millis(10_300),
            new_item_id: item(100),
            expected_revision: None,
        };
        let applied = apply_ok(&example(), split);
        let [dissolve] = dissolves(&applied.project) else {
            panic!("the dissolve should stay");
        };
        assert_eq!(dissolve.duration, millis(600));
        assert_eq!(dissolve.right_item, item(2));
    }

    #[test]
    fn a_ripple_delete_keeps_the_dissolve_of_the_items_it_shifts() {
        let ripple = RippleDeleteRange {
            sequence: SEQUENCE,
            range: range_ms(2000, 1000),
            tracks: Some(vec![V1]),
            split_ids: BTreeMap::from([(item(1), item(100))]),
        };
        let applied = apply_ok(&example(), ripple);
        let [dissolve] = dissolves(&applied.project) else {
            panic!("the dissolve should stay");
        };
        assert_eq!(dissolve.left_item, item(100));
        assert_eq!(dissolve.duration, millis(1000));
    }

    #[test]
    fn restorations_list_transitions_whose_items_touch_again() {
        let before = TransitionSnapshot::of(&example());
        let mut project = example();
        project.sequences[0].tracks[0].transitions.clear();
        let additions = before.restorations(&project, &[]);
        let [addition] = additions.as_slice() else {
            panic!("one restoration expected, got {additions:?}");
        };
        assert_eq!(addition.track, V1);
        assert_eq!(addition.transition, before.0[0].2);
        project.sequences[0].tracks[0].items[1].timeline_range = range_ms(11_000, 3000);
        assert_eq!(before.restorations(&project, &[]).len(), 0);
    }
}
