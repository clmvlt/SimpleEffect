//! Trims, splits, gaps and locked tracks.

use se_app::item_labels;
use se_domain::{Flicks, ItemEdge, TimeRange, TrackPatch};
use se_protocol::{ErrorCode, ItemTrim, PlacementMode, ProtocolError};

use super::support::{
    A1, V1, V2, clip, code, item_id, open, placement, project_with, secs, track_id,
};

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
