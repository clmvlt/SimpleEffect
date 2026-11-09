//! Version 8 → 9 (prompt 24, ADR-0035): motion templates get a source, parameters, a version and
//! a default duration; template items a transform.
//!
//! Version 8 had only template skeletons (`{ id, name }`): nothing could create one, so no real
//! project holds them. A migrated template gets a source that draws nothing and declares no
//! parameter, and its items lose their values (no parameter accepts them any more).

use serde_json::{Value, json};

/// Source given to the templates of a version-8 document: valid, draws nothing.
pub const MIGRATED_TEMPLATE_SOURCE: &str = "export const meta = { name: 'Template', defaultDuration: 5 };\n\nexport const params = {};\n\nexport function render() {}\n";

/// Version 8 → 9 (see the module documentation).
pub(super) fn migrate_v8_to_v9(mut document: Value) -> Result<Value, String> {
    let Some(project) = document.as_object_mut() else {
        return Err("the document is not an object".to_owned());
    };
    if let Some(templates) = project.get_mut("templates").and_then(Value::as_array_mut) {
        for template in templates {
            let fields = template
                .as_object_mut()
                .ok_or_else(|| "a template is not an object".to_owned())?;
            fields.entry("description").or_insert_with(|| json!(""));
            fields.entry("version").or_insert_with(|| json!(1));
            fields
                .entry("source")
                .or_insert_with(|| json!(MIGRATED_TEMPLATE_SOURCE));
            fields.entry("params").or_insert_with(|| json!([]));
            fields
                .entry("defaultDuration")
                .or_insert_with(|| json!(se_domain::Flicks::from_millis(5000).get()));
            fields
                .entry("origin")
                .or_insert_with(|| json!({ "kind": "user" }));
        }
    }
    let transform = serde_json::to_value(se_domain::Transform2D::default())
        .map_err(|error| error.to_string())?;
    let Some(sequences) = project.get_mut("sequences").and_then(Value::as_array_mut) else {
        return Err("`sequences` is missing or not a list".to_owned());
    };
    let kinds = sequences
        .iter_mut()
        .filter_map(|sequence| sequence.get_mut("tracks").and_then(Value::as_array_mut))
        .flatten()
        .filter_map(|track| track.get_mut("items").and_then(Value::as_array_mut))
        .flatten()
        .filter_map(|item| item.get_mut("kind").and_then(Value::as_object_mut));
    for kind in kinds {
        if kind.get("kind").and_then(Value::as_str) == Some("motionTemplate") {
            kind.insert("params".to_owned(), json!({}));
            kind.entry("transform").or_insert_with(|| transform.clone());
        }
    }
    Ok(document)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::{MIGRATIONS, migrate_document};
    use super::*;

    #[test]
    fn templates_get_an_empty_definition_and_items_a_transform() {
        let document = json!({
            "schemaVersion": 8,
            "templates": [{ "id": "t", "name": "Lower third" }],
            "sequences": [{ "tracks": [{ "items": [
                { "kind": { "kind": "motionTemplate", "templateId": "t",
                            "params": { "title": { "kind": "text", "value": "Jane" } } } },
                { "kind": { "kind": "text" } }
            ] }] }]
        });
        let migrated = migrate_document(document, MIGRATIONS, 9).unwrap();
        let template = &migrated["templates"][0];
        assert_eq!(template["name"], json!("Lower third"));
        assert_eq!(template["version"], json!(1));
        assert_eq!(template["params"], json!([]));
        assert_eq!(template["source"], json!(MIGRATED_TEMPLATE_SOURCE));
        let items = &migrated["sequences"][0]["tracks"][0]["items"];
        assert_eq!(items[0]["kind"]["params"], json!({}));
        assert_eq!(
            items[0]["kind"]["transform"]["opacity"],
            json!({ "kind": "static", "value": 1.0 })
        );
        assert_eq!(items[1], json!({ "kind": { "kind": "text" } }));
    }
}
