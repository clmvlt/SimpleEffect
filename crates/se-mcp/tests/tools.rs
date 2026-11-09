//! Every read tool against the example project opened over in-memory ports: outputs agents rely
//! on (ids, times, pagination), analyses brought back to sequence time, frames through a fake
//! renderer, and actionable errors.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

mod support;

use std::time::Duration;

use rmcp::model::ContentBlock;
use se_domain::test_support::example_ids;
use serde_json::json;
use support::{Bed, first_json, range_codes, texts};

#[tokio::test]
async fn the_overview_shows_sequences_tracks_media_and_scope() {
    let bed = Bed::open().await;
    let overview = bed.json("get_project_overview", json!({})).await;
    assert_eq!(overview["project"]["name"], "Example");
    let sequence = &overview["sequences"][0];
    assert_eq!(sequence["name"], "Main edit");
    assert_eq!(sequence["width"], 1920);
    assert_eq!(sequence["fps"], 29.97);
    assert_eq!(sequence["duration"]["timecode"], "00:00:14.000");
    assert_eq!(sequence["item_count"], 9);
    assert_eq!(sequence["is_task_sequence"], true);
    let tracks: Vec<&str> = overview["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|track| track["name"].as_str().unwrap())
        .collect();
    assert_eq!(tracks, ["V1", "V2", "V3", "A1", "A2"]);
    assert_eq!(overview["assets"]["total"], 3);
    assert_eq!(overview["brand_kit"], true);
    assert_eq!(overview["scope"]["range"], serde_json::Value::Null);
    assert_eq!(overview["scope"]["sequence_name"], "Main edit");
}

#[tokio::test]
async fn the_timeline_lists_items_in_time_order_with_pages() {
    let bed = Bed::open().await;
    let timeline = bed.json("get_timeline", json!({})).await;
    assert_eq!(timeline["total_items"], 9);
    assert_eq!(timeline["range"]["end"]["timecode"], "00:00:14.000");
    let first = &timeline["items"][0];
    assert_eq!(first["kind"], "media_clip");
    assert_eq!(first["label"], "interview.mp4");
    assert_eq!(first["source"]["start"]["timecode"], "00:00:05.000");
    assert_eq!(timeline["transitions"][0]["at"]["timecode"], "00:00:10.000");
    assert_eq!(timeline["markers"][0]["label"], "Intro ends");
    assert_eq!(timeline["next_offset"], serde_json::Value::Null);

    let page = bed.json("get_timeline", json!({ "limit": 2 })).await;
    assert_eq!(page["items"].as_array().unwrap().len(), 2);
    assert_eq!(page["next_offset"], 2);
    let next = bed
        .json("get_timeline", json!({ "limit": 2, "offset": 8 }))
        .await;
    assert_eq!(next["items"].as_array().unwrap().len(), 1);

    let logo = bed
        .json(
            "get_timeline",
            json!({ "range": { "start": "00:05", "end": 7 }, "tracks": ["V2"] }),
        )
        .await;
    assert_eq!(logo["total_items"], 1);
    assert_eq!(logo["items"][0]["kind"], "image");
    assert_eq!(logo["items"][0]["label"], "logo.png");
    assert_eq!(logo["markers"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn timeline_errors_say_what_exists() {
    let bed = Bed::open().await;
    assert_eq!(
        bed.error_code("get_timeline", json!({ "tracks": ["V9"] }))
            .await,
        "TRACK_NOT_FOUND"
    );
    assert_eq!(
        bed.error_code("get_timeline", json!({ "sequence": "Other" }))
            .await,
        "SEQUENCE_NOT_FOUND"
    );
    assert_eq!(
        bed.error_code("get_timeline", json!({ "range": { "start": 5, "end": 2 } }))
            .await,
        "INVALID_TIME"
    );
    assert_eq!(
        bed.error_code("get_timeline", json!({ "trackz": [] }))
            .await,
        "INVALID_ARGUMENTS"
    );
}

#[tokio::test]
async fn an_item_shows_its_whole_document_form() {
    let bed = Bed::open().await;
    let text_id = bed.project.sequences[0].tracks[1].items[0].id.to_string();
    let detail = bed.json("get_item", json!({ "item_id": text_id })).await;
    assert_eq!(detail["track_name"], "V2");
    assert_eq!(detail["item"]["label"], "Bonjour");
    assert_eq!(detail["item"]["animated"], json!(["opacity"]));
    let properties = &detail["properties"];
    assert_eq!(properties["content"], "Bonjour");
    let keyframe = &properties["transform"]["opacity"]["keyframes"][1];
    assert_eq!(keyframe["time"]["timecode"], "00:00:00.500");
    assert_eq!(
        keyframe["time"]["sequence_time"]["timecode"],
        "00:00:01.500"
    );
    assert_eq!(
        bed.error_code("get_item", json!({ "item_id": "nope" }))
            .await,
        "ITEM_NOT_FOUND"
    );
}

#[tokio::test]
async fn assets_are_filtered_by_kind_and_name() {
    let bed = Bed::open().await;
    let all = bed.json("list_assets", json!({})).await;
    assert_eq!(all["total"], 3);
    let audio = bed.json("list_assets", json!({ "kind": "audio" })).await;
    assert_eq!(audio["assets"][0]["name"], "music.wav");
    assert_eq!(audio["assets"][0]["used_by"], 1);
    let logo = bed.json("list_assets", json!({ "query": "LOGO" })).await;
    assert_eq!(logo["total"], 1);
    assert_eq!(logo["assets"][0]["kind"], "image");
}

#[tokio::test]
async fn audio_analyses_come_in_source_time_or_in_sequence_time() {
    let bed = Bed::open().await;
    bed.set_audio_analysis(
        example_ids::VIDEO_ASSET,
        &[(7_000, 8_000), (12_000, 13_000)],
    );
    bed.set_audio_analysis(example_ids::AUDIO_ASSET, &[(2_000, 3_500)]);
    let asset = bed
        .json(
            "get_audio_analysis",
            json!({ "asset": example_ids::VIDEO_ASSET.to_string(), "max_points": 4 }),
        )
        .await;
    assert_eq!(asset["time_base"], "source");
    assert_eq!(asset["integrated_lufs"], -15.9);
    assert_eq!(asset["silence_total"], 2);
    assert_eq!(asset["loudness"].as_array().unwrap().len(), 4);
    assert_eq!(asset["loudest"][0]["lufs"], -21.0);

    let sequence = bed.json("get_audio_analysis", json!({})).await;
    assert_eq!(sequence["time_base"], "sequence");
    let interview = &sequence["clips"][0];
    assert_eq!(interview["name"], "interview.mp4");
    assert_eq!(
        range_codes(&interview["silences"]),
        [
            ("00:00:02.000".to_owned(), "00:00:03.000".to_owned()),
            ("00:00:07.000".to_owned(), "00:00:08.000".to_owned())
        ]
    );
    assert_eq!(
        range_codes(&sequence["silences"]),
        [("00:00:02.000".to_owned(), "00:00:03.000".to_owned())]
    );
    assert_eq!(sequence["unanalysed"], json!([]));
}

#[tokio::test]
async fn missing_analyses_are_reported_as_such() {
    let bed = Bed::open().await;
    assert_eq!(
        bed.error_code(
            "get_scene_changes",
            json!({ "asset": example_ids::IMAGE_ASSET.to_string() })
        )
        .await,
        "NOT_ANALYSED"
    );
}

#[tokio::test]
async fn scene_changes_follow_each_clip_into_the_sequence() {
    let bed = Bed::open().await;
    bed.set_scenes(example_ids::VIDEO_ASSET, &[7_000, 22_000, 40_000]);
    let asset = bed
        .json(
            "get_scene_changes",
            json!({ "asset": example_ids::VIDEO_ASSET.to_string() }),
        )
        .await;
    assert_eq!(asset["total"], 3);
    let sequence = bed.json("get_scene_changes", json!({})).await;
    let times: Vec<&str> = sequence["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|change| change["time"]["timecode"].as_str().unwrap())
        .collect();
    assert_eq!(times, ["00:00:02.000", "00:00:11.000"]);
    assert_eq!(sequence["cuts"][0]["timecode"], "00:00:10.000");
}

#[tokio::test]
async fn source_frames_come_back_as_images() {
    let bed = Bed::open().await;
    let result = bed
        .call(
            "get_source_frames",
            json!({ "asset": example_ids::VIDEO_ASSET.to_string(), "times": [1, "00:00:02.500"] }),
        )
        .await;
    assert_ne!(result.is_error, Some(true), "{:?}", texts(&result));
    let header = first_json(&result);
    assert_eq!(header["time_base"], "source");
    assert_eq!(header["frames"][1]["timecode"], "00:00:02.500");
    let images = result
        .content
        .iter()
        .filter(|block| matches!(block, ContentBlock::Image(_)))
        .count();
    assert_eq!(images, 2);
    assert_eq!(
        bed.error_code(
            "get_source_frames",
            json!({ "asset": example_ids::AUDIO_ASSET.to_string(), "times": [1] })
        )
        .await,
        "INVALID_ARGUMENTS"
    );
}

#[tokio::test]
async fn sequence_frames_are_drawn_by_the_renderer() {
    let bed = Bed::open().await;
    let calling = tokio::spawn({
        let server = bed.server.clone();
        let agent = bed.agent.clone();
        async move {
            let arguments = json!({ "range": { "start": 0, "end": 14 }, "count": 4 });
            let serde_json::Value::Object(arguments) = arguments else {
                unreachable!()
            };
            server.call(agent, "get_frames", arguments).await.unwrap()
        }
    });
    let request = loop {
        if let Some(request) = bed.services.renderer.requests().pop() {
            break request;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    };
    assert_eq!((request.width, request.height), (768, 432));
    assert_eq!(request.times.len(), 4);
    for index in 0..4 {
        bed.services
            .renders
            .deliver(&request.request_id, index, vec![0xFF, 0xD8, 0xFF, 0xE0])
            .unwrap();
    }
    let result = calling.await.unwrap();
    let header = first_json(&result);
    let times: Vec<&str> = header["frames"]
        .as_array()
        .unwrap()
        .iter()
        .map(|time| time["timecode"].as_str().unwrap())
        .collect();
    assert_eq!(
        times,
        [
            "00:00:01.750",
            "00:00:05.250",
            "00:00:08.750",
            "00:00:12.250"
        ]
    );
    let images = result
        .content
        .iter()
        .filter(|block| matches!(block, ContentBlock::Image(_)))
        .count();
    assert_eq!(images, 4);
    assert!(
        texts(&result)
            .iter()
            .any(|text| text == "Frame 2 at 00:00:05.250")
    );
}

#[tokio::test]
async fn frame_requests_are_checked_before_reaching_the_renderer() {
    let bed = Bed::open().await;
    assert_eq!(
        bed.error_code("get_frames", json!({ "times": [20] })).await,
        "INVALID_TIME"
    );
    assert_eq!(
        bed.error_code("get_frames", json!({ "count": 13 })).await,
        "TOO_MANY_FRAMES"
    );
    assert_eq!(
        bed.error_code(
            "get_frames",
            json!({ "times": [1], "range": { "start": 0, "end": 1 } })
        )
        .await,
        "INVALID_ARGUMENTS"
    );
    assert_eq!(bed.services.renderer.requests(), Vec::new());
    bed.services.renderer.set_connected(false);
    assert_eq!(
        bed.error_code("get_frames", json!({ "times": [1] })).await,
        "RENDERER_UNAVAILABLE"
    );
}

#[tokio::test]
async fn the_brand_kit_stub_templates_and_presets_answer_plainly() {
    let bed = Bed::open().await;
    assert_eq!(
        bed.json("get_brand_kit", json!({})).await["available"],
        false
    );
    let templates = bed.json("list_templates", json!({})).await;
    assert_eq!(templates["templates"][0]["name"], "Lower third");
    assert_eq!(templates["templates"][0]["instances"], 1);
    let presets = bed.json("list_animation_presets", json!({})).await;
    let fade = presets["presets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|preset| preset["preset_id"] == "fade")
        .expect("the fade preset");
    assert_eq!(fade["phases"], json!(["in", "out"]));
    assert!(fade["default_duration"]["seconds"].as_f64().unwrap() > 0.0);
}

#[tokio::test]
async fn a_closed_project_and_an_unknown_tool_are_refused() {
    let bed = Bed::open().await;
    let unknown = bed
        .server
        .call(
            bed.agent.clone(),
            "delete_everything",
            serde_json::Map::new(),
        )
        .await;
    assert!(unknown.is_err());
    bed.services
        .projects
        .close_project(bed.project_id())
        .await
        .unwrap();
    assert_eq!(
        bed.error_code("get_project_overview", json!({})).await,
        "PROJECT_NOT_OPEN"
    );
}

/// The interview's words in source time: its sound clip (A1) plays source 5–15 s at 0–10 s.
const INTERVIEW_WORDS: [(&str, i64, i64, u32); 8] = [
    ("Bonjour", 5_500, 5_900, 0),
    ("à", 5_900, 6_000, 0),
    ("tous.", 6_000, 6_400, 0),
    ("Abonnez-vous", 8_000, 8_800, 1),
    ("à", 8_800, 8_900, 1),
    ("la", 8_900, 9_000, 1),
    ("chaîne !", 9_000, 9_600, 1),
    ("Coupé", 16_000, 16_500, 2),
];

#[tokio::test]
async fn get_transcript_gives_the_sequences_words_in_sequence_time() {
    let bed = Bed::open().await;
    let empty = bed.json("get_transcript", json!({})).await;
    assert_eq!(empty["available"], false);
    assert_eq!(
        empty["untranscribed"].as_array().unwrap().len(),
        2,
        "interview and music"
    );

    bed.set_transcript(example_ids::VIDEO_ASSET, &INTERVIEW_WORDS);
    let answer = bed.json("get_transcript", json!({})).await;
    assert_eq!(answer["available"], true);
    assert_eq!(
        answer["total_words"], 7,
        "the word beyond the clip is not played"
    );
    let segments = answer["segments"].as_array().unwrap();
    assert_eq!(segments[0]["text"], "Bonjour à tous.");
    assert_eq!(segments[0]["start"]["seconds"], 0.5);
    assert_eq!(segments[1]["words"][3]["text"], "chaîne !");
    assert_eq!(segments[1]["words"][3]["unsure"], true);
    assert_eq!(segments[1]["words"][0]["start_seconds"], 3.0);
    assert!(segments[0]["words"][0].get("unsure").is_none());
    assert_eq!(answer["untranscribed"][0]["name"], "music.wav");

    let page = bed
        .json(
            "get_transcript",
            json!({ "range": { "start": 0, "end": 10 }, "limit": 2, "words": false }),
        )
        .await;
    assert_eq!(page["next_offset"], 2);
    assert!(page["segments"][0].get("words").is_none());
    let late = bed
        .json(
            "get_transcript",
            json!({ "range": { "start": 2.5, "end": 4 } }),
        )
        .await;
    assert_eq!(
        late["total_words"], 3,
        "« chaîne ! » has its middle at 4.3 s"
    );
}

#[tokio::test]
async fn search_transcript_finds_where_words_are_said() {
    let bed = Bed::open().await;
    bed.set_transcript(example_ids::VIDEO_ASSET, &INTERVIEW_WORDS);
    let answer = bed
        .json("search_transcript", json!({ "query": "chaine abonn" }))
        .await;
    let results = answer["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["text"], "Abonnez-vous à la chaîne !");
    assert_eq!(results[0]["start"]["seconds"], 3.0);
    let matches: Vec<&str> = results[0]["matches"]
        .as_array()
        .unwrap()
        .iter()
        .map(|word| word["text"].as_str().unwrap())
        .collect();
    assert_eq!(matches, ["Abonnez-vous", "chaîne !"]);
    let cut = bed
        .json("search_transcript", json!({ "query": "coupé" }))
        .await;
    assert_eq!(cut["results"], json!([]));
    assert_eq!(
        cut["elsewhere"], 1,
        "said in the media, not in the sequence"
    );
    assert_eq!(
        bed.error_code("search_transcript", json!({ "query": " ! " }))
            .await,
        "INVALID_ARGUMENTS"
    );
}
