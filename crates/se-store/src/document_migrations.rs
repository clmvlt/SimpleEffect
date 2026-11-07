//! Upgrading stored documents to the current format (`schemaVersion`, ADR-0006).
//!
//! A migration is a pure function from the JSON of version `N` to the JSON of version `N + 1`.
//! When the document format changes (the snapshot in `se-domain/src/document/snapshots/`
//! changes):
//! 1. bump `se_domain::CURRENT_SCHEMA_VERSION`;
//! 2. add `fn migrate_vN_to_vN+1(document: Value) -> Result<Value, String>` here and append it to
//!    [`MIGRATIONS`];
//! 3. keep the fixtures of older versions (`tests/fixtures/project-vN.json`) and add one for the
//!    new version.
//!
//! History: version 2 (prompt 08) gave assets a `fingerprint` and a per-stream `decodability`;
//! version 3 (prompt 09) gave them `analysis` summaries; version 4 (prompt 14) gave tracks their
//! `transitions` and media clips their fade shapes; version 5 (prompt 15) gave texts their full
//! style and `reveal`, shapes an outline, images a fit and transforms a blur, and made font
//! sizes, text colours and shape fills animatable. Journals need no migration: no transaction
//! touched assets before version 2, every field added since has a serde default (empty, linear,
//! off), and `Animatable` reads the bare values of properties that became animatable.

use serde_json::{Map, Value, json};

/// One step: the document of version `N` (index `N - 1`) to version `N + 1`.
pub type DocumentMigration = fn(Value) -> Result<Value, String>;

/// Every migration, in order: `MIGRATIONS[0]` turns version 1 into version 2, and so on.
/// Its length is always `CURRENT_SCHEMA_VERSION - 1` (checked by a test).
pub const MIGRATIONS: &[DocumentMigration] = &[
    migrate_v1_to_v2,
    migrate_v2_to_v3,
    migrate_v3_to_v4,
    migrate_v4_to_v5,
];

/// Version 1 → 2: every asset gets `fingerprint: null` and its single decodability becomes one
/// state per stream (`noStream` for the streams its media info lacks).
fn migrate_v1_to_v2(mut document: Value) -> Result<Value, String> {
    let Some(assets) = document.get_mut("assets").and_then(Value::as_array_mut) else {
        return Err("`assets` is missing or not a list".to_owned());
    };
    for asset in assets {
        let fields = asset
            .as_object_mut()
            .ok_or_else(|| "an asset is not an object".to_owned())?;
        let decodability = per_stream_decodability(fields);
        fields.insert("decodability".to_owned(), decodability);
        fields.insert("fingerprint".to_owned(), Value::Null);
    }
    Ok(document)
}

/// Version 2 → 3: every asset gets empty analysis summaries (the analyses run again).
fn migrate_v2_to_v3(mut document: Value) -> Result<Value, String> {
    let Some(assets) = document.get_mut("assets").and_then(Value::as_array_mut) else {
        return Err("`assets` is missing or not a list".to_owned());
    };
    for asset in assets {
        let fields = asset
            .as_object_mut()
            .ok_or_else(|| "an asset is not an object".to_owned())?;
        fields.insert(
            "analysis".to_owned(),
            json!({ "audio": null, "scenes": null }),
        );
    }
    Ok(document)
}

/// Version 3 → 4: every track gets an empty `transitions` list and every media clip linear
/// fade shapes (what version 3 played).
fn migrate_v3_to_v4(mut document: Value) -> Result<Value, String> {
    let Some(sequences) = document.get_mut("sequences").and_then(Value::as_array_mut) else {
        return Err("`sequences` is missing or not a list".to_owned());
    };
    for sequence in sequences {
        let tracks = sequence
            .get_mut("tracks")
            .and_then(Value::as_array_mut)
            .ok_or_else(|| "a sequence has no `tracks` list".to_owned())?;
        for track in tracks {
            let fields = track
                .as_object_mut()
                .ok_or_else(|| "a track is not an object".to_owned())?;
            for item in fields
                .get_mut("items")
                .and_then(Value::as_array_mut)
                .into_iter()
                .flatten()
            {
                add_fade_shapes(item);
            }
            fields.insert("transitions".to_owned(), json!([]));
        }
    }
    Ok(document)
}

