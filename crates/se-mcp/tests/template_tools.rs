//! Motion template tools (prompt 24) against the example project and a fake renderer answering
//! template checks: `create_motion_template`, `update_motion_template`, `instantiate_template`,
//! `update_template_params`, `list_templates`.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

mod support;

use rmcp::model::ContentBlock;
use se_domain::{
    Animatable, Flicks, ItemId, ItemKind, Project, TemplateItem, TemplateParamValue,
    test_support::example_ids,
};
use se_protocol::{TemplateCheckFrame, TemplateCheckOutcome, TemplateCheckStage, TemplateProblem};
use serde_json::{Value, json};
use support::{Bed, first_json, texts};

/// The example's lower-third instance (V3, 2–3 s).
const INSTANCE: u64 = 5;

const SOURCE: &str = "export const meta = { name: 'Badge', defaultDuration: 3 };\nexport const params = {};\nexport function render(ctx) {}\n";

fn passed(params_json: &str) -> TemplateCheckOutcome {
    let frame = |millis| TemplateCheckFrame {
        time: Flicks::from_millis(millis),
        jpeg_base64: "/9j/4AAQ".to_owned(),
    };
    TemplateCheckOutcome::Passed {
        meta_json: r#"{"name":"Badge","description":"A corner badge","defaultDuration":3}"#
            .to_owned(),
        params_json: params_json.to_owned(),
        frames: vec![frame(400), frame(1500), frame(2600)],
        slowest_frame_ms: 2.5,
    }
}

const BADGE_PARAMS: &str = r##"[
    ["label", {"type": "string", "title": "Texte", "default": "NEW"}],
    ["size", {"type": "number", "default": 60, "minimum": 10, "maximum": 200}],
    ["tint", {"type": "string", "format": "color", "default": "#ff3366"}],
    ["corner", {"type": "string", "enum": ["topLeft", "topRight"], "default": "topRight"}]
]"##;

fn instance(document: &Project, item: &str) -> TemplateItem {
    let id = ItemId::parse(item).unwrap();
    match &document
        .items()
        .find(|candidate| candidate.id == id)
        .unwrap()
        .kind
    {
        ItemKind::MotionTemplate(instance) => instance.clone(),
        other => panic!("{item} is a {}", other.label()),
    }
}

