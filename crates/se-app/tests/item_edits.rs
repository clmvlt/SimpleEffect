//! Item edit use cases (prompt 11) against in-memory ports: each gesture is one transaction, so
//! one undo step, and ripple edits keep linked items in sync.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{path::Path, path::PathBuf, sync::Arc, time::Duration};

use se_app::{
    AppError, DEFAULT_IMAGE_DURATION, ProjectService, ProjectServiceConfig, ProjectServiceDeps,
    item_labels,
    test_support::{
        FixedClock, InMemoryAppLibrary, InMemoryProjectRepository, RecordingEventSink,
        SequentialIds,
    },
};
use se_domain::{
    Flicks, Item, ItemEdge, ItemId, ItemKind, LinkGroupId, MediaClip, Project, ProjectId,
    SequenceId, TimeRange, TrackId, TrackKind, TrackPatch,
    test_support::{example_ids, example_project, item, track},
};
use se_protocol::{
    ErrorCode, ItemPlacement, ItemTrim, PastedItem, PlaceMediaRequest, PlacementMode,
    ProjectSession, ProtocolError, TrackTarget,
};

const V1: u64 = 101;
const V2: u64 = 102;
const A1: u64 = 103;

fn secs(seconds: i64) -> Flicks {
    Flicks::new(seconds * Flicks::PER_SECOND).unwrap()
}

fn track_id(index: u64) -> TrackId {
    TrackId::from_test_index(index)
}

fn item_id(index: u64) -> ItemId {
    ItemId::from_test_index(index)
}

/// A clip of the example's 60 s interview over `[start, end)` seconds.
fn clip(index: u64, start: i64, end: i64, group: Option<u64>) -> Item {
    let range = TimeRange::from_start_end(secs(start), secs(end)).unwrap();
    let mut clip = item(
        index,
        range,
        ItemKind::MediaClip(MediaClip::new(example_ids::VIDEO_ASSET)),
    );
    clip.link_group = group.map(LinkGroupId::from_test_index);
    clip
}

/// The example project reduced to one sequence with tracks V1, V2, A1 holding these items.
fn project_with(video1: Vec<Item>, video2: Vec<Item>, audio1: Vec<Item>) -> Project {
    let mut project = example_project();
    project.sequences.truncate(1);
    project.sequences[0].tracks = vec![
        track(V1, TrackKind::Video, "V1", video1),
        track(V2, TrackKind::Video, "V2", video2),
        track(A1, TrackKind::Audio, "A1", audio1),
    ];
    project
}

struct Opened {
    service: ProjectService,
    project: ProjectId,
    sequence: SequenceId,
}

/// A service with `project` open from an in-memory folder.
async fn open(project: Project) -> Opened {
    let repository = Arc::new(InMemoryProjectRepository::default());
    repository.insert("/projects/Film.seproj", project.clone(), Vec::new());
    let deps = ProjectServiceDeps {
        repository,
        library: Arc::new(InMemoryAppLibrary::default()),
        events: Arc::new(RecordingEventSink::default()),
        clock: Arc::new(FixedClock::default()),
        ids: Arc::new(SequentialIds::default()),
    };
    let config = ProjectServiceConfig {
        default_parent_dir: PathBuf::from("/projects"),
        save_delay: Duration::from_secs(2),
    };
    let service = ProjectService::new(deps, config);
    service
        .open_project(Path::new("/projects/Film.seproj"))
        .await
        .unwrap();
    Opened {
        service,
        project: project.id,
        sequence: project.sequences[0].id,
    }
}

impl Opened {
    async fn session(&self) -> ProjectSession {
        self.service.project_session(self.project).await.unwrap()
    }

