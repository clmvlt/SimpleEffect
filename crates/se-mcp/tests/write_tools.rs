//! Write tools against the example project opened over in-memory ports: what they create and
//! change, the codes of their refusals, `apply_batch` (references, atomicity), the task's changes,
//! and the replay of a recorded agent session reverted by `RevertTask`.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

mod support;

use se_domain::{ItemId, Project, TimeRange, TrackPatch};
use serde_json::{Value, json};
use support::{Bed, first_json};

/// Items of the example project (`se_domain::test_support::example_project`).
const TITLE: u64 = 3; // text on V2, 1–4 s
const SHAPE: u64 = 6; // shape on V3, 6–8 s
const MUSIC: u64 = 9; // A2, 0–14 s

fn item(index: u64) -> String {
    ItemId::from_test_index(index).to_string()
}

fn seconds(range: &[f64; 2]) -> TimeRange {
    let at = |seconds: f64| se_domain::Flicks::from_seconds_f64(seconds).unwrap();
    TimeRange::from_start_end(at(range[0]), at(range[1])).unwrap()
}

fn without_revisions(mut project: Project) -> Project {
    project.revision = 0;
    for track in project
        .sequences
        .iter_mut()
        .flat_map(|sequence| &mut sequence.tracks)
    {
        track.items.iter_mut().for_each(|item| item.revision = 0);
    }
    project
}

async fn error_message(bed: &Bed, tool: &str, arguments: Value) -> (String, String) {
    let result = bed.call(tool, arguments).await;
    assert_eq!(result.is_error, Some(true), "{tool} succeeded");
    let error = first_json(&result)["error"].clone();
    (
        error["code"].as_str().unwrap().to_owned(),
        error["message"].as_str().unwrap().to_owned(),
    )
}

#[tokio::test]
async fn add_text_creates_an_animated_text_owned_by_the_task() {
    let bed = Bed::open().await;
    let output = bed
        .json(
            "add_text",
            json!({"text": "Abonne-toi", "start": 11, "duration": 2, "font_size": 120,
                   "color": "#ffdd00", "stroke": {}, "animation_in": {"preset": "pop"},
                   "animation_out": {"preset": "fade", "duration": 0.3}}),
        )
        .await;
    let created = output["created"][0].as_str().unwrap().to_owned();
    let summary = &output["items"][0];
    assert_eq!(summary["item_id"], created);
    assert_eq!(summary["kind"], "text");
    assert_eq!(summary["origin"], "ai_task");
    assert_eq!(summary["start"]["timecode"], "00:00:11.000");
    assert_eq!(summary["end"]["timecode"], "00:00:13.000");
    let animated: Vec<&str> = summary["animated"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| name.as_str().unwrap())
        .collect();
    assert!(
        animated.contains(&"scale") && animated.contains(&"opacity"),
        "{animated:?}"
    );
    let detail = bed.json("get_item", json!({"item_id": created})).await;
    let style = &detail["properties"]["style"];
    assert_eq!(style["color"]["value"], "#ffdd00");
    assert_eq!(style["stroke"]["enabled"], true);
    assert_eq!(style["fontSize"]["value"], 120.0);
}

#[tokio::test]
async fn refusals_carry_the_write_codes() {
    let mut bed = Bed::open().await;
    bed.agent.scope.range = Some(seconds(&[1.0, 9.0]));
    let (code, message) = error_message(
        &bed,
        "add_text",
        json!({"text": "Late", "start": 8, "end": 10}),
    )
    .await;
    assert_eq!(code, "OUT_OF_SCOPE");
    assert!(message.contains("00:00:01.000–00:00:09.000"), "{message}");
    let (code, _) = error_message(
        &bed,
        "update_item",
        json!({"item_id": item(TITLE), "text": "Hi", "expected_revision": 999}),
    )
    .await;
    assert_eq!(code, "REVISION_CONFLICT");
    let (code, message) = error_message(
        &bed,
        "move_item",
        json!({"item_id": item(SHAPE), "start": 7.5}),
    )
    .await;
    assert_eq!(code, "VALIDATION_ERROR");
    assert!(message.contains("OVERLAPPING_ITEMS"), "{message}");
    let (code, _) =
        error_message(&bed, "update_item", json!({"item_id": "nope", "text": "x"})).await;
    assert_eq!(code, "ITEM_NOT_FOUND");
    let (code, message) = error_message(
        &bed,
        "update_item",
        json!({"item_id": item(TITLE), "speed": 2.0}),
    )
    .await;
    assert_eq!(code, "INVALID_ARGUMENTS");
    assert!(message.contains("a text has no speed"), "{message}");
    let v3 = bed.project.sequences[0].tracks[2].id;
    bed.services
        .projects
        .update_track(
            bed.project_id(),
            v3,
            TrackPatch {
                locked: Some(true),
                ..TrackPatch::default()
            },
        )
        .await
        .unwrap();
    let (code, _) = error_message(
        &bed,
        "move_item",
        json!({"item_id": item(SHAPE), "start": 5.5}),
    )
    .await;
    assert_eq!(code, "LOCKED_TRACK");
}

