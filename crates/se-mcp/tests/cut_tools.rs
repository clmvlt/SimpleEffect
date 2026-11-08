//! Smart-cut tools (prompt 22) against the example project: words, fillers and silences cut as
//! one transaction on the speech's tracks and their linked picture, inside the task's scope.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

mod support;

use se_domain::{TimeRange, test_support::example_ids};
use serde_json::json;
use support::Bed;

/// The interview's words in source time (its clip plays source 5–15 s at 0–10 s).
const WORDS: [(&str, i64, i64, u32); 7] = [
    ("Bonjour", 5200, 5600, 0),
    ("à", 5650, 5700, 0),
    ("tous.", 5750, 6100, 0),
    ("Euh", 7000, 7300, 1),
    ("je", 7400, 7500, 1),
    ("commence", 7550, 8100, 1),
    ("maintenant.", 8200, 8900, 1),
];

async fn bed() -> Bed {
    let bed = Bed::open().await;
    bed.set_transcript(example_ids::VIDEO_ASSET, &WORDS);
    bed.set_audio_analysis(example_ids::VIDEO_ASSET, &[(7600, 9000)]);
    bed
}

fn seconds(range: &TimeRange) -> (f64, f64) {
    (range.start().to_seconds_f64(), range.end().to_seconds_f64())
}

#[tokio::test]
async fn remove_words_cuts_the_speech_and_its_linked_picture() {
    let bed = bed().await;
    let output = bed
        .json(
            "remove_words",
            json!({"ranges": [{"start": 2.0, "end": 2.3}]}),
        )
        .await;
    assert_eq!(output["cuts"], 1);
    assert_eq!(output["words_removed"], 1);
    assert_eq!(output["tracks"], json!(["V1", "A1"]));
    assert!(output["revision"].is_u64());
    let start = output["ranges"][0]["start"]["seconds"].as_f64().unwrap();
    assert!(
        (start - 1.968).abs() < 0.02,
        "a breath before « Euh »: {start}"
    );
    let document = bed.document().await;
    let v1 = &document.sequences[0].tracks[0].items;
    let a1 = &document.sequences[0].tracks[3].items;
    assert_eq!(
        seconds(&v1[1].timeline_range),
        seconds(&a1[1].timeline_range)
    );
    assert_eq!(v1[1].link_group, a1[1].link_group);
}

#[tokio::test]
async fn remove_fillers_uses_the_lists_or_the_agents_words() {
    let bed = bed().await;
    let output = bed.json("remove_fillers", json!({})).await;
    assert_eq!(output["words_removed"], 1);
    let none = bed.json("remove_fillers", json!({})).await;
    assert_eq!(none["cuts"], 0);
    assert!(none["note"].as_str().unwrap().contains("nothing matched"));
    let own = bed
        .json("remove_fillers", json!({"words": ["maintenant"]}))
        .await;
    assert_eq!(own["words_removed"], 1);
    assert_eq!(
        bed.error_code("remove_fillers", json!({"words": [" "]}))
            .await,
        "INVALID_ARGUMENTS"
    );
}

#[tokio::test]
async fn remove_silences_reads_the_listened_tracks_and_checks_its_arguments() {
    let bed = bed().await;
    let output = bed
        .json("remove_silences", json!({"tracks": ["A1"], "padding": 0.1}))
        .await;
    assert_eq!(output["cuts"], 1);
    let removed = output["removed"]["seconds"].as_f64().unwrap();
    assert!(
        (removed - 1.2).abs() < 0.05,
        "1.4 s minus 2 × 0.1 s: {removed}"
    );
    assert_eq!(
        bed.error_code("remove_silences", json!({"threshold_db": 3}))
            .await,
        "INVALID_ARGUMENTS"
    );
    assert_eq!(
        bed.error_code("remove_silences", json!({"min_silence": 0}))
            .await,
        "INVALID_ARGUMENTS"
    );
}

#[tokio::test]
async fn cuts_stay_in_the_scope_and_never_desync_linked_tracks() {
    let mut bed = bed().await;
    let a1 = bed.project.sequences[0].tracks[3].id;
    bed.agent.scope.tracks = Some(vec![a1]);
    assert_eq!(
        bed.error_code(
            "remove_words",
            json!({"ranges": [{"start": 2.0, "end": 2.3}]})
        )
        .await,
        "OUT_OF_SCOPE"
    );
    bed.agent.scope.tracks = None;
    bed.agent.scope.range = Some(
        TimeRange::from_start_end(
            se_domain::Flicks::from_seconds_f64(0.0).unwrap(),
            se_domain::Flicks::from_seconds_f64(3.0).unwrap(),
        )
        .unwrap(),
    );
    let output = bed.json("remove_silences", json!({"tracks": ["A1"]})).await;
    let end = output["ranges"][0]["end"]["seconds"].as_f64().unwrap();
    assert!(end <= 3.0, "cut up to the scope's end only: {end}");
}
