//! Transitions, fades and volume curves (prompt 14).

use se_app::{item_labels, transition_labels};
use se_domain::{
    Animatable, Easing, FadeShape, Flicks, ItemKind, ItemPatch, Keyframe, MediaClipPatch,
    PropertyAnimation, TransitionAlignment, TransitionKind, TransitionPatch,
};
use se_protocol::{ErrorCode, ItemUpdate, NewTransition, PlacementMode};

use super::support::{
    Opened, V1, V2, clip_from, code, item_id, open, placement, project_with, secs, track_id,
};

/// V1: clip 1 `[0, 4 s)` playing from 10 s and clip 2 `[4 s, 8 s)` playing from 1 s, so the cut
/// at 4 s allows 1 s before it (the handle of clip 2) and 4 s after it (the length of clip 2).
/// V2: clips 5 and 6, which do not touch.
async fn two_clips() -> Opened {
    open(project_with(
        vec![clip_from(1, 0, 4, 10), clip_from(2, 4, 8, 1)],
        vec![clip_from(5, 0, 2, 0), clip_from(6, 3, 5, 0)],
        vec![],
    ))
    .await
}

fn cut(track: u64, left: u64, right: u64, seconds: i64) -> NewTransition {
    NewTransition {
        track_id: track_id(track),
        left_item_id: item_id(left),
        right_item_id: item_id(right),
        duration: secs(seconds),
        alignment: TransitionAlignment::Center,
    }
}

/// Two seconds and a flick: the longest centred cross-fade with at most 1 s before the cut.
fn longest_centred() -> Flicks {
    secs(2).checked_add(Flicks::new(1).unwrap()).unwrap()
}

#[tokio::test]
async fn a_cross_fade_is_shortened_to_its_cut_and_undone_in_one_step() {
    let opened = two_clips().await;
    let ids = opened
        .service
        .add_transitions(opened.project, &[cut(V1, 1, 2, 10)])
        .await
        .unwrap();
    let transitions = opened.transitions(track_id(V1)).await;
    let [transition] = transitions.as_slice() else {
        panic!("one transition expected, got {transitions:?}");
    };
    assert_eq!(transition.id, ids[0]);
    assert_eq!(transition.kind, TransitionKind::CrossDissolve);
    assert_eq!(transition.duration, longest_centred());
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(transition_labels::ADD)
    );
    opened.service.undo(opened.project).await.unwrap();
    assert_eq!(opened.transitions(track_id(V1)).await, []);
}

#[tokio::test]
async fn cuts_without_room_are_skipped_or_refused() {
    let opened = two_clips().await;
    let error = opened
        .service
        .add_transitions(opened.project, &[cut(V2, 5, 6, 1)])
        .await
        .unwrap_err();
    assert_eq!(code(error), ErrorCode::TransitionNoRoom);
    let ids = opened
        .service
        .add_transitions(opened.project, &[cut(V2, 5, 6, 1), cut(V1, 1, 2, 1)])
        .await
        .unwrap();
    assert_eq!(ids.len(), 1);
    assert_eq!(opened.transitions(track_id(V2)).await, []);
}

#[tokio::test]
async fn resizing_is_capped_by_the_cut_and_removing_is_one_step() {
    let opened = two_clips().await;
    let ids = opened
        .service
        .add_transitions(opened.project, &[cut(V1, 1, 2, 1)])
        .await
        .unwrap();
    let longer = TransitionPatch {
        duration: Some(secs(60)),
        alignment: None,
    };
    opened
        .service
        .update_transition(opened.project, ids[0], longer)
        .await
        .unwrap();
    assert_eq!(
        opened.transitions(track_id(V1)).await[0].duration,
        longest_centred()
    );
    let ending = TransitionPatch {
        duration: None,
        alignment: Some(TransitionAlignment::End),
    };
    opened
        .service
        .update_transition(opened.project, ids[0], ending)
        .await
        .unwrap();
    let transition = opened.transitions(track_id(V1)).await[0].clone();
    assert_eq!(transition.alignment, TransitionAlignment::End);
    assert_eq!(transition.duration, secs(1), "the handle of clip 2");
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(transition_labels::CHANGE)
    );
    opened
        .service
        .remove_transitions(opened.project, &ids)
        .await
        .unwrap();
    assert_eq!(opened.transitions(track_id(V1)).await, []);
    opened.service.undo(opened.project).await.unwrap();
    assert_eq!(opened.transitions(track_id(V1)).await, [transition]);
}

#[tokio::test]
async fn cross_fades_follow_items_moved_together_and_go_with_one_moved_away() {
    let opened = two_clips().await;
    let ids = opened
        .service
        .add_transitions(opened.project, &[cut(V1, 1, 2, 1)])
        .await
        .unwrap();
    let together = [placement(1, V1, 2), placement(2, V1, 6)];
    opened
        .service
        .move_items(opened.project, &together, PlacementMode::Overwrite)
        .await
        .unwrap();
    let kept = opened.transitions(track_id(V1)).await;
    let kept_ids: Vec<_> = kept.iter().map(|transition| transition.id).collect();
    assert_eq!(kept_ids, ids);
    let away = [placement(2, V2, 20)];
    opened
        .service
        .move_items(opened.project, &away, PlacementMode::Overwrite)
        .await
        .unwrap();
    assert_eq!(opened.transitions(track_id(V1)).await, []);
    opened.service.undo(opened.project).await.unwrap();
    assert_eq!(opened.transitions(track_id(V1)).await, kept);
}

#[tokio::test]
async fn fades_and_volume_curves_are_one_step_each() {
    let opened = two_clips().await;
    let fades = ItemUpdate {
        item_id: item_id(1),
        patch: ItemPatch::MediaClip(MediaClipPatch {
            fade_in: Some(secs(1)),
            fade_in_shape: Some(FadeShape::ConstantPower),
            ..MediaClipPatch::default()
        }),
    };
    opened
        .service
        .update_items(opened.project, &[fades], &[])
        .await
        .unwrap();
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(item_labels::CHANGE_FADES)
    );
    let curve = Animatable::Keyframed {
        keyframes: vec![
            Keyframe {
                time: secs(0),
                value: 0.0,
                easing: Easing::Linear,
            },
            Keyframe {
                time: secs(2),
                value: -12.0,
                easing: Easing::Linear,
            },
        ],
    };
    let volume = PropertyAnimation::VolumeDb {
        value: curve.clone(),
    };
    opened
        .service
        .set_item_animation(opened.project, item_id(1), volume)
        .await
        .unwrap();
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(item_labels::CHANGE_VOLUME)
    );
    let ItemKind::MediaClip(clip) = opened.item(item_id(1)).await.kind else {
        panic!("clip 1 is a media clip");
    };
    assert_eq!(clip.fade_in, secs(1));
    assert_eq!(clip.fade_in_shape, FadeShape::ConstantPower);
    assert_eq!(clip.volume_db, curve);
    let too_loud = PropertyAnimation::VolumeDb {
        value: Animatable::constant(40.0),
    };
    let error = opened
        .service
        .set_item_animation(opened.project, item_id(1), too_loud)
        .await
        .unwrap_err();
    assert_eq!(code(error), ErrorCode::EditRejected);
}