async fn create_badge(bed: &Bed) -> String {
    let result = bed
        .call_with_check(
            "create_motion_template",
            json!({ "source": SOURCE }),
            passed(BADGE_PARAMS),
        )
        .await;
    assert_ne!(result.is_error, Some(true), "{:?}", texts(&result));
    first_json(&result)["template"]["template_id"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn create_motion_template_adds_the_checked_module_with_its_pictures() {
    let bed = Bed::open().await;
    let result = bed
        .call_with_check(
            "create_motion_template",
            json!({ "source": SOURCE }),
            passed(BADGE_PARAMS),
        )
        .await;
    let output = first_json(&result);
    assert_eq!(output["template"]["name"], "Badge");
    assert_eq!(output["template"]["version"], 1);
    assert_eq!(output["template"]["default_duration"]["seconds"], 3.0);
    assert_eq!(output["template"]["params"][1]["type"], "number");
    assert_eq!(output["template"]["params"][1]["maximum"], 200.0);
    assert_eq!(
        output["template"]["params"][3]["options"],
        json!(["topLeft", "topRight"])
    );
    assert_eq!(output["frames"][1]["timecode"], "00:00:01.500");
    let images = result
        .content
        .iter()
        .filter(|block| matches!(block, ContentBlock::Image(_)))
        .count();
    assert_eq!(images, 3);
    assert!(texts(&result)[1].starts_with("Frame 1 (entrance) at 00:00:00.400"));

    let request = bed.services.renderer.template_checks().pop().unwrap();
    assert_eq!(request.source, SOURCE);
    assert_eq!((request.width, request.height), (1920, 1080));

    let listing = bed.json("list_templates", json!({})).await;
    assert_eq!(listing["templates"].as_array().unwrap().len(), 2);
    assert_eq!(listing["templates"][0]["instances"], 1);
    let id = output["template"]["template_id"].as_str().unwrap();
    let detail = bed
        .json("list_templates", json!({ "template_id": id }))
        .await;
    assert_eq!(detail["source"], SOURCE);
}

#[tokio::test]
async fn refused_modules_come_back_with_every_problem() {
    let bed = Bed::open().await;
    let failed = TemplateCheckOutcome::Failed {
        problems: vec![TemplateProblem {
            stage: TemplateCheckStage::Budget,
            message: "the frame took longer than 50 ms (an endless loop?)".to_owned(),
            line: Some(7),
            column: None,
            time: Some(Flicks::from_millis(1500)),
        }],
    };
    let result = bed
        .call_with_check(
            "create_motion_template",
            json!({ "source": SOURCE }),
            failed,
        )
        .await;
    let error = &first_json(&result)["error"];
    assert_eq!(error["code"], "TEMPLATE_INVALID");
    let message = error["message"].as_str().unwrap();
    assert!(
        message.contains("- [budget] line 7, at 1.500 s"),
        "{message}"
    );

    let bad_params = passed(r#"[["size", {"type": "number"}]]"#);
    let result = bed
        .call_with_check(
            "create_motion_template",
            json!({ "source": SOURCE }),
            bad_params,
        )
        .await;
    let message = first_json(&result)["error"]["message"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(
        message.contains("[definition] params.size.default"),
        "{message}"
    );
    assert_eq!(bed.document().await.templates.len(), 1, "nothing was added");
}

#[tokio::test]
async fn instances_get_values_checked_against_the_schema() {
    let bed = Bed::open().await;
    let template = create_badge(&bed).await;
    let output = bed
        .json(
            "instantiate_template",
            json!({
                "template_id": template,
                "start": 12,
                "params": {
                    "label": "PROMO",
                    "size": { "keyframes": [
                        { "time": 0, "value": 20, "easing": "ease_out" },
                        { "time": 0.5, "value": 80 }
                    ] },
                    "tint": "#00cc88",
                    "corner": "topLeft"
                }
            }),
        )
        .await;
    let item = output["created"][0].as_str().unwrap().to_owned();
    assert_eq!(
        output["items"][0]["end"]["seconds"], 15.0,
        "the template's 3 s"
    );
    let document = bed.document().await;
    let values = instance(&document, &item).params;
    assert_eq!(
        values["label"],
        TemplateParamValue::Text {
            value: "PROMO".to_owned()
        }
    );
    assert!(values["size"].is_keyframed());
    assert!(matches!(values["corner"], TemplateParamValue::Enum { .. }));

    let unknown = bed
        .error_code(
            "instantiate_template",
            json!({ "template_id": template, "start": 20, "params": { "colour": "#fff" } }),
        )
        .await;
    assert_eq!(unknown, "INVALID_ARGUMENTS");
    let wrong = bed
        .error_code(
            "instantiate_template",
            json!({ "template_id": template, "start": 20, "params": { "size": "big" } }),
        )
        .await;
    assert_eq!(wrong, "INVALID_ARGUMENTS");
    let too_big = bed
        .error_code(
            "instantiate_template",
            json!({ "template_id": template, "start": 20, "params": { "size": 900 } }),
        )
        .await;
    assert_eq!(too_big, "VALIDATION_ERROR");
    let missing = bed
        .error_code(
            "instantiate_template",
            json!({ "template_id": "00000000-0099-7000-8000-000000000000", "start": 20 }),
        )
        .await;
    assert_eq!(missing, "TEMPLATE_NOT_FOUND");
}

#[tokio::test]
async fn update_template_params_sets_and_resets_values() {
    let bed = Bed::open().await;
    let item = ItemId::from_test_index(INSTANCE).to_string();
    bed.json(
        "update_template_params",
        json!({ "item_id": item, "params": { "name": "Grace Hopper", "accent": null } }),
    )
    .await;
    let values = instance(&bed.document().await, &item).params;
    assert_eq!(
        values["name"],
        TemplateParamValue::Text {
            value: "Grace Hopper".to_owned()
        }
    );
    assert!(!values.contains_key("accent"));
    let text = ItemId::from_test_index(3).to_string();
    let not_template = bed
        .error_code(
            "update_template_params",
            json!({ "item_id": text, "params": { "name": "x" } }),
        )
        .await;
    assert_eq!(not_template, "INVALID_ARGUMENTS");
    let batch = bed
        .json(
            "apply_batch",
            json!({ "actions": [
                { "tool": "update_template_params", "arguments": { "item_id": item, "params": { "slide": 0 } } }
            ] }),
        )
        .await;
    assert!(batch["revision"].as_u64().is_some(), "{batch}");
    let values = instance(&bed.document().await, &item).params;
    assert_eq!(
        values["slide"],
        TemplateParamValue::Number {
            value: Animatable::constant(0.0)
        }
    );
}

#[tokio::test]
async fn update_motion_template_changes_instances_in_place() {
    let bed = Bed::open().await;
    let template = example_ids::TEMPLATE.to_string();
    let result = bed
        .call_with_check(
            "update_motion_template",
            json!({ "template_id": template, "source": SOURCE }),
            passed(r#"[["name", {"type": "string", "default": "Jane"}]]"#),
        )
        .await;
    let output: Value = first_json(&result);
    assert_eq!(output["template"]["template_id"], template);
    assert_eq!(output["template"]["version"], 2);
    assert!(output.get("copied_from").is_none());
    let item = ItemId::from_test_index(INSTANCE).to_string();
    assert_eq!(output["instances"], json!([item]));
    assert_eq!(
        output["reset"][0]["params"],
        json!(["accent", "showLine", "slide"])
    );
}
