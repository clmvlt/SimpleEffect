//! Track and marker use cases against in-memory ports: each edit is one undo step.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{path::Path, path::PathBuf, sync::Arc, time::Duration};

use se_app::{
    ProjectService, ProjectServiceConfig, ProjectServiceDeps, marker_labels,
    test_support::{
        FixedClock, InMemoryAppLibrary, InMemoryProjectRepository, RecordingEventSink,
        SequentialIds,
    },
    track_labels,
};
use se_domain::{
    Flicks, Format, FormatPreset, FrameRate, MarkerColor, MarkerPatch, ProjectId, SequenceId,
    TrackKind, TrackPatch,
};
use se_protocol::{ErrorCode, ProjectSession, ProtocolError};

fn service() -> ProjectService {
    let deps = ProjectServiceDeps {
        repository: Arc::new(InMemoryProjectRepository::default()),
        library: Arc::new(InMemoryAppLibrary::default()),
        events: Arc::new(RecordingEventSink::default()),
        clock: Arc::new(FixedClock::default()),
        ids: Arc::new(SequentialIds::default()),
    };
    let config = ProjectServiceConfig {
        default_parent_dir: PathBuf::from("/projects"),
        save_delay: Duration::from_secs(2),
    };
    ProjectService::new(deps, config)
}

/// A service with a new project open, its id and the id of its default sequence.
async fn open_project() -> (ProjectService, ProjectId, SequenceId) {
    let service = service();
    let format = Format::from_preset(FormatPreset::Hd16x9, FrameRate::FPS_25).unwrap();
    let session = service
        .create_project("Film", Path::new("/projects"), format)
        .await
        .unwrap();
    let project_id = session.snapshot.project.id;
    let sequence_id = session.snapshot.sequences[0].id;
    (service, project_id, sequence_id)
}

async fn session(service: &ProjectService, project_id: ProjectId) -> ProjectSession {
    service.project_session(project_id).await.unwrap()
}

/// Names of the sequence's tracks in list order.
fn track_names(session: &ProjectSession) -> Vec<String> {
    let order = &session.snapshot.sequences[0].track_ids;
    order
        .iter()
        .map(|id| {
            let track = session.snapshot.tracks.iter().find(|t| t.id == *id);
            track.unwrap().name.clone()
        })
        .collect()
}

#[tokio::test]
async fn tracks_are_added_named_reordered_and_removed_one_undo_step_each() {
    let (service, project, sequence) = open_project().await;
    let video = service
        .add_track(project, sequence, TrackKind::Video, Some(2))
        .await
        .unwrap();
    let added = session(&service, project).await;
    assert_eq!(track_names(&added), ["V1", "V2", "V3", "A1", "A2"]);
    assert_eq!(added.history.undo_label.as_deref(), Some(track_labels::ADD));
    service
        .add_track(project, sequence, TrackKind::Audio, None)
        .await
        .unwrap();
    assert_eq!(
        track_names(&session(&service, project).await),
        ["V1", "V2", "V3", "A1", "A2", "A3"]
    );

    let mut order = session(&service, project).await.snapshot.sequences[0]
        .track_ids
        .clone();
    order.swap(0, 2);
    service
        .reorder_tracks(project, sequence, order)
        .await
        .unwrap();
    let reordered = session(&service, project).await;
    assert_eq!(track_names(&reordered)[..3], ["V3", "V2", "V1"]);
    assert_eq!(
        reordered.history.undo_label.as_deref(),
        Some(track_labels::REORDER)
    );

    service.remove_track(project, video).await.unwrap();
    assert_eq!(track_names(&session(&service, project).await).len(), 5);
    service.undo(project).await.unwrap();
    service.undo(project).await.unwrap();
    assert_eq!(
        track_names(&session(&service, project).await),
        ["V1", "V2", "V3", "A1", "A2", "A3"]
    );
}

#[tokio::test]
async fn header_changes_are_labelled_and_a_locked_track_cannot_be_removed() {
    let (service, project, _) = open_project().await;
    let track = session(&service, project).await.snapshot.tracks[0].id;
    let lock = TrackPatch {
        locked: Some(true),
        ..TrackPatch::default()
    };
    service.update_track(project, track, lock).await.unwrap();
    let locked = session(&service, project).await;
    assert!(locked.snapshot.tracks[0].locked);
    assert_eq!(
        locked.history.undo_label.as_deref(),
        Some(track_labels::LOCK)
    );
    let refused = service.remove_track(project, track).await.unwrap_err();
    assert_eq!(ProtocolError::from(refused).code, ErrorCode::EditRejected);

    let rename = TrackPatch {
        name: Some("Titres".to_owned()),
        ..TrackPatch::default()
    };
    service.update_track(project, track, rename).await.unwrap();
    assert_eq!(
        session(&service, project).await.snapshot.tracks[0].name,
        "Titres"
    );
    let empty = TrackPatch {
        name: Some(String::new()),
        ..TrackPatch::default()
    };
    let refused = service
        .update_track(project, track, empty)
        .await
        .unwrap_err();
    assert_eq!(ProtocolError::from(refused).code, ErrorCode::EditRejected);
    service.undo(project).await.unwrap();
    assert_eq!(
        session(&service, project).await.snapshot.tracks[0].name,
        "V1"
    );
}

#[tokio::test]
async fn markers_are_added_changed_and_removed_one_undo_step_each() {
    let (service, project, sequence) = open_project().await;
    let two_seconds = Flicks::new(2 * Flicks::PER_SECOND).unwrap();
    let marker = service
        .add_marker(project, sequence, two_seconds, String::new(), None)
        .await
        .unwrap();
    let added = session(&service, project).await;
    assert_eq!(added.snapshot.markers[0].marker.color, MarkerColor::Green);
    assert_eq!(added.snapshot.markers[0].marker.time, two_seconds);
    assert_eq!(
        added.history.undo_label.as_deref(),
        Some(marker_labels::ADD)
    );

    let rename = MarkerPatch {
        label: Some("Intro".to_owned()),
        ..MarkerPatch::default()
    };
    service
        .update_marker(project, marker, rename)
        .await
        .unwrap();
    let renamed = session(&service, project).await;
    assert_eq!(renamed.snapshot.markers[0].marker.label, "Intro");
    assert_eq!(
        renamed.history.undo_label.as_deref(),
        Some(marker_labels::RENAME)
    );

    service.remove_marker(project, marker).await.unwrap();
    assert_eq!(session(&service, project).await.snapshot.markers, []);
    service.undo(project).await.unwrap();
    assert_eq!(session(&service, project).await.snapshot.markers.len(), 1);
    let before_zero = Flicks::new(-1).unwrap();
    let refused = service
        .add_marker(project, sequence, before_zero, String::new(), None)
        .await
        .unwrap_err();
    assert_eq!(ProtocolError::from(refused).code, ErrorCode::EditRejected);
}