#[tokio::test]
async fn timing_tools_cut_trim_move_and_delete() {
    let bed = Bed::open().await;
    let split = bed
        .json("split_at", json!({"at": 7, "item_ids": [item(MUSIC)]}))
        .await;
    let right = split["created"][0].as_str().unwrap().to_owned();
    assert_eq!(split["items"].as_array().unwrap().len(), 2);
    let faded = bed
        .json(
            "add_transition",
            json!({"track": "A2", "at": "00:00:07.000", "duration": 0.4}),
        )
        .await;
    assert_eq!(
        faded["revision"].as_u64().unwrap(),
        split["revision"].as_u64().unwrap() + 1
    );
    let trimmed = bed
        .json(
            "trim_item",
            json!({"item_id": item(SHAPE), "edge": "end", "to": 7.5}),
        )
        .await;
    assert_eq!(trimmed["items"][0]["end"]["timecode"], "00:00:07.500");
    let moved = bed
        .json(
            "move_item",
            json!({"item_id": item(SHAPE), "start": 5.2, "track": "V3"}),
        )
        .await;
    assert_eq!(moved["items"][0]["start"]["timecode"], "00:00:05.200");
    let volume = bed
        .json(
            "set_volume",
            json!({"item_id": right, "automation": [{"time": 0, "gain_db": 0}, {"time": 2, "gain_db": -12, "easing": "ease_in_out"}],
                   "fade_out": 0.05}),
        )
        .await;
    assert_eq!(volume["items"][0]["animated"][0], "volumeDb");
    let deleted = bed
        .json("delete_items", json!({"item_ids": [item(SHAPE)]}))
        .await;
    assert_eq!(deleted["removed"][0], item(SHAPE));
    let marker = bed
        .json(
            "add_marker",
            json!({"time": 3, "label": "Beat", "color": "blue"}),
        )
        .await;
    assert_eq!(marker["created"], json!([]));
    let gap = bed
        .json("insert_gap", json!({"at": 14, "duration": 1}))
        .await;
    assert!(gap["revision"].as_u64().is_some());
    let ripple = bed
        .json(
            "ripple_delete_range",
            json!({"range": {"start": 14, "end": 15}}),
        )
        .await;
    assert!(ripple["revision"].as_u64().unwrap() > gap["revision"].as_u64().unwrap());
}

#[tokio::test]
async fn keyframes_and_presets_animate_items() {
    let bed = Bed::open().await;
    let output = bed
        .json(
            "set_keyframes",
            json!({"item_id": item(TITLE), "property": "position",
                   "keyframes": [{"time": 0, "value": [0.5, 0.9], "easing": "ease_out"},
                                 {"time": 0.5, "value": {"x": 0.5, "y": 0.8}}]}),
        )
        .await;
    assert!(
        output["items"][0]["animated"]
            .as_array()
            .unwrap()
            .contains(&json!("position"))
    );
    let (code, message) = error_message(
        &bed,
        "set_keyframes",
        json!({"item_id": item(TITLE), "property": "opacity", "value": "#ffffff"}),
    )
    .await;
    assert_eq!(code, "INVALID_ARGUMENTS");
    assert!(
        message.contains("values of opacity are numbers"),
        "{message}"
    );
    let preset = bed
        .json(
            "apply_animation_preset",
            json!({"item_id": item(TITLE), "preset": "fade", "phase": "out", "duration": 0.5}),
        )
        .await;
    assert!(
        preset["items"][0]["animated"]
            .as_array()
            .unwrap()
            .contains(&json!("opacity"))
    );
}