    /// `(start, end)` in whole seconds of the items of a track, in order.
    async fn spans(&self, track: TrackId) -> Vec<(i64, i64)> {
        let session = self.session().await;
        let entity = session
            .snapshot
            .tracks
            .iter()
            .find(|entity| entity.id == track)
            .unwrap();
        entity
            .item_ids
            .iter()
            .map(|id| {
                let item = session
                    .snapshot
                    .items
                    .iter()
                    .find(|i| i.item.id == *id)
                    .unwrap();
                let range = item.item.timeline_range;
                (
                    range.start().get() / Flicks::PER_SECOND,
                    range.end().get() / Flicks::PER_SECOND,
                )
            })
            .collect()
    }

    async fn item(&self, id: ItemId) -> Item {
        let session = self.session().await;
        session
            .snapshot
            .items
            .into_iter()
            .find(|entity| entity.item.id == id)
            .unwrap()
            .item
    }

    async fn undo_label(&self) -> Option<String> {
        self.session().await.history.undo_label
    }

    /// Undoes once and checks that the tracks are back as `before`.
    async fn undo_restores(&self, before: &[(u64, Vec<(i64, i64)>)]) {
        self.service.undo(self.project).await.unwrap();
        for (track, spans) in before {
            assert_eq!(&self.spans(track_id(*track)).await, spans, "track {track}");
        }
    }
}

fn placement(item: u64, track: u64, start: i64) -> ItemPlacement {
    ItemPlacement {
        item_id: item_id(item),
        track_id: track_id(track),
        start: secs(start),
    }
}

fn code(error: AppError) -> ErrorCode {
    ProtocolError::from(error).code
}

#[tokio::test]
async fn dropped_video_becomes_linked_picture_and_sound_in_one_step() {
    let opened = open(project_with(
        vec![clip(1, 0, 10, None)],
        Vec::new(),
        Vec::new(),
    ))
    .await;
    let request = PlaceMediaRequest {
        project_id: opened.project,
        sequence_id: opened.sequence,
        asset_ids: vec![example_ids::VIDEO_ASSET, example_ids::IMAGE_ASSET],
        start: secs(4),
        video_track: TrackTarget::Existing {
            track_id: track_id(V1),
        },
        audio_track: TrackTarget::Existing {
            track_id: track_id(A1),
        },
        mode: PlacementMode::Overwrite,
    };
    let ids = opened.service.place_media(&request).await.unwrap();
    assert_eq!(
        ids.len(),
        3,
        "picture and sound of the video, then the image"
    );
    assert_eq!(
        opened.spans(track_id(V1)).await,
        [(0, 4), (4, 64), (64, 69)]
    );
    assert_eq!(opened.spans(track_id(A1)).await, [(4, 64)]);
    let (picture, sound) = (opened.item(ids[0]).await, opened.item(ids[1]).await);
    assert!(picture.link_group.is_some());
    assert_eq!(picture.link_group, sound.link_group);
    let image = opened.item(ids[2]).await;
    assert_eq!(image.timeline_range.duration(), DEFAULT_IMAGE_DURATION);
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(item_labels::ADD_CLIPS)
    );
    opened
        .undo_restores(&[(V1, vec![(0, 10)]), (A1, Vec::new())])
        .await;
}

#[tokio::test]
async fn dropping_on_a_new_track_adds_it_in_the_same_step() {
    let opened = open(project_with(Vec::new(), Vec::new(), Vec::new())).await;
    let request = PlaceMediaRequest {
        project_id: opened.project,
        sequence_id: opened.sequence,
        asset_ids: vec![example_ids::AUDIO_ASSET],
        start: secs(0),
        video_track: TrackTarget::New,
        audio_track: TrackTarget::New,
        mode: PlacementMode::Insert,
    };
    opened.service.place_media(&request).await.unwrap();
    let session = opened.session().await;
    let names: Vec<&str> = session
        .snapshot
        .tracks
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(
        names,
        ["V1", "V2", "A1", "A2"],
        "only the audio track was needed"
    );
    opened.service.undo(opened.project).await.unwrap();
    assert_eq!(opened.session().await.snapshot.tracks.len(), 3);
}

