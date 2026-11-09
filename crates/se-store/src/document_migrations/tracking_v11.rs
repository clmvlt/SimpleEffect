//! Version 10 → 11 (prompt 26, ADR-0037): tracked reframes carry their tracking settings.
//!
//! Nothing could create a tracked reframe before version 11, so documents hold none in practice;
//! any that exists gets the default settings (`TrackingSettings::default()`), its focus kept.

use serde_json::{Value, json};

/// The default tracking settings, as written in documents.
fn default_tracking() -> Value {
    json!({ "smoothing": 0.5, "eyeLine": 0.33, "groupFraming": true, "deadZone": 0.15 })
}

/// Version 10 → 11 (see the module documentation).
pub(super) fn migrate_v10_to_v11(mut document: Value) -> Result<Value, String> {
    let Some(sequences) = document.get_mut("sequences").and_then(Value::as_array_mut) else {
        return Err("`sequences` is missing or not a list".to_owned());
    };
    let reframes = sequences
        .iter_mut()
        .filter_map(|sequence| sequence.get_mut("tracks").and_then(Value::as_array_mut))
        .flatten()
        .filter_map(|track| track.get_mut("items").and_then(Value::as_array_mut))
        .flatten()
        .filter_map(|item| {
            item.pointer_mut("/kind/reframe")
                .and_then(Value::as_object_mut)
        });
    for reframe in reframes {
        if reframe.get("kind").and_then(Value::as_str) == Some("tracked") {
            reframe.entry("tracking").or_insert_with(default_tracking);
        }
    }
    Ok(document)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracked_reframes_get_the_default_settings() {
        let focus = json!({ "kind": "static", "value": { "x": 0.3, "y": 0.5 } });
        let clip = |reframe: Value| json!({ "kind": { "kind": "mediaClip", "reframe": reframe } });
        let document = json!({
            "schemaVersion": 10,
            "sequences": [{ "tracks": [{ "items": [
                clip(json!({ "kind": "tracked", "focus": focus })),
                clip(json!({ "kind": "fill", "focus": focus })),
                { "kind": { "kind": "text" } }
            ] }] }]
        });
        let migrated = migrate_v10_to_v11(document).unwrap();
        let items = &migrated["sequences"][0]["tracks"][0]["items"];
        assert_eq!(
            items[0]["kind"]["reframe"],
            json!({ "kind": "tracked", "focus": focus, "tracking": default_tracking() })
        );
        assert_eq!(
            items[1]["kind"]["reframe"],
            json!({ "kind": "fill", "focus": focus })
        );
        let settings: se_domain::TrackingSettings =
            serde_json::from_value(default_tracking()).unwrap();
        assert_eq!(settings, se_domain::TrackingSettings::default());
    }
}
