//! Sequence use cases (prompt 16): versions in another format, format changes, renames and
//! removals — one undo step each.

use se_app::{AppError, sequence_labels};
use se_domain::{
    Animatable, ClipReframePolicy, DesignPixels, Format, FormatAdaptation, FormatPreset, FrameRate,
    ItemKind, ItemPatch, MediaClipPatch, PropertyAnimation, Reframe, TextItem, Vec2,
    test_support::item,
};
use se_protocol::{ErrorCode, ItemAnimation, ItemUpdate, ProtocolError};

use super::support::{Opened, clip, code, item_id, open, project_with, secs};

/// A clip on V1 over [0 s, 10 s) and a title on V2 over [2 s, 6 s) with a 1600 px wrapping width.
async fn opened() -> Opened {
    let mut title = TextItem::new("Un titre qui prend presque toute la largeur");
    title.style.max_width = Some(DesignPixels(1600.0));
    let title = item(
        2,
        se_domain::TimeRange::from_start_end(secs(2), secs(6)).unwrap(),
        ItemKind::Text(title),
    );
    open(project_with(
        vec![clip(1, 0, 10, None)],
        vec![title],
        vec![],
    ))
    .await
}

/// 9:16 at the frame rate of the example sequence.
fn vertical(opened_rate: FrameRate) -> Format {
    Format::from_preset(FormatPreset::Vertical9x16, opened_rate).unwrap()
}

fn fill_and_fit_texts() -> FormatAdaptation {
    FormatAdaptation {
        clip_reframe: ClipReframePolicy::Fill,
        fit_text_widths: true,
    }
}

impl Opened {
    async fn sequence_ids(&self) -> Vec<se_domain::SequenceId> {
        self.session()
            .await
            .snapshot
            .sequences
            .iter()
            .map(|sequence| sequence.id)
            .collect()
    }

    async fn frame_rate(&self) -> FrameRate {
        self.session().await.snapshot.sequences[0].format.frame_rate
    }

    /// The items of the sequence's tracks (first sequence when `sequence` is 0), track by track.
    async fn items_of(&self, sequence: usize) -> Vec<se_domain::Item> {
        let session = self.session().await;
        let entity = &session.snapshot.sequences[sequence];
        entity
            .track_ids
            .iter()
            .flat_map(|track| {
                let track = session
                    .snapshot
                    .tracks
                    .iter()
                    .find(|t| t.id == *track)
                    .unwrap();
                track.item_ids.clone()
            })
            .map(|id| {
                session
                    .snapshot
                    .items
                    .iter()
                    .find(|entity| entity.item.id == id)
                    .unwrap()
                    .item
                    .clone()
            })
            .collect()
    }
}

fn domain_code(error: AppError) -> String {
    let protocol = ProtocolError::from(error);
    assert_eq!(protocol.code, ErrorCode::EditRejected);
    protocol.details.unwrap()["domainCode"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn a_version_in_another_format_is_an_independent_sequence() {
    let opened = opened().await;
    let rate = opened.frame_rate().await;
    let version = opened
        .service
        .duplicate_sequence(
            opened.project,
            opened.sequence,
            "Main edit 9:16",
            Some(vertical(rate)),
            fill_and_fit_texts(),
        )
        .await
        .unwrap();
    assert_eq!(opened.sequence_ids().await, [opened.sequence, version]);
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(sequence_labels::CREATE_VERSION)
    );
    let session = opened.session().await;
    assert_eq!(session.snapshot.sequences[1].format, vertical(rate));
    assert_eq!(session.snapshot.sequences[1].name, "Main edit 9:16");
    let copies = opened.items_of(1).await;
    assert!(matches!(
        &copies[0].kind,
        ItemKind::MediaClip(clip) if clip.reframe == Reframe::fill()
    ));
    assert!(matches!(
        &copies[1].kind,
        ItemKind::Text(text) if text.style.max_width == Some(DesignPixels(972.0))
    ));

    // Editing the version leaves the source alone.
    let originals = opened.items_of(0).await;
    opened
        .service
        .delete_items(opened.project, &[copies[1].id], false)
        .await
        .unwrap();
    assert_eq!(opened.items_of(0).await, originals);
    assert_eq!(opened.items_of(1).await.len(), 1);

    opened.service.undo(opened.project).await.unwrap();
    opened.service.undo(opened.project).await.unwrap();
    assert_eq!(opened.sequence_ids().await, [opened.sequence]);
}

#[tokio::test]
async fn a_plain_duplicate_keeps_the_format() {
    let opened = opened().await;
    let copy = opened
        .service
        .duplicate_sequence(
            opened.project,
            opened.sequence,
            "Copie",
            None,
            FormatAdaptation::default(),
        )
        .await
        .unwrap();
    let session = opened.session().await;
    assert_eq!(session.snapshot.sequences[1].id, copy);
    assert_eq!(
        session.snapshot.sequences[1].format,
        session.snapshot.sequences[0].format
    );
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(sequence_labels::DUPLICATE)
    );
    let refused = opened
        .service
        .duplicate_sequence(
            opened.project,
            opened.sequence,
            "  ",
            None,
            FormatAdaptation::default(),
        )
        .await
        .unwrap_err();
    assert_eq!(code(refused), ErrorCode::InvalidRequest);
}

