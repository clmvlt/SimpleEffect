//! Turning the document tree into normalised entities.

use se_domain::{Project, Sequence, Track};
use se_protocol::{
    ItemEntity, MarkerEntity, ProjectHeader, ProjectSnapshot, SequenceEntity, TrackEntity,
};

/// Every entity of `project`, normalised by id.
#[must_use]
pub fn project_snapshot(project: &Project) -> ProjectSnapshot {
    let sequences = &project.sequences;
    let tracks = sequences.iter().flat_map(|sequence| {
        sequence
            .tracks
            .iter()
            .map(move |track| track_entity(sequence, track))
    });
    let items = sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| {
            track.items.iter().map(move |item| ItemEntity {
                track_id: track.id,
                item: item.clone(),
            })
        });
    let markers = sequences.iter().flat_map(|sequence| {
        sequence.markers.iter().map(move |marker| MarkerEntity {
            sequence_id: sequence.id,
            marker: marker.clone(),
        })
    });
    ProjectSnapshot {
        revision: project.revision,
        project: project_header(project),
        sequences: sequences.iter().map(sequence_entity).collect(),
        tracks: tracks.collect(),
        items: items.collect(),
        markers: markers.collect(),
        assets: project.assets.clone(),
    }
}

/// The project's own fields.
pub(crate) fn project_header(project: &Project) -> ProjectHeader {
    ProjectHeader {
        id: project.id,
        name: project.name.clone(),
        settings: project.settings,
        templates: project.templates.clone(),
        brand_kit: project.brand_kit.clone(),
    }
}

/// A sequence without its children, which it lists by id.
pub(crate) fn sequence_entity(sequence: &Sequence) -> SequenceEntity {
    SequenceEntity {
        id: sequence.id,
        name: sequence.name.clone(),
        format: sequence.format,
        track_ids: sequence.tracks.iter().map(|track| track.id).collect(),
        marker_ids: sequence.markers.iter().map(|marker| marker.id).collect(),
    }
}

/// A track header, the ids of its items and its transitions, pointing back to its sequence.
pub(crate) fn track_entity(sequence: &Sequence, track: &Track) -> TrackEntity {
    TrackEntity {
        id: track.id,
        sequence_id: sequence.id,
        kind: track.kind,
        name: track.name.clone(),
        muted: track.muted,
        solo: track.solo,
        locked: track.locked,
        hidden: track.hidden,
        volume_db: track.volume_db,
        pan: track.pan,
        item_ids: track.items.iter().map(|item| item.id).collect(),
        transitions: track.transitions.clone(),
    }
}

#[cfg(test)]
mod tests {
    use se_domain::test_support::example_project;

    use super::*;

    #[test]
    fn snapshots_list_every_entity_with_its_parent() {
        let project = example_project();
        let snapshot = project_snapshot(&project);
        let sequence = &project.sequences[0];
        assert_eq!(snapshot.revision, project.revision);
        assert_eq!(snapshot.project.name, project.name);
        assert_eq!(snapshot.sequences.len(), project.sequences.len());
        assert_eq!(snapshot.tracks.len(), sequence.tracks.len());
        let item_count: usize = sequence.tracks.iter().map(|track| track.items.len()).sum();
        assert_eq!(snapshot.items.len(), item_count);
        assert_eq!(snapshot.markers.len(), sequence.markers.len());
        assert_eq!(snapshot.assets, project.assets);
        let first_track = &sequence.tracks[0];
        assert_eq!(snapshot.sequences[0].track_ids[0], first_track.id);
        assert_eq!(snapshot.tracks[0].sequence_id, sequence.id);
        assert_eq!(
            snapshot.tracks[0].item_ids,
            first_track
                .items
                .iter()
                .map(|item| item.id)
                .collect::<Vec<_>>()
        );
        assert_eq!(snapshot.items[0].track_id, first_track.id);
        assert_eq!(snapshot.markers[0].sequence_id, sequence.id);
    }
}
