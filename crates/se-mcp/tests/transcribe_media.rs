//! `transcribe_media` against the example project: refused without a speech model, then the
//! sequence's sound media transcribed (and readable with `get_transcript`), nothing started twice,
//! a call that does not wait followed by one that does, and explicit assets.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

mod support;

use std::path::Path;

use se_app::{
    test_support::{FakeModelFile, TESTBED_MODELS_DIR},
    transcription::{MODELS, VOICE_MODEL},
};
use se_domain::test_support::example_ids;
use serde_json::json;
use support::Bed;

/// Puts catalogue model `id` (and the voice model) in the fake models folder, as if downloaded.
async fn install_model(bed: &Bed, id: &str) {
    let model = MODELS.iter().find(|model| model.id == id).unwrap();
    for file in [model.file, VOICE_MODEL] {
        bed.services.model_files.put(
            &Path::new(TESTBED_MODELS_DIR).join(file.file_name),
            FakeModelFile {
                size: file.size,
                sha256: file.sha256.to_owned(),
                is_model: true,
            },
        );
    }
    bed.services.transcription.status().await.unwrap();
}

#[tokio::test]
async fn without_a_model_the_agent_is_told_to_ask_for_one() {
    let bed = Bed::open().await;
    bed.services.transcription.start().await.unwrap();
    assert_eq!(
        bed.error_code("transcribe_media", json!({})).await,
        "NO_TRANSCRIPTION_MODEL"
    );
}

#[tokio::test]
async fn the_sequences_sound_media_are_transcribed_and_readable() {
    let bed = Bed::open().await;
    bed.services.transcription.start().await.unwrap();
    install_model(&bed, "base").await;

    let answer = bed.json("transcribe_media", json!({})).await;
    assert_eq!(answer["finished"], true);
    assert_eq!(answer["started"], 2, "the interview and the music");
    let statuses: Vec<&str> = answer["assets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|asset| asset["status"].as_str().unwrap())
        .collect();
    assert_eq!(statuses, ["transcribed", "transcribed"]);
    assert_eq!(answer["assets"][0]["word_count"], 5);
    assert!(answer["note"].as_str().unwrap().starts_with("done"));

    let transcript = bed.json("get_transcript", json!({})).await;
    assert_eq!(transcript["available"], true);
    assert_eq!(transcript["untranscribed"], json!([]));

    let again = bed.json("transcribe_media", json!({})).await;
    assert_eq!(again["started"], 0, "nothing is transcribed twice");
    assert_eq!(bed.services.transcriber.requests().len(), 2);
}

#[tokio::test]
async fn a_call_that_does_not_wait_is_followed_by_one_that_does() {
    let bed = Bed::open().await;
    bed.services.transcription.start().await.unwrap();
    install_model(&bed, "base").await;
    let video = example_ids::VIDEO_ASSET.to_string();
    bed.services.transcriber.hold();

    let started = bed
        .json(
            "transcribe_media",
            json!({ "asset_ids": [video], "wait_seconds": 0 }),
        )
        .await;
    assert_eq!(started["started"], 1);
    assert_eq!(started["finished"], false);
    assert_eq!(started["assets"][0]["status"], "running");
    assert!(
        started["note"]
            .as_str()
            .unwrap()
            .starts_with("still transcribing")
    );

    bed.services.transcriber.release();
    let waited = bed
        .json("transcribe_media", json!({ "asset_ids": [video] }))
        .await;
    assert_eq!(waited["started"], 0, "the running transcription is awaited");
    assert_eq!(waited["finished"], true);
    assert_eq!(waited["assets"][0]["status"], "transcribed");
}

#[tokio::test]
async fn explicit_assets_are_checked() {
    let bed = Bed::open().await;
    bed.services.transcription.start().await.unwrap();
    install_model(&bed, "base").await;
    assert_eq!(
        bed.error_code("transcribe_media", json!({ "asset_ids": ["nope"] }))
            .await,
        "ASSET_NOT_FOUND"
    );
    assert_eq!(
        bed.error_code("transcribe_media", json!({ "wait_seconds": -1 }))
            .await,
        "INVALID_ARGUMENTS"
    );
    let image = example_ids::IMAGE_ASSET.to_string();
    let answer = bed
        .json("transcribe_media", json!({ "asset_ids": [image] }))
        .await;
    assert_eq!(answer["started"], 0);
    assert_eq!(answer["assets"][0]["status"], "unavailable");
    assert!(
        answer["assets"][0]["reason"]
            .as_str()
            .unwrap()
            .starts_with("no_sound")
    );
    assert!(answer["note"].as_str().unwrap().starts_with("some media"));
}