#[tokio::test]
async fn linked_items_move_together_and_overwrite_what_they_land_on() {
    let opened = open(project_with(
        vec![clip(1, 0, 2, Some(1)), clip(2, 5, 9, None)],
        Vec::new(),
        vec![clip(3, 0, 2, Some(1))],
    ))
    .await;
    let moves = [placement(1, V1, 4), placement(3, A1, 4)];
    opened
        .service
        .move_items(opened.project, &moves, PlacementMode::Overwrite)
        .await
        .unwrap();
    assert_eq!(opened.spans(track_id(V1)).await, [(4, 6), (6, 9)]);
    assert_eq!(opened.spans(track_id(A1)).await, [(4, 6)]);
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(item_labels::MOVE)
    );
    opened
        .undo_restores(&[(V1, vec![(0, 2), (5, 9)]), (A1, vec![(0, 2)])])
        .await;
}

#[tokio::test]
async fn an_insert_move_pushes_what_follows_on_linked_tracks() {
    let opened = open(project_with(
        vec![clip(1, 0, 2, None), clip(2, 4, 6, Some(2))],
        vec![clip(4, 4, 5, None)],
        vec![clip(3, 4, 6, Some(2))],
    ))
    .await;
    opened
        .service
        .move_items(
            opened.project,
            &[placement(1, V1, 4)],
            PlacementMode::Insert,
        )
        .await
        .unwrap();
    assert_eq!(opened.spans(track_id(V1)).await, [(4, 6), (6, 8)]);
    assert_eq!(
        opened.spans(track_id(A1)).await,
        [(6, 8)],
        "linked sound follows"
    );
    assert_eq!(
        opened.spans(track_id(V2)).await,
        [(4, 5)],
        "unrelated track stays"
    );
}

#[tokio::test]
async fn duplicates_and_pastes_are_new_linked_items() {
    let opened = open(project_with(
        vec![clip(1, 0, 2, Some(1))],
        Vec::new(),
        vec![clip(3, 0, 2, Some(1))],
    ))
    .await;
    let copies = opened
        .service
        .duplicate_items(
            opened.project,
            &[placement(1, V1, 2), placement(3, A1, 2)],
            PlacementMode::Insert,
        )
        .await
        .unwrap();
    let (picture, sound) = (opened.item(copies[0]).await, opened.item(copies[1]).await);
    assert_eq!(picture.link_group, sound.link_group);
    assert_ne!(picture.link_group, Some(LinkGroupId::from_test_index(1)));
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(item_labels::DUPLICATE)
    );

    let mut pasted = clip(1, 10, 12, None);
    pasted.timeline_range = TimeRange::from_start_end(secs(10), secs(12)).unwrap();
    let ids = opened
        .service
        .paste_items(
            opened.project,
            &[PastedItem {
                track_id: track_id(V2),
                item: pasted,
            }],
            PlacementMode::Overwrite,
        )
        .await
        .unwrap();
    assert_ne!(ids[0], item_id(1));
    assert_eq!(opened.spans(track_id(V2)).await, [(10, 12)]);
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(item_labels::PASTE)
    );
}