#[tokio::test]
async fn a_format_change_adapts_items_in_one_undo_step() {
    let opened = opened().await;
    let rate = opened.frame_rate().await;
    let before = opened.items_of(0).await;
    opened
        .service
        .set_sequence_format(
            opened.project,
            opened.sequence,
            vertical(rate),
            fill_and_fit_texts(),
        )
        .await
        .unwrap();
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(sequence_labels::FORMAT)
    );
    assert_eq!(
        opened.session().await.snapshot.sequences[0].format,
        vertical(rate)
    );
    let changed = opened.item(item_id(1)).await;
    assert!(matches!(changed.kind, ItemKind::MediaClip(clip) if clip.reframe == Reframe::fill()));

    opened.service.undo(opened.project).await.unwrap();
    let kinds = |items: Vec<se_domain::Item>| -> Vec<ItemKind> {
        items.into_iter().map(|item| item.kind).collect()
    };
    assert_eq!(kinds(opened.items_of(0).await), kinds(before));
}

#[tokio::test]
async fn the_frame_rate_of_a_sequence_with_items_stays() {
    let opened = opened().await;
    let rate = opened.frame_rate().await;
    let other = if rate == FrameRate::FPS_60 {
        FrameRate::FPS_25
    } else {
        FrameRate::FPS_60
    };
    let refused = opened
        .service
        .set_sequence_format(
            opened.project,
            opened.sequence,
            vertical(other),
            fill_and_fit_texts(),
        )
        .await
        .unwrap_err();
    assert_eq!(domain_code(refused), "FRAME_RATE_LOCKED");
    let refused = opened
        .service
        .duplicate_sequence(
            opened.project,
            opened.sequence,
            "60",
            Some(vertical(other)),
            fill_and_fit_texts(),
        )
        .await
        .unwrap_err();
    assert_eq!(domain_code(refused), "FRAME_RATE_LOCKED");
}

#[tokio::test]
async fn sequences_are_renamed_and_the_last_one_stays() {
    let opened = opened().await;
    opened
        .service
        .rename_sequence(opened.project, opened.sequence, "  Montage  ")
        .await
        .unwrap();
    assert_eq!(opened.session().await.snapshot.sequences[0].name, "Montage");
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(sequence_labels::RENAME)
    );
    let refused = opened
        .service
        .remove_sequence(opened.project, opened.sequence)
        .await
        .unwrap_err();
    assert_eq!(domain_code(refused), "LAST_SEQUENCE");

    let copy = opened
        .service
        .duplicate_sequence(
            opened.project,
            opened.sequence,
            "Copie",
            None,
            FormatAdaptation::default(),
        )
        .await
        .unwrap();
    opened
        .service
        .remove_sequence(opened.project, opened.sequence)
        .await
        .unwrap();
    assert_eq!(opened.sequence_ids().await, [copy]);
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(sequence_labels::REMOVE)
    );
    opened.service.undo(opened.project).await.unwrap();
    assert_eq!(opened.sequence_ids().await, [opened.sequence, copy]);
}

#[tokio::test]
async fn a_reframe_mode_and_a_compensating_scale_change_in_one_step() {
    let opened = opened().await;
    let update = ItemUpdate {
        item_id: item_id(1),
        patch: ItemPatch::MediaClip(MediaClipPatch {
            reframe: Some(Reframe::Custom),
            ..MediaClipPatch::default()
        }),
    };
    let scale = ItemAnimation {
        item_id: item_id(1),
        animation: PropertyAnimation::Scale {
            value: Animatable::constant(Vec2::splat(0.5)),
        },
    };
    opened
        .service
        .update_items(opened.project, &[update], &[scale])
        .await
        .unwrap();
    let ItemKind::MediaClip(clip) = opened.item(item_id(1)).await.kind else {
        unreachable!("item 1 is a clip")
    };
    assert_eq!(clip.reframe, Reframe::Custom);
    assert_eq!(clip.transform.scale, Animatable::constant(Vec2::splat(0.5)));
    assert_eq!(
        opened.undo_label().await.as_deref(),
        Some(se_app::item_labels::REFRAME)
    );

    // A focus needs a fill mode: refused together with a patch to fit, nothing changes.
    let refused = opened
        .service
        .update_items(
            opened.project,
            &[ItemUpdate {
                item_id: item_id(1),
                patch: ItemPatch::MediaClip(MediaClipPatch {
                    reframe: Some(Reframe::Fit),
                    ..MediaClipPatch::default()
                }),
            }],
            &[ItemAnimation {
                item_id: item_id(1),
                animation: PropertyAnimation::ReframeFocus {
                    value: Animatable::constant(Vec2::new(0.2, 0.5)),
                },
            }],
        )
        .await
        .unwrap_err();
    assert_eq!(code(refused), ErrorCode::EditRejected);
    assert!(matches!(
        opened.item(item_id(1)).await.kind,
        ItemKind::MediaClip(clip) if clip.reframe == Reframe::Custom
    ));
}
