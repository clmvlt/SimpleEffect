//! Caption tools (prompt 23) against the example project with its interview transcribed:
//! `create_captions`, `edit_caption_text`, `emphasize_caption_words`, `update_caption_style`.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

mod support;

use se_domain::{CaptionItem, ItemKind, Project, test_support::example_ids};
use serde_json::{Value, json};
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
    bed
}

/// The captions of the caption track « Sous-titres ».
fn captions(document: &Project) -> Vec<CaptionItem> {
    document.sequences[0]
        .tracks
        .iter()
        .filter(|track| track.name == "Sous-titres")
        .flat_map(|track| &track.items)
        .filter_map(|item| match &item.kind {
            ItemKind::Caption(caption) => Some(caption.clone()),
            _ => None,
        })
        .collect()
}

async fn create(bed: &Bed, arguments: Value) -> Vec<String> {
    let output = bed.json("create_captions", arguments).await;
    output["captions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|caption| caption["item_id"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn create_captions_groups_the_words_on_a_caption_track() {
    let bed = bed().await;
    let output = bed
        .json("create_captions", json!({"style": "karaoke"}))
        .await;
    let texts: Vec<&str> = output["captions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|caption| caption["text"].as_str().unwrap())
        .collect();
    assert_eq!(texts, ["Bonjour à tous.", "Euh je commence maintenant."]);
    assert!(output["style_id"].is_string());
    assert_eq!(output["captions"][0]["start"]["timecode"], "00:00:00.167");
    let document = bed.document().await;
    assert_eq!(captions(&document).len(), 2);
    let item = bed
        .json(
            "get_item",
            json!({"item_id": output["captions"][1]["item_id"]}),
        )
        .await;
    assert_eq!(item["item"]["origin"], "ai_task");
    assert_eq!(item["caption_style"]["preset"], "karaoke");
    assert_eq!(
        item["properties"]["words"][2]["start"]["sequence_time"]["timecode"],
        "00:00:02.550"
    );
}

#[tokio::test]
async fn create_captions_explains_ranges_without_speech_and_bad_styles() {
    let bed = bed().await;
    let output = bed
        .json(
            "create_captions",
            json!({"range": {"start": 11, "end": 13}}),
        )
        .await;
    assert_eq!(output["captions"], json!([]));
    // The music plays there untranscribed: the note says what to transcribe.
    assert!(
        output["note"]
            .as_str()
            .unwrap()
            .contains("between 00:00:11.000")
    );
    assert_eq!(output["untranscribed"].as_array().unwrap().len(), 1);
    assert_eq!(
        bed.error_code("create_captions", json!({"style": "disco"}))
            .await,
        "INVALID_ARGUMENTS"
    );
    assert_eq!(
        bed.error_code("create_captions", json!({"max_lines": 9}))
            .await,
        "VALIDATION_ERROR"
    );
}

#[tokio::test]
async fn text_fixes_and_emphasis_keep_the_word_timings() {
    let bed = bed().await;
    let ids = create(&bed, json!({"style": "minimal"})).await;
    let before = captions(&bed.document().await)[0].words.clone();
    bed.json(
        "apply_batch",
        json!({"actions": [
            {"tool": "edit_caption_text", "arguments": {"item_id": ids[0], "text": "Bonjour à tous !"}},
            {"tool": "emphasize_caption_words", "arguments": {"item_id": ids[0], "words": [0]}}
        ]}),
    )
    .await;
    let after = captions(&bed.document().await)[0].words.clone();
    let texts: Vec<&str> = after.iter().map(|word| word.text.as_str()).collect();
    assert_eq!(texts, ["Bonjour", "à", "tous", "!"]);
    assert_eq!(
        (after[0].start, after[0].end),
        (before[0].start, before[0].end)
    );
    assert_eq!(
        (after[1].start, after[1].end),
        (before[1].start, before[1].end)
    );
    assert!(after[0].emphasis);
    assert_eq!(
        bed.error_code(
            "edit_caption_text",
            json!({"item_id": ids[0], "text": "  "})
        )
        .await,
        "INVALID_ARGUMENTS"
    );
    assert_eq!(
        bed.error_code(
            "emphasize_caption_words",
            json!({"item_id": ids[0], "words": [12]})
        )
        .await,
        "VALIDATION_ERROR"
    );
}

#[tokio::test]
async fn caption_styles_change_for_some_captions_or_a_whole_style() {
    let bed = bed().await;
    let ids = create(&bed, json!({"style": "minimal"})).await;
    let shared = captions(&bed.document().await)[0].style_id;
    bed.json(
        "update_caption_style",
        json!({"item_ids": [ids[0]], "font_size": 90, "active_color": "#ff0000"}),
    )
    .await;
    let document = bed.document().await;
    let styles: Vec<_> = captions(&document)
        .iter()
        .map(|caption| caption.style_id)
        .collect();
    assert_ne!(styles[0], shared, "a copy for the first caption");
    assert_eq!(styles[1], shared);
    bed.json(
        "update_caption_style",
        json!({"style_id": shared.to_string(), "animation": "pop_per_word", "outline_width": 5}),
    )
    .await;
    let document = bed.document().await;
    let style = document
        .caption_styles
        .iter()
        .find(|style| style.id == shared)
        .unwrap();
    assert_eq!(style.animation, se_domain::CaptionAnimation::PopPerWord);
    assert!(style.outline.enabled);
    bed.json(
        "update_caption_style",
        json!({"item_ids": ids, "preset": "box", "max_lines": 1}),
    )
    .await;
    let document = bed.document().await;
    let styles: Vec<_> = captions(&document)
        .iter()
        .map(|caption| caption.style_id)
        .collect();
    assert_eq!(styles[0], styles[1]);
    let boxed = document
        .caption_styles
        .iter()
        .find(|style| style.id == styles[0])
        .unwrap();
    assert_eq!(boxed.preset.as_deref(), Some("box"));
    assert_eq!(boxed.max_lines, 1);
    assert_eq!(
        bed.error_code("update_caption_style", json!({"font_size": 50}))
            .await,
        "INVALID_ARGUMENTS"
    );
}