#[tokio::test]
async fn ripple_trims_shift_what_follows_on_the_concerned_tracks() {
    let opened = open(project_with(
        vec![clip(1, 0, 4, Some(1)), clip(2, 4, 8, Some(2))],
        vec![clip(5, 6, 7, None)],
        vec![clip(3, 0, 4, Some(1)), clip(4, 4, 8, Some(2))],
    ))
    .await;
    let trims = [
        ItemTrim {
            item_id: item_id(1),
            edge: ItemEdge::End,
            to: secs(3),
        },
        ItemTrim {
            item_id: item_id(3),
            edge: ItemEdge::End,
            to: secs(3),
        },
    ];
    opened
        .service
        .trim_items(opened.project, &trims, true)
        .await
        .unwrap();
    assert_eq!(opened.spans(track_id(V1)).await, [(0, 3), (3, 7)]);
    assert_eq!(opened.spans(track_id(A1)).await, [(0, 3), (3, 7)]);
    assert_eq!(opened.spans(track_id(V2)).await, [(6, 7)]);
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(item_labels::RIPPLE_TRIM)
    );
    opened
        .undo_restores(&[(V1, vec![(0, 4), (4, 8)]), (A1, vec![(0, 4), (4, 8)])])
        .await;

    let lengthen = [ItemTrim {
        item_id: item_id(2),
        edge: ItemEdge::Start,
        to: secs(3),
    }];
    opened
        .service
        .trim_items(opened.project, &lengthen, true)
        .await
        .unwrap_err();
    let plain = [ItemTrim {
        item_id: item_id(2),
        edge: ItemEdge::End,
        to: secs(10),
    }];
    opened
        .service
        .trim_items(opened.project, &plain, false)
        .await
        .unwrap();
    assert_eq!(opened.spans(track_id(V1)).await, [(0, 4), (4, 10)]);
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(item_labels::TRIM)
    );
}

#[tokio::test]
async fn split_keeps_the_right_parts_linked() {
    let opened = open(project_with(
        vec![clip(1, 0, 4, Some(1))],
        Vec::new(),
        vec![clip(3, 0, 4, Some(1))],
    ))
    .await;
    let parts = opened
        .service
        .split_items(opened.project, &[item_id(1), item_id(3)], secs(1))
        .await
        .unwrap();
    assert_eq!(opened.spans(track_id(V1)).await, [(0, 1), (1, 4)]);
    let (picture, sound) = (opened.item(parts[0]).await, opened.item(parts[1]).await);
    assert!(picture.link_group.is_some());
    assert_eq!(picture.link_group, sound.link_group);
    assert_ne!(picture.link_group, opened.item(item_id(1)).await.link_group);
    opened
        .undo_restores(&[(V1, vec![(0, 4)]), (A1, vec![(0, 4)])])
        .await;
}

#[tokio::test]
async fn deletes_leave_or_close_gaps() {
    let items = || {
        project_with(
            vec![clip(1, 0, 2, Some(1)), clip(2, 2, 5, Some(2))],
            vec![clip(5, 3, 4, None)],
            vec![clip(3, 0, 2, Some(1)), clip(4, 2, 5, Some(2))],
        )
    };
    let opened = open(items()).await;
    opened
        .service
        .delete_items(opened.project, &[item_id(1), item_id(3)], false)
        .await
        .unwrap();
    assert_eq!(opened.spans(track_id(V1)).await, [(2, 5)]);
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(item_labels::DELETE)
    );
    opened.service.undo(opened.project).await.unwrap();
    opened
        .service
        .delete_items(opened.project, &[item_id(1), item_id(3)], true)
        .await
        .unwrap();
    assert_eq!(opened.spans(track_id(V1)).await, [(0, 3)]);
    assert_eq!(opened.spans(track_id(A1)).await, [(0, 3)]);
    assert_eq!(opened.spans(track_id(V2)).await, [(3, 4)]);
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(item_labels::RIPPLE_DELETE)
    );
}

#[tokio::test]
async fn items_link_and_unlink_in_one_step_each() {
    let opened = open(project_with(
        vec![clip(1, 0, 2, None)],
        Vec::new(),
        vec![clip(3, 0, 2, None)],
    ))
    .await;
    let group = opened
        .service
        .link_items(opened.project, &[item_id(1), item_id(3)])
        .await
        .unwrap();
    assert_eq!(opened.item(item_id(3)).await.link_group, Some(group));
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(item_labels::LINK)
    );
    opened
        .service
        .unlink_items(opened.project, &[item_id(1), item_id(3)])
        .await
        .unwrap();
    assert_eq!(opened.item(item_id(1)).await.link_group, None);
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(item_labels::UNLINK)
    );
    let lone = opened
        .service
        .link_items(opened.project, &[item_id(1)])
        .await;
    assert_eq!(code(lone.unwrap_err()), ErrorCode::EditRejected);
}

