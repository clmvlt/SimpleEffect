//! Validation of the invariants that span several values of a document (`docs/architecture.md`
//! §5.3). Invariants of a single value (time range, frame rate, id, colour) are enforced by the
//! value types themselves, so a deserialized document can only break the rules checked here.
//!
//! [`validate_project`] never stops at the first problem: it reports every [`Violation`], in
//! document order, each with a stable [`ViolationCode`], a path and an actionable message.

mod animation_rules;
mod identity_rules;
mod item_rules;
mod link_rules;
mod media_rules;
mod report;
mod sequence_rules;
mod track_rules;
mod transition_rules;
mod violation;

pub use violation::{Violation, ViolationCode};

use crate::Project;
use item_rules::ItemContext;
use report::{Path, ProjectIndex, Report};

/// Every broken invariant of `project`; empty when the document is valid.
///
/// Checked: unique ids, existing references of the right asset kind, sequence formats, times
/// not before zero and within range, non-empty items, no overlap on a track, items sorted by
/// start, items allowed on their track kind, positive speeds, source ranges inside their media,
/// keyframes present, sorted and inside their item, value ranges (opacity, volume, pan, fades,
/// Bézier control points), link groups within one sequence and transitions on the cut between
/// adjacent items with enough media beyond their edges.
#[must_use]
pub fn validate_project(project: &Project) -> Vec<Violation> {
    let mut report = Report::default();
    identity_rules::check(project, &mut report);
    let index = ProjectIndex::new(project);
    let sequences = Path::default().field("sequences");
    for sequence in &project.sequences {
        let sequence_path = sequences.key(sequence.id);
        sequence_rules::check(sequence, &sequence_path, &mut report);
        for track in &sequence.tracks {
            let track_path = sequence_path.field("tracks").key(track.id);
            track_rules::check(track, &track_path, &mut report);
            transition_rules::check(track, &index, &track_path, &mut report);
            for item in &track.items {
                let context = ItemContext {
                    item,
                    track_kind: track.kind,
                    index: &index,
                    path: track_path.field("items").key(item.id),
                };
                item_rules::check(&context, &mut report);
            }
        }
    }
    link_rules::check(project, &mut report);
    report.into_violations()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::example_project;

    #[test]
    fn the_example_project_is_valid() {
        assert_eq!(validate_project(&example_project()), []);
    }

    #[test]
    fn reports_every_violation_with_a_path_to_it() {
        let mut project = example_project();
        project.sequences[0].markers[0].time = crate::test_support::millis(-1);
        project.sequences[0].tracks[0].pan = 3.0;
        let violations = validate_project(&project);
        assert_eq!(violations.len(), 2, "{violations:#?}");
        assert_eq!(
            violations[0].path,
            "sequences[00000000-0001-7000-8000-000000000000].markers[00000000-0001-7000-8000-000000000000].time"
        );
        assert_eq!(
            violations[1].path,
            "sequences[00000000-0001-7000-8000-000000000000].tracks[00000000-0001-7000-8000-000000000000].pan"
        );
    }
}