/// Version 4 → 5: the font size and colour of texts and the fill of shapes become static
/// animatable values (`{"kind": "static", "value": …}`). Fields added by version 5 (font weight,
/// outline, shadow, plate, spacing, alignment, wrapping, case, reveal, image fit, blur) take
/// their defaults when the document is read.
fn migrate_v4_to_v5(mut document: Value) -> Result<Value, String> {
    let Some(sequences) = document.get_mut("sequences").and_then(Value::as_array_mut) else {
        return Err("`sequences` is missing or not a list".to_owned());
    };
    let items = sequences
        .iter_mut()
        .filter_map(|sequence| sequence.get_mut("tracks").and_then(Value::as_array_mut))
        .flatten()
        .filter_map(|track| track.get_mut("items").and_then(Value::as_array_mut))
        .flatten();
    for item in items {
        let Some(kind) = item.get_mut("kind").and_then(Value::as_object_mut) else {
            continue;
        };
        match kind.get("kind").and_then(Value::as_str) {
            Some("text") => {
                if let Some(style) = kind.get_mut("style").and_then(Value::as_object_mut) {
                    make_static(style, "fontSize");
                    make_static(style, "color");
                }
            }
            Some("shape") => make_static(kind, "fill"),
            _ => {}
        }
    }
    Ok(document)
}

/// Wraps the bare value of `field` in a static animatable value.
fn make_static(fields: &mut Map<String, Value>, field: &str) {
    if let Some(value) = fields.get_mut(field) {
        let bare = value.take();
        *value = json!({ "kind": "static", "value": bare });
    }
}

/// Linear fade shapes on `item` when it is a media clip.
fn add_fade_shapes(item: &mut Value) {
    let Some(kind) = item.get_mut("kind").and_then(Value::as_object_mut) else {
        return;
    };
    if kind.get("kind").and_then(Value::as_str) == Some("mediaClip") {
        kind.insert("fadeInShape".to_owned(), json!("linear"));
        kind.insert("fadeOutShape".to_owned(), json!("linear"));
    }
}

/// The version-2 `decodability` of a version-1 asset.
fn per_stream_decodability(asset: &Map<String, Value>) -> Value {
    let info = asset.get("mediaInfo").filter(|info| !info.is_null());
    let has_video = info
        .and_then(|info| info.get("videoStream"))
        .is_some_and(|stream| !stream.is_null());
    let has_audio = info
        .and_then(|info| info.get("audioStreams"))
        .and_then(Value::as_array)
        .is_some_and(|streams| !streams.is_empty());
    let (video, audio) = match asset.get("decodability").and_then(Value::as_str) {
        Some("decodable") => ("decodable", "decodable"),
        Some("needsMezzanine") => ("undecodable", "unknown"),
        _ => ("unknown", "unknown"),
    };
    let state = |present: bool, known: &str| {
        if present {
            known.to_owned()
        } else {
            "noStream".to_owned()
        }
    };
    json!({ "video": state(has_video, video), "audio": state(has_audio, audio) })
}

/// Field holding the format version.
const VERSION_FIELD: &str = "schemaVersion";

/// Why a document could not be migrated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationError {
    /// `schemaVersion` is missing or not a positive integer.
    MissingVersion,
    /// The document comes from a newer version of the app.
    TooNew {
        /// Version found in the document.
        found: u32,
        /// Newest version known.
        supported: u32,
    },
    /// A migration refused the document.
    Failed {
        /// Version the failing migration starts from.
        from: u32,
        /// Its explanation.
        reason: String,
    },
}

