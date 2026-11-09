//! `auto_reframe` against the example project (prompt 26): the clips of the scope follow their
//! subject once their media's faces are analysed, a 9:16 version is added after the task's
//! sequence, a call that does not wait answers `finished: false`, and refusals (bad formats,
//! smoothing out of range, detection unavailable).

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

mod support;

use se_domain::{ItemKind, Project};
use serde_json::json;
use support::Bed;

/// Whether every media clip of video tracks of sequence `index` is tracked.
fn all_tracked(project: &Project, index: usize) -> bool {
    project.sequences[index]
        .tracks
        .iter()
        .filter(|track| track.kind == se_domain::TrackKind::Video)
        .flat_map(|track| &track.items)
        .filter_map(|item| match &item.kind {
            ItemKind::MediaClip(clip) => Some(clip),
            _ => None,
        })
        .filter(|clip| clip.asset_id == se_domain::test_support::example_ids::VIDEO_ASSET)
        .all(|clip| clip.reframe.tracking().is_some())
}

#[tokio::test]
async fn clips_of_the_scope_follow_their_subject() {
    let bed = Bed::open().await;
    bed.services.sampler.set_default_duration(60);
    let answer = bed.json("auto_reframe", json!({ "smoothing": 0.6 })).await;
    assert_eq!(answer["finished"], true, "{answer}");
    assert!(
        answer["edit"]["items"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
    );
    let project = bed.document().await;
    let tracked: Vec<f64> = project
        .items()
        .filter_map(|item| match &item.kind {
            ItemKind::MediaClip(clip) => clip.reframe.tracking().map(|settings| settings.smoothing),
            _ => None,
        })
        .collect();
    assert_ne!(tracked, Vec::<f64>::new());
    assert!(
        tracked
            .iter()
            .all(|smoothing| (smoothing - 0.6).abs() < 1e-12)
    );
}

#[tokio::test]
async fn a_vertical_version_is_added_after_the_task_sequence() {
    let bed = Bed::open().await;
    bed.services.sampler.set_default_duration(60);
    let before = bed.document().await.sequences.len();
    let answer = bed
        .json("auto_reframe", json!({ "target_format": "9:16" }))
        .await;
    assert_eq!(answer["finished"], true, "{answer}");
    let project = bed.document().await;
    assert_eq!(project.sequences.len(), before + 1);
    let version = &project.sequences[1];
    assert_eq!(
        answer["version_sequence_id"].as_str(),
        Some(version.id.to_string().as_str())
    );
    assert_eq!((version.format.width, version.format.height), (1080, 1920));
    assert_eq!(
        version.format.frame_rate,
        project.sequences[0].format.frame_rate
    );
    assert!(version.name.ends_with("9:16"), "{}", version.name);
    assert!(all_tracked(&project, 1));
}

#[tokio::test]
async fn a_call_that_does_not_wait_answers_unfinished() {
    let bed = Bed::open().await;
    bed.services.sampler.hold();
    let answer = bed.json("auto_reframe", json!({ "wait_seconds": 0 })).await;
    assert_eq!(answer["finished"], false, "{answer}");
    assert_eq!(answer["analysing"].as_array().map(Vec::len), Some(1));
    assert!(bed.document().await.items().all(|item| match &item.kind {
        ItemKind::MediaClip(clip) => clip.reframe.tracking().is_none(),
        _ => true,
    }));
    bed.services.sampler.release();
}

#[tokio::test]
async fn refusals_name_what_to_fix() {
    let bed = Bed::open().await;
    assert_eq!(
        bed.error_code("auto_reframe", json!({ "target_format": "21:9" }))
            .await,
        "INVALID_ARGUMENTS"
    );
    assert_eq!(
        bed.error_code("auto_reframe", json!({ "smoothing": 3 }))
            .await,
        "INVALID_ARGUMENTS"
    );
    assert_eq!(
        bed.error_code("auto_reframe", json!({ "item_ids": [] }))
            .await,
        "INVALID_ARGUMENTS"
    );
    bed.services.detector.set_unavailable(Some(
        "Microsoft publishes no ONNX Runtime build for Intel Macs",
    ));
    assert_eq!(
        bed.error_code("auto_reframe", json!({})).await,
        "VISION_UNAVAILABLE"
    );
}