#[tokio::test]
async fn batches_are_atomic_and_refer_to_their_own_items() {
    let bed = Bed::open().await;
    let output = bed
        .json(
            "apply_batch",
            json!({"actions": [
                {"tool": "add_shape", "as": "bar", "arguments": {"shape": "rounded_rectangle", "start": 11, "end": 13, "position": "bottom", "width": 900, "height": 160, "color": "#000000aa"}},
                {"tool": "add_text", "as": "title", "arguments": {"text": "Merci", "start": 11, "end": 13, "position": "bottom"}},
                {"tool": "apply_animation_preset", "arguments": {"item_id": "$title", "preset": "pop", "phase": "in"}},
                {"tool": "update_item", "arguments": {"item_id": "$bar", "opacity": 0.8}}
            ]}),
        )
        .await;
    let actions = output["actions"].as_array().unwrap();
    assert_eq!(actions.len(), 4);
    assert_eq!(actions[0]["created"].as_array().unwrap().len(), 1);
    assert_eq!(actions[2]["created"], json!([]));
    let revision = output["revision"].as_u64().unwrap();
    let before = bed.document().await;
    let (code, message) = error_message(
        &bed,
        "apply_batch",
        json!({"actions": [
            {"tool": "add_text", "arguments": {"text": "A", "start": 0, "end": 1}},
            {"tool": "move_item", "arguments": {"item_id": item(SHAPE), "start": 7.5}}
        ]}),
    )
    .await;
    assert_eq!(code, "VALIDATION_ERROR");
    assert!(message.starts_with("action 2 (move_item)"), "{message}");
    assert_eq!(
        bed.document().await,
        before,
        "nothing of the batch was applied"
    );
    assert_eq!(before.revision, revision);
    let (code, message) = error_message(
        &bed,
        "apply_batch",
        json!({"actions": [{"tool": "add_marker", "as": "m", "arguments": {"time": 1}}]}),
    )
    .await;
    assert_eq!(code, "INVALID_ARGUMENTS");
    assert!(message.contains("creates none"), "{message}");
    let (code, _) = error_message(
        &bed,
        "apply_batch",
        json!({"actions": [{"tool": "get_timeline", "arguments": {}}]}),
    )
    .await;
    assert_eq!(code, "INVALID_ARGUMENTS");
}

#[tokio::test]
async fn the_task_sees_its_changes() {
    let mut bed = Bed::open().await;
    bed.run_task().await;
    let empty = bed.json("get_task_changes", json!({})).await;
    assert_eq!(empty["changeset_id"], Value::Null);
    let created = bed
        .json("add_text", json!({"text": "Hi", "start": 11, "end": 12}))
        .await["created"][0]
        .clone();
    bed.json("move_item", json!({"item_id": item(SHAPE), "start": 5.5}))
        .await;
    let changes = bed.json("get_task_changes", json!({})).await;
    assert_eq!(changes["edits"], 2);
    assert_eq!(changes["created"][0]["item_id"], created);
    assert_eq!(changes["changed"][0]["item_id"], item(SHAPE));
    assert!(changes["changeset_id"].is_string());
}

/// Replays `fixtures/subscribe-pop.json` (an agent's calls for « De 51 s à 54 s affiche le texte
/// “Abonne-toi” en gros au centre avec une animation pop »): the expected item, then `RevertTask`
/// gives back the initial project.
#[tokio::test]
async fn a_recorded_session_replays_and_reverts() {
    let mut bed = Bed::open().await;
    bed.agent.scope.range = Some(seconds(&[51.0, 54.0]));
    bed.run_task().await;
    let initial = bed.document().await;
    let fixture: Value = serde_json::from_str(include_str!("fixtures/subscribe-pop.json")).unwrap();
    let mut created: Vec<String> = Vec::new();
    for call in fixture["calls"].as_array().unwrap() {
        let mut arguments = call["arguments"].to_string();
        for (index, id) in created.iter().enumerate() {
            arguments = arguments.replace(&format!("{{created.{index}}}"), id);
        }
        let output = bed
            .json(
                call["tool"].as_str().unwrap(),
                serde_json::from_str(&arguments).unwrap(),
            )
            .await;
        created.push(
            output["created"][0]
                .as_str()
                .map_or_else(String::new, str::to_owned),
        );
    }
    let mut detail = bed.json("get_item", json!({"item_id": created[0]})).await;
    for id in ["item_id", "track_id", "task_id"] {
        detail["item"][id] = json!(format!("[{id}]"));
    }
    insta::assert_json_snapshot!("subscribe_pop_item", detail);
    let report = bed
        .services
        .tasks
        .revert(bed.project_id(), bed.agent.task_id)
        .await
        .unwrap();
    assert!(report.reverted);
    assert_eq!(
        without_revisions(bed.document().await),
        without_revisions(initial)
    );
}