/// Upgrades `document` to `current` with `migrations` (see [`MIGRATIONS`]).
///
/// # Errors
/// [`MigrationError`] when the version is missing or too new, or a migration fails.
pub fn migrate_document(
    mut document: Value,
    migrations: &[DocumentMigration],
    current: u32,
) -> Result<Value, MigrationError> {
    let version = document
        .get(VERSION_FIELD)
        .and_then(Value::as_u64)
        .and_then(|version| u32::try_from(version).ok())
        .filter(|version| *version >= 1)
        .ok_or(MigrationError::MissingVersion)?;
    if version > current {
        return Err(MigrationError::TooNew {
            found: version,
            supported: current,
        });
    }
    for from in version..current {
        let migration = usize::try_from(from - 1)
            .ok()
            .and_then(|index| migrations.get(index))
            .ok_or_else(|| MigrationError::Failed {
                from,
                reason: format!("no migration from version {from} is registered"),
            })?;
        document = migration(document).map_err(|reason| MigrationError::Failed { from, reason })?;
        if let Some(fields) = document.as_object_mut() {
            fields.insert(VERSION_FIELD.to_owned(), Value::from(from + 1));
        }
    }
    Ok(document)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[allow(
        clippy::unnecessary_wraps,
        reason = "migrations share the fallible `DocumentMigration` signature"
    )]
    fn add_title(mut document: Value) -> Result<Value, String> {
        document["title"] = json!("added by v2");
        Ok(document)
    }

    fn refuse(_: Value) -> Result<Value, String> {
        Err("cannot convert".to_owned())
    }

    #[test]
    fn there_is_one_migration_per_version_after_the_first() {
        let expected = se_domain::CURRENT_SCHEMA_VERSION - 1;
        assert_eq!(MIGRATIONS.len(), usize::try_from(expected).unwrap());
    }

    #[test]
    fn current_documents_are_left_unchanged() {
        let document = json!({ "schemaVersion": 2, "name": "Film" });
        assert_eq!(
            migrate_document(document.clone(), MIGRATIONS, 2),
            Ok(document)
        );
    }

    #[test]
    fn version_2_splits_decodability_per_stream_and_adds_fingerprints() {
        let document = json!({
            "schemaVersion": 1,
            "assets": [
                { "decodability": "decodable", "mediaInfo": { "videoStream": null, "audioStreams": [{}] } },
                { "decodability": "needsMezzanine", "mediaInfo": { "videoStream": {}, "audioStreams": [{}] } },
                { "decodability": "unknown", "mediaInfo": null }
            ]
        });
        let migrated = migrate_document(document, MIGRATIONS, 2).unwrap();
        let assets = migrated["assets"].as_array().unwrap();
        assert_eq!(
            assets[0]["decodability"],
            json!({ "video": "noStream", "audio": "decodable" })
        );
        assert_eq!(
            assets[1]["decodability"],
            json!({ "video": "undecodable", "audio": "unknown" })
        );
        assert_eq!(
            assets[2]["decodability"],
            json!({ "video": "noStream", "audio": "noStream" })
        );
        assert!(assets.iter().all(|asset| asset["fingerprint"].is_null()));
        assert_eq!(migrated["schemaVersion"], json!(2));
    }

    #[test]
    fn version_3_adds_empty_analysis_summaries() {
        let document = json!({ "schemaVersion": 2, "assets": [{ "id": "a" }] });
        let migrated = migrate_document(document, MIGRATIONS, 3).unwrap();
        assert_eq!(
            migrated["assets"][0]["analysis"],
            json!({ "audio": null, "scenes": null })
        );
        assert_eq!(migrated["schemaVersion"], json!(3));
    }

    #[test]
    fn version_4_adds_transitions_and_linear_fade_shapes() {
        let document = json!({
            "schemaVersion": 3,
            "sequences": [{ "tracks": [{ "items": [
                { "kind": { "kind": "mediaClip", "fadeIn": 0 } },
                { "kind": { "kind": "text" } }
            ] }] }]
        });
        let migrated = migrate_document(document, MIGRATIONS, 4).unwrap();
        let track = &migrated["sequences"][0]["tracks"][0];
        assert_eq!(track["transitions"], json!([]));
        assert_eq!(track["items"][0]["kind"]["fadeInShape"], json!("linear"));
        assert_eq!(track["items"][0]["kind"]["fadeOutShape"], json!("linear"));
        assert_eq!(track["items"][1]["kind"], json!({ "kind": "text" }));
        assert_eq!(migrated["schemaVersion"], json!(4));
    }

    #[test]
    fn version_5_makes_text_sizes_colours_and_shape_fills_animatable() {
        let document = json!({
            "schemaVersion": 4,
            "sequences": [{ "tracks": [{ "items": [
                { "kind": { "kind": "text", "style": { "fontSize": 72.0, "color": "#ffffff" } } },
                { "kind": { "kind": "shape", "fill": "#000000cc" } },
                { "kind": { "kind": "caption", "text": "Hi" } }
            ] }] }]
        });
        let migrated = migrate_document(document, MIGRATIONS, 5).unwrap();
        let items = &migrated["sequences"][0]["tracks"][0]["items"];
        assert_eq!(
            items[0]["kind"]["style"],
            json!({
                "fontSize": { "kind": "static", "value": 72.0 },
                "color": { "kind": "static", "value": "#ffffff" }
            })
        );
        assert_eq!(
            items[1]["kind"]["fill"],
            json!({ "kind": "static", "value": "#000000cc" })
        );
        assert_eq!(items[2]["kind"], json!({ "kind": "caption", "text": "Hi" }));
        assert_eq!(migrated["schemaVersion"], json!(5));
    }

    #[test]
    fn older_documents_go_through_every_step() {
        let migrated = migrate_document(json!({ "schemaVersion": 1 }), &[add_title], 2).unwrap();
        assert_eq!(
            migrated,
            json!({ "schemaVersion": 2, "title": "added by v2" })
        );
    }

    #[test]
    fn newer_or_unversioned_documents_are_refused() {
        assert_eq!(
            migrate_document(json!({ "schemaVersion": 9 }), MIGRATIONS, 2),
            Err(MigrationError::TooNew {
                found: 9,
                supported: 2
            })
        );
        assert_eq!(
            migrate_document(json!({ "name": "?" }), MIGRATIONS, 2),
            Err(MigrationError::MissingVersion)
        );
    }

    #[test]
    fn a_failing_migration_names_its_version() {
        let error = migrate_document(json!({ "schemaVersion": 1 }), &[refuse], 2).unwrap_err();
        assert_eq!(
            error,
            MigrationError::Failed {
                from: 1,
                reason: "cannot convert".to_owned()
            }
        );
    }
}
