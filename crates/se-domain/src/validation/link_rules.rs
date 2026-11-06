//! `LINK_GROUP_ACROSS_SEQUENCES`: the items of a link group sit in one sequence.

use std::collections::HashMap;

use super::{Path, Report, ViolationCode};
use crate::{LinkGroupId, Project, SequenceId};

/// Reports each linked item whose group already has members in another sequence (linked items
/// move and trim together, which only makes sense on one timeline).
pub(super) fn check(project: &Project, report: &mut Report) {
    let mut group_sequences: HashMap<LinkGroupId, SequenceId> = HashMap::new();
    let sequences = Path::default().field("sequences");
    for sequence in &project.sequences {
        let sequence_path = sequences.key(sequence.id);
        for track in &sequence.tracks {
            let items_path = sequence_path.field("tracks").key(track.id).field("items");
            for item in &track.items {
                let Some(group) = item.link_group else {
                    continue;
                };
                let first = *group_sequences.entry(group).or_insert(sequence.id);
                if first != sequence.id {
                    report.push(
                        ViolationCode::LinkGroupAcrossSequences,
                        &items_path.key(item.id).field("linkGroup"),
                        format!(
                            "link group {group} already has items in sequence {first}; unlink this \
                             item or keep linked items in one sequence"
                        ),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        SequenceId, TrackId, ViolationCode, test_support::example_project, validate_project,
    };

    #[test]
    fn reports_link_groups_spanning_sequences() {
        let mut project = example_project();
        let mut other = project.sequences[0].clone();
        other.id = SequenceId::from_test_index(2);
        other.markers.clear();
        // Keep only A1, whose clip is linked to clip 1 of the first sequence.
        other.tracks = vec![other.tracks[3].clone()];
        other.tracks[0].id = TrackId::from_test_index(20);
        other.tracks[0].items[0].id = crate::ItemId::from_test_index(20);
        project.sequences.push(other);
        let violations = validate_project(&project);
        assert_eq!(violations.len(), 1, "{violations:#?}");
        assert_eq!(violations[0].code, ViolationCode::LinkGroupAcrossSequences);
        assert!(violations[0].path.ends_with("linkGroup"));
    }
}
