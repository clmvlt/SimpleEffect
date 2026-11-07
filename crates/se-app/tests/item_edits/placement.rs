//! Dropped media, moves, copies, pastes, deletes and links.

use se_app::{DEFAULT_IMAGE_DURATION, item_labels};
use se_domain::{LinkGroupId, TimeRange, test_support::example_ids};
use se_protocol::{ErrorCode, PastedItem, PlaceMediaRequest, PlacementMode, TrackTarget};

use super::support::{
    A1, V1, V2, clip, code, item_id, open, placement, project_with, secs, track_id,
};

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
