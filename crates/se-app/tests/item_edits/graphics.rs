//! Graphics, batched keyframes and animation presets (prompt 15).

use se_app::item_labels;
use se_domain::{
    Animatable, AnimationPhase, DesignPixels, ItemKind, PropertyAnimation, ShapeKind, TrackKind,
    Vec2, test_support::example_ids,
};
use se_protocol::{AddGraphicRequest, ErrorCode, ItemAnimation, NewGraphic};

use super::support::{Opened, V1, V2, clip, code, item_id, open, project_with, secs, track_id};

/// V1: a clip over `[0, 10 s)`; V2: a clip over `[4 s, 6 s)`.
async fn edited() -> Opened {
    open(project_with(
        vec![clip(1, 0, 10, None)],
        vec![clip(2, 4, 6, None)],
        vec![],
    ))
    .await
}

fn request(opened: &Opened, at: i64, graphic: NewGraphic) -> AddGraphicRequest {
    AddGraphicRequest {
        project_id: opened.project,
        sequence_id: opened.sequence,
        at: secs(at),
        duration: secs(2),
        graphic,
    }
}

fn text() -> NewGraphic {
    NewGraphic::Text {
        content: "Titre".to_owned(),
    }
}

#[tokio::test]
async fn a_text_goes_on_the_highest_free_track_or_a_new_one_on_top() {
    let opened = edited().await;
    let placed = opened
        .service
        .add_graphic_item(&request(&opened, 0, text()))
        .await
        .unwrap();
    assert_eq!(placed.track_id, track_id(V2), "V2 is free over [0, 2 s)");
    let ItemKind::Text(text) = opened.item(placed.item_id).await.kind else {
        panic!("expected a text");
    };
    assert_eq!(text.content, "Titre");
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(item_labels::ADD_TEXT)
    );

    // V2 is busy over [4 s, 6 s): a new V3 is created above it, in the same undo step.
    let shape = NewGraphic::Shape {
        shape: ShapeKind::Ellipse,
    };
    let placed = opened
        .service
        .add_graphic_item(&request(&opened, 5, shape))
        .await
        .unwrap();
    let session = opened.session().await;
    let new_track = session
        .snapshot
        .tracks
        .iter()
        .find(|track| track.id == placed.track_id)
        .unwrap();
    assert_eq!(
        (new_track.kind, new_track.name.as_str()),
        (TrackKind::Video, "V3")
    );
    let sequence = &session.snapshot.sequences[0];
    assert_eq!(sequence.track_ids[2], placed.track_id, "right above V2");
    opened.service.undo(opened.project).await.unwrap();
    assert_eq!(opened.session().await.snapshot.tracks.len(), 3);
    assert_eq!(opened.spans(track_id(V1)).await, [(0, 10)]);
}

#[tokio::test]
async fn images_must_be_image_assets_and_durations_positive() {
    let opened = edited().await;
    let image = NewGraphic::Image {
        asset_id: example_ids::IMAGE_ASSET,
    };
    let placed = opened
        .service
        .add_graphic_item(&request(&opened, 0, image))
        .await
        .unwrap();
    assert!(matches!(
        opened.item(placed.item_id).await.kind,
        ItemKind::Image(_)
    ));
    let video = NewGraphic::Image {
        asset_id: example_ids::VIDEO_ASSET,
    };
    let error = opened
        .service
        .add_graphic_item(&request(&opened, 0, video))
        .await
        .unwrap_err();
    assert_eq!(code(error), ErrorCode::EditRejected);
    let mut empty = request(&opened, 0, text());
    empty.duration = secs(0);
    let error = opened.service.add_graphic_item(&empty).await.unwrap_err();
    assert_eq!(code(error), ErrorCode::EditRejected);
}

#[tokio::test]
async fn presets_become_keyframes_of_every_item_in_one_step() {
    let opened = edited().await;
    let first = opened
        .service
        .add_graphic_item(&request(&opened, 0, text()))
        .await
        .unwrap();
    let second = opened
        .service
        .add_graphic_item(&request(&opened, 7, text()))
        .await
        .unwrap();
    opened
        .service
        .apply_animation_preset(
            opened.project,
            &[first.item_id, second.item_id],
            "typewriter",
            AnimationPhase::In,
            secs(1),
            None,
        )
        .await
        .unwrap();
    for id in [first.item_id, second.item_id] {
        let ItemKind::Text(text) = opened.item(id).await.kind else {
            panic!("expected a text");
        };
        assert!(text.reveal.is_keyframed());
    }
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(item_labels::APPLY_ANIMATION)
    );
    // A clip has no reveal property: the whole command is refused.
    let error = opened
        .service
        .apply_animation_preset(
            opened.project,
            &[first.item_id, item_id(1)],
            "typewriter",
            AnimationPhase::Out,
            secs(1),
            None,
        )
        .await
        .unwrap_err();
    assert_eq!(code(error), ErrorCode::EditRejected);
}

#[tokio::test]
async fn several_properties_change_in_one_step() {
    let opened = edited().await;
    let placed = opened
        .service
        .add_graphic_item(&request(&opened, 0, text()))
        .await
        .unwrap();
    let changes = [
        ItemAnimation {
            item_id: placed.item_id,
            animation: PropertyAnimation::Position {
                value: Animatable::constant(Vec2::new(0.25, 0.75)),
            },
        },
        ItemAnimation {
            item_id: placed.item_id,
            animation: PropertyAnimation::Scale {
                value: Animatable::constant(Vec2::splat(2.0)),
            },
        },
    ];
    opened
        .service
        .set_item_animations(opened.project, &changes)
        .await
        .unwrap();
    let item = opened.item(placed.item_id).await;
    let transform = item.kind.transform().unwrap();
    assert_eq!(transform.scale, Animatable::constant(Vec2::splat(2.0)));
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(item_labels::TRANSFORM)
    );
    opened.service.undo(opened.project).await.unwrap();
    let item = opened.item(placed.item_id).await;
    assert_eq!(
        item.kind.transform().unwrap().position,
        Animatable::constant(Vec2::splat(0.5))
    );
    let size = [ItemAnimation {
        item_id: placed.item_id,
        animation: PropertyAnimation::FontSize {
            value: Animatable::constant(DesignPixels(-1.0)),
        },
    }];
    let error = opened
        .service
        .set_item_animations(opened.project, &size)
        .await
        .unwrap_err();
    assert_eq!(code(error), ErrorCode::EditRejected);
}
