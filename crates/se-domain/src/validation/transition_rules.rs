//! Rules of transitions: `INVALID_TRANSITION` (placement on the cut, kind, clashes) and
//! `INSUFFICIENT_HANDLES` (media beyond the items' edges). The geometry lives in
//! `document::transition_fit`, shared with the operations.

use super::{Path, Report, ViolationCode, report::ProjectIndex, report::seconds};
use crate::{ItemEdge, Track, TransitionProblem, transition_problem};

/// Checks every transition of `track`, each against the transitions listed before it (so a
/// clash between two transitions is reported once, on the later one).
pub(super) fn check(track: &Track, index: &ProjectIndex<'_>, path: &Path, report: &mut Report) {
    let durations = |asset| index.asset_duration(asset);
    let transitions_path = path.field("transitions");
    for (position, transition) in track.transitions.iter().enumerate() {
        let earlier = &track.transitions[..position];
        let Some(problem) = transition_problem(track, transition, earlier, &durations) else {
            continue;
        };
        let code = if problem.is_handle_problem() {
            ViolationCode::InsufficientHandles
        } else {
            ViolationCode::InvalidTransition
        };
        report.push(
            code,
            &transitions_path.key(transition.id),
            message(&problem, track),
        );
    }
}

/// What is wrong and how to fix it.
fn message(problem: &TransitionProblem, track: &Track) -> String {
    match *problem {
        TransitionProblem::NotPositive => {
            "a transition lasts more than 0s; give it a positive duration".to_owned()
        }
        TransitionProblem::WrongTrackKind => format!(
            "track {} holds {}; use a cross dissolve on video tracks and an audio cross-fade on \
             audio tracks",
            track.name,
            match track.kind {
                crate::TrackKind::Video => "pictures",
                crate::TrackKind::Audio => "sound",
            }
        ),
        TransitionProblem::SameItem => {
            "a transition joins two different items; give it the items on each side of a cut"
                .to_owned()
        }
        TransitionProblem::MissingItem { item } => format!(
            "item {item} is not on track {}; a transition joins two adjacent items of its own \
             track",
            track.name
        ),
        TransitionProblem::NotAdjacent {
            left_end,
            right_start,
        } => format!(
            "the left item ends at {} but the right item starts at {}; a transition sits on a \
             cut, where one item ends and the next starts",
            seconds(left_end),
            seconds(right_start)
        ),
        TransitionProblem::LongerThanItem {
            item,
            needed,
            available,
        } => format!(
            "the transition covers {} of item {item}, which lasts {}; shorten the transition or \
             change its alignment",
            seconds(needed),
            seconds(available)
        ),
        TransitionProblem::EdgeTaken { other, item } => {
            format!("transition {other} already joins item {item} on this side; remove it first")
        }
        TransitionProblem::Overlap { other, item } => format!(
            "the transition overlaps transition {other} inside item {item}; shorten one of them"
        ),
        TransitionProblem::HandleTooShort {
            item,
            edge,
            needed,
            available,
        } => format!(
            "item {item} must play {} {} but its media allows {}; shorten the transition, change \
             its alignment or trim the item to free media",
            seconds(needed),
            match edge {
                ItemEdge::Start => "before its start",
                ItemEdge::End => "past its end",
            },
            seconds(available)
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Flicks, ItemKind, Project, TransitionAlignment, ViolationCode,
        test_support::{example_project, millis},
        validate_project,
    };

    fn codes(project: &Project) -> Vec<ViolationCode> {
        validate_project(project)
            .into_iter()
            .map(|violation| violation.code)
            .collect()
    }

    /// The example's dissolve between the two interview clips of V1, at 10 s.
    fn dissolve(project: &mut Project) -> &mut crate::Transition {
        &mut project.sequences[0].tracks[0].transitions[0]
    }

    #[test]
    fn reports_transitions_off_the_cut() {
        let mut project = example_project();
        project.sequences[0].tracks[0].items[1].timeline_range =
            crate::test_support::range_ms(10_500, 3500);
        let violations = validate_project(&project);
        assert_eq!(violations.len(), 1, "{violations:#?}");
        assert_eq!(violations[0].code, ViolationCode::InvalidTransition);
        assert!(
            violations[0]
                .path
                .ends_with(".transitions[00000000-0001-7000-8000-000000000000]")
        );
        assert!(
            violations[0].message.contains("10.500s"),
            "{}",
            violations[0].message
        );
    }

    #[test]
    fn reports_missing_media_beyond_the_edges() {
        let mut project = example_project();
        // The fast clip plays 20 s → 28 s at 2×: 10 s of timeline before its start.
        dissolve(&mut project).alignment = TransitionAlignment::End;
        dissolve(&mut project).duration = millis(10_001);
        assert_eq!(
            codes(&project),
            [ViolationCode::InvalidTransition],
            "longer than the first clip (10 s)"
        );
        dissolve(&mut project).duration = millis(4000);
        assert_eq!(codes(&project), []);
        // The first clip then plays 49.5 s → 59.5 s of the 60 s interview: 0.5 s after its end.
        let ItemKind::MediaClip(clip) = &mut project.sequences[0].tracks[0].items[0].kind else {
            unreachable!("the example's first V1 item is a media clip")
        };
        clip.source_start = millis(49_500);
        dissolve(&mut project).alignment = TransitionAlignment::Start;
        assert_eq!(codes(&project), [ViolationCode::InsufficientHandles]);
    }

    #[test]
    fn reports_wrong_kinds_and_durations() {
        let mut project = example_project();
        dissolve(&mut project).kind = crate::TransitionKind::AudioCrossfade;
        assert_eq!(codes(&project), [ViolationCode::InvalidTransition]);
        let mut project = example_project();
        dissolve(&mut project).duration = Flicks::ZERO;
        assert_eq!(codes(&project), [ViolationCode::InvalidTransition]);
    }

    #[test]
    fn reports_duplicate_transition_ids() {
        let mut project = example_project();
        let copy = project.sequences[0].tracks[0].transitions[0].clone();
        project.sequences[0].tracks[3].transitions.push(copy);
        assert!(codes(&project).contains(&ViolationCode::DuplicateId));
    }
}
