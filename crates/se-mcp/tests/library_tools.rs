//! Library templates and brand kits for agents (prompt 25): the library listed by category,
//! `insert_library_template` (copied once, brand kit values, explicit values, refusals), a
//! project without a kit.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

mod support;

use se_domain::{
    ItemId, ItemKind, TemplateCategory, TemplateParamValue, test_support::example_ids,
};
use se_protocol::LibraryTemplateInfo;
use serde_json::json;
use support::Bed;

async fn saved_entry(bed: &Bed) -> String {
    let entry = bed
        .services
        .library_templates
        .save_template(
            bed.project_id(),
            example_ids::TEMPLATE,
            &LibraryTemplateInfo {
                name: "Bandeau".to_owned(),
                description: "Nom et fonction".to_owned(),
                tags: vec!["interview".to_owned()],
                category: Some(TemplateCategory::LowerThird),
            },
        )
        .await
        .unwrap();
    entry.id.to_string()
}

#[tokio::test]
async fn library_templates_are_listed_by_category_with_their_project_copy() {
    let bed = Bed::open().await;
    let id = saved_entry(&bed).await;
    let listing = bed
        .json("list_templates", json!({ "source": "library" }))
        .await;
    let entry = &listing["templates"][0];
    assert_eq!(entry["library_template_id"], id);
    assert_eq!(entry["category"], "lowerThird");
    assert_eq!(entry["tags"], json!(["interview"]));
    assert_eq!(
        entry["project_template_id"],
        example_ids::TEMPLATE.to_string()
    );
    let none = bed
        .json(
            "list_templates",
            json!({ "source": "library", "category": "callToAction" }),
        )
        .await;
    assert_eq!(none["templates"], json!([]));
    let detail = bed
        .json(
            "list_templates",
            json!({ "source": "library", "template_id": id }),
        )
        .await;
    assert!(
        detail["source"]
            .as_str()
            .unwrap()
            .contains("export function render")
    );
    assert_eq!(
        bed.error_code("list_templates", json!({ "category": "intro" }))
            .await,
        "INVALID_ARGUMENTS"
    );
}

#[tokio::test]
async fn library_templates_are_inserted_with_the_brand_kit_values() {
    let bed = Bed::open().await;
    let id = saved_entry(&bed).await;
    let placed = bed
        .json(
            "insert_library_template",
            json!({ "template_id": id, "start": 20, "params": { "name": "Camille Martin" } }),
        )
        .await;
    let item = ItemId::parse(placed["created"][0].as_str().unwrap()).unwrap();
    let project = bed
        .services
        .projects
        .project(bed.project_id())
        .await
        .unwrap();
    assert_eq!(
        project.templates.len(),
        1,
        "the project's copy of this version is reused"
    );
    let placed_item = project
        .items()
        .find(|candidate| candidate.id == item)
        .unwrap();
    assert_eq!(placed_item.timeline_range.start().to_seconds_f64(), 20.0);
    let ItemKind::MotionTemplate(instance) = &placed_item.kind else {
        panic!("an instance");
    };
    assert_eq!(instance.template_id, example_ids::TEMPLATE);
    assert_eq!(
        instance.params["name"],
        TemplateParamValue::Text {
            value: "Camille Martin".to_owned()
        }
    );
    let accent = project.brand_kit.as_ref().unwrap().colors.accent;
    assert_eq!(
        instance.params["accent"],
        TemplateParamValue::Color {
            value: se_domain::Animatable::constant(accent)
        }
    );

    assert_eq!(
        bed.error_code(
            "insert_library_template",
            json!({ "template_id": "nope", "start": 1 })
        )
        .await,
        "INVALID_ARGUMENTS"
    );
    assert_eq!(
        bed.error_code(
            "insert_library_template",
            json!({ "template_id": "00000000-0042-7000-8000-000000000000", "start": 1 })
        )
        .await,
        "LIBRARY_TEMPLATE_NOT_FOUND"
    );
    assert_eq!(
        bed.error_code(
            "insert_library_template",
            json!({ "template_id": id, "start": 1, "params": { "colour": "#fff" } })
        )
        .await,
        "INVALID_ARGUMENTS"
    );
}

#[tokio::test]
async fn projects_without_a_kit_say_so_and_keep_defaults() {
    let bed = Bed::open().await;
    bed.services
        .brand_kits
        .set_project_kit(bed.project_id(), None)
        .await
        .unwrap();
    let kit = bed.json("get_brand_kit", json!({})).await;
    assert_eq!(kit["available"], false);
    let placed = bed
        .json(
            "add_text",
            json!({ "text": "Titre", "start": 20, "duration": 2 }),
        )
        .await;
    let item = ItemId::parse(placed["created"][0].as_str().unwrap()).unwrap();
    let project = bed
        .services
        .projects
        .project(bed.project_id())
        .await
        .unwrap();
    let ItemKind::Text(text) = &project
        .items()
        .find(|candidate| candidate.id == item)
        .unwrap()
        .kind
    else {
        panic!("a text");
    };
    assert_eq!(text.style.font_family, "Inter");
}
