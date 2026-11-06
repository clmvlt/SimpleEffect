//! `DUPLICATE_ID`: ids are unique per entity kind across the whole project.

use std::{collections::HashSet, fmt::Display, hash::Hash};

use super::{Path, Report, ViolationCode};
use crate::Project;

/// Reports every id already used by an earlier entity of the same kind.
pub(super) fn check(project: &Project, report: &mut Report) {
    let root = Path::default();
    let assets = root.field("assets");
    report_duplicates(
        project
            .assets
            .iter()
            .map(|asset| (asset.id, assets.key(asset.id))),
        "asset",
        report,
    );
    let templates = root.field("templates");
    report_duplicates(
        project
            .templates
            .iter()
            .map(|template| (template.id, templates.key(template.id))),
        "template",
        report,
    );
    let sequences = root.field("sequences");
    report_duplicates(
        project
            .sequences
            .iter()
            .map(|sequence| (sequence.id, sequences.key(sequence.id))),
        "sequence",
        report,
    );
    check_sequence_children(project, &sequences, report);
}

/// Markers, tracks and items: unique across all sequences, not only within one.
fn check_sequence_children(project: &Project, sequences: &Path, report: &mut Report) {
    let with_paths = || {
        project
            .sequences
            .iter()
            .map(|sequence| (sequence, sequences.key(sequence.id)))
    };
    report_duplicates(
        with_paths().flat_map(|(sequence, path)| {
            let markers = path.field("markers");
            sequence
                .markers
                .iter()
                .map(move |marker| (marker.id, markers.key(marker.id)))
        }),
        "marker",
        report,
    );
    let tracks = || {
        with_paths().flat_map(|(sequence, path)| {
            let tracks = path.field("tracks");
            sequence
                .tracks
                .iter()
                .map(move |track| (track, tracks.key(track.id)))
        })
    };
    report_duplicates(
        tracks().map(|(track, path)| (track.id, path)),
        "track",
        report,
    );
    report_duplicates(
        tracks().flat_map(|(track, path)| {
            let items = path.field("items");
            track
                .items
                .iter()
                .map(move |item| (item.id, items.key(item.id)))
        }),
        "item",
        report,
    );
}

fn report_duplicates<Id: Copy + Eq + Hash + Display>(
    entries: impl Iterator<Item = (Id, Path)>,
    kind: &str,
    report: &mut Report,
) {
    let mut seen = HashSet::new();
    for (id, path) in entries {
        if !seen.insert(id) {
            report.push(
                ViolationCode::DuplicateId,
                &path,
                format!(
                    "{kind} id {id} is used by more than one {kind}; give each {kind} its own id"
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ViolationCode, test_support::example_project, validate_project};

    #[test]
    fn reports_items_sharing_an_id_across_tracks() {
        let mut project = example_project();
        let first_id = project.sequences[0].tracks[0].items[0].id;
        project.sequences[0].tracks[4].items[0].id = first_id;
        let violations = validate_project(&project);
        assert_eq!(violations.len(), 1, "{violations:#?}");
        assert_eq!(violations[0].code, ViolationCode::DuplicateId);
        assert!(violations[0].path.ends_with(&format!("items[{first_id}]")));
        assert!(violations[0].message.contains("item id"));
    }

    #[test]
    fn reports_duplicate_assets_tracks_and_markers() {
        let mut project = example_project();
        let asset = project.assets[0].clone();
        project.assets.push(asset);
        project.sequences[0].tracks[1].id = project.sequences[0].tracks[0].id;
        let marker = project.sequences[0].markers[0].clone();
        project.sequences[0].markers.push(marker);
        let codes: Vec<_> = validate_project(&project)
            .into_iter()
            .map(|violation| violation.code)
            .collect();
        assert_eq!(codes, [ViolationCode::DuplicateId; 3]);
    }
}
