//! The « cut » scope rule of AI smart cuts (prompt 22, ADR-0032).
//!
//! A cut removes time, so everything after it moves left and a clip crossing the scope's edges
//! is cut inside it only: the strict rule of other edits (items entirely inside the scope, nothing
//! shifted outside) would refuse every cut of a long interview. A cut is accepted when **the time
//! it removes** lies in the scope's range and **the tracks it edits** (the speech's tracks and
//! their linked partners, or every track) are the task's tracks. The plan guarantees the rest:
//! items lose only removed time and shift by the time removed before them.

use std::collections::BTreeSet;

use se_domain::{Project, SequenceId, TimeRange, TrackId, format_clock};
use se_protocol::TaskScope;

use super::scope::{describe_scope, track_allowed};
use crate::AppError;

/// Refuses cuts removing time outside `scope`'s range or editing tracks outside its tracks.
///
/// # Errors
/// [`AppError::OutOfScope`] naming the first cut or track outside the scope and what to do.
pub(crate) fn check_cut_scope(
    scope: &TaskScope,
    project: &Project,
    sequence_id: SequenceId,
    cuts: &[TimeRange],
    tracks: &BTreeSet<TrackId>,
) -> Result<(), AppError> {
    let scope_text = || describe_scope(scope, project);
    if sequence_id != scope.sequence_id {
        return Err(AppError::OutOfScope {
            reason: format!(
                "this task may only cut its own sequence ({}): leave the sequence out",
                scope_text()
            ),
        });
    }
    if let Some(range) = scope.range
        && let Some(outside) = cuts.iter().find(|cut| !range.contains_range(**cut))
    {
        return Err(AppError::OutOfScope {
            reason: format!(
                "the cut {}–{} lies outside this task's scope ({}): cut only between {} and {}",
                format_clock(outside.start()),
                format_clock(outside.end()),
                scope_text(),
                format_clock(range.start()),
                format_clock(range.end())
            ),
        });
    }
    let names = project
        .sequences
        .iter()
        .find(|sequence| sequence.id == sequence_id)
        .map(|sequence| &sequence.tracks);
    if let Some(track) = tracks.iter().find(|track| !track_allowed(scope, **track)) {
        let name = names
            .and_then(|all| all.iter().find(|candidate| candidate.id == *track))
            .map_or_else(|| track.to_string(), |track| track.name.clone());
        return Err(AppError::OutOfScope {
            reason: format!(
                "the cut would also edit track {name} (it holds the speech, or picture or sound \
                 linked to it, which must stay in sync), but this task may only change {}: ask \
                 the user for a task including {name}",
                scope_text()
            ),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use se_domain::{Flicks, test_support::example_project};

    use super::*;

    fn span(start_ms: u32, end_ms: u32) -> TimeRange {
        TimeRange::from_start_end(Flicks::from_millis(start_ms), Flicks::from_millis(end_ms))
            .unwrap()
    }

    fn scope(range: Option<TimeRange>, tracks: Option<Vec<TrackId>>) -> TaskScope {
        TaskScope {
            sequence_id: example_project().sequences[0].id,
            range,
            tracks,
        }
    }

    #[test]
    fn cuts_inside_the_range_and_tracks_pass() {
        let project = example_project();
        let sequence = &project.sequences[0];
        let tracks: BTreeSet<TrackId> = sequence.tracks.iter().map(|track| track.id).collect();
        let all = scope(Some(span(0, 120_000)), None);
        assert!(check_cut_scope(&all, &project, sequence.id, &[span(1000, 2000)], &tracks).is_ok());
    }

    #[test]
    fn cuts_outside_the_range_or_on_other_tracks_are_refused() {
        let project = example_project();
        let sequence = &project.sequences[0];
        let v1 = sequence.tracks[0].id;
        let a1 = sequence.tracks[3].id;
        let limited = scope(Some(span(0, 5000)), Some(vec![a1]));
        let late = check_cut_scope(
            &limited,
            &project,
            sequence.id,
            &[span(4500, 5500)],
            &BTreeSet::from([a1]),
        );
        assert!(
            matches!(late, Err(AppError::OutOfScope { reason }) if reason.contains("00:00:04.500"))
        );
        let linked = check_cut_scope(
            &limited,
            &project,
            sequence.id,
            &[span(1000, 2000)],
            &BTreeSet::from([a1, v1]),
        );
        assert!(
            matches!(linked, Err(AppError::OutOfScope { reason }) if reason.contains("track V1"))
        );
    }
}