#[tokio::test]
async fn resizing_a_gap_shifts_what_follows_on_linked_tracks() {
    let opened = open(project_with(
        vec![clip(1, 0, 2, None), clip(2, 4, 6, Some(2))],
        vec![clip(5, 4, 5, None)],
        vec![clip(3, 4, 6, Some(2))],
    ))
    .await;
    let gap = TimeRange::from_start_end(secs(2), secs(4)).unwrap();
    opened
        .service
        .resize_gap(opened.project, track_id(V1), gap, secs(5))
        .await
        .unwrap();
    assert_eq!(opened.spans(track_id(V1)).await, [(0, 2), (7, 9)]);
    assert_eq!(opened.spans(track_id(A1)).await, [(7, 9)]);
    assert_eq!(opened.spans(track_id(V2)).await, [(4, 5)]);
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(item_labels::RESIZE_GAP)
    );
    opened.service.undo(opened.project).await.unwrap();
    opened
        .service
        .resize_gap(opened.project, track_id(V1), gap, Flicks::ZERO)
        .await
        .unwrap();
    assert_eq!(opened.spans(track_id(V1)).await, [(0, 2), (2, 4)]);
    assert_eq!(opened.spans(track_id(A1)).await, [(2, 4)]);
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(item_labels::CLOSE_GAP)
    );
}

#[tokio::test]
async fn a_gap_is_not_closed_over_other_content_of_a_linked_track() {
    let opened = open(project_with(
        vec![clip(1, 0, 2, None), clip(2, 4, 6, Some(2))],
        Vec::new(),
        vec![clip(4, 0, 3, None), clip(3, 4, 6, Some(2))],
    ))
    .await;
    let gap = TimeRange::from_start_end(secs(2), secs(4)).unwrap();
    let error = opened
        .service
        .resize_gap(opened.project, track_id(V1), gap, Flicks::ZERO)
        .await
        .unwrap_err();
    assert_eq!(code(error), ErrorCode::RippleBlocked);
    assert!(
        !opened.session().await.history.can_undo,
        "nothing committed"
    );
    opened
        .service
        .update_track(
            opened.project,
            track_id(A1),
            TrackPatch {
                locked: Some(true),
                ..TrackPatch::default()
            },
        )
        .await
        .unwrap();
    opened
        .service
        .resize_gap(opened.project, track_id(V1), gap, Flicks::ZERO)
        .await
        .unwrap();
    assert_eq!(opened.spans(track_id(V1)).await, [(0, 2), (2, 4)]);
    assert_eq!(
        opened.spans(track_id(A1)).await,
        [(0, 3), (4, 6)],
        "locked: left out"
    );
}

#[tokio::test]
async fn locked_tracks_are_never_changed() {
    let mut project = project_with(
        vec![clip(1, 0, 2, None)],
        vec![clip(2, 0, 4, None)],
        Vec::new(),
    );
    project.sequences[0].tracks[1].locked = true;
    let opened = open(project).await;
    let error = opened
        .service
        .move_items(
            opened.project,
            &[placement(1, V2, 1)],
            PlacementMode::Overwrite,
        )
        .await
        .unwrap_err();
    let protocol = ProtocolError::from(error);
    assert_eq!(protocol.code, ErrorCode::EditRejected);
    assert_eq!(protocol.details.unwrap()["domainCode"], "TRACK_LOCKED");
    let split = opened
        .service
        .split_items(opened.project, &[item_id(2)], secs(1))
        .await;
    assert!(split.is_err());
    assert_eq!(opened.spans(track_id(V2)).await, [(0, 4)]);
    assert_eq!(opened.spans(track_id(V1)).await, [(0, 2)]);
}
