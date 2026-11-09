//! Version 9 → 10 (prompt 25, ADR-0036): complete brand kits; templates get a category and a
//! library link.
//!
//! Version 9 brand kits were skeletons (`{ colors, fonts }`) that nothing could set: the first
//! colours become the palette (primary, secondary, accent, text, background, in that order, with
//! neutral defaults for missing ones), the first font the heading font and the second (or the
//! first) the body font. The migrated kit has no library source, no logo, the default text style
//! in the heading font and the `minimal` caption look in the body font. Templates are filed under
//! `other` and linked to no library entry.

use serde_json::{Map, Value, json};

/// Id of the caption look of a migrated kit (never used by captions: new styles get their own).
pub const MIGRATED_BRAND_CAPTION_STYLE_ID: &str = "00000000-0000-7000-8000-0000000b4a4d";

/// Name of a migrated kit.
pub const MIGRATED_BRAND_KIT_NAME: &str = "Brand kit";

/// Palette colours of a migrated kit missing from its colour list, in role order.
const DEFAULT_PALETTE: [&str; 5] = ["#ffffff", "#1a1a1a", "#ffcc00", "#ffffff", "#000000"];

/// Version 9 → 10 (see the module documentation).
pub(super) fn migrate_v9_to_v10(mut document: Value) -> Result<Value, String> {
    let Some(project) = document.as_object_mut() else {
        return Err("the document is not an object".to_owned());
    };
    if let Some(templates) = project.get_mut("templates").and_then(Value::as_array_mut) {
        for template in templates {
            let fields = template
                .as_object_mut()
                .ok_or_else(|| "a template is not an object".to_owned())?;
            fields.entry("category").or_insert_with(|| json!("other"));
            fields.entry("library").or_insert(Value::Null);
        }
    }
    if let Some(kit) = project.get("brandKit").and_then(Value::as_object) {
        let migrated = complete_kit(kit)?;
        project.insert("brandKit".to_owned(), migrated);
    }
    Ok(document)
}

fn complete_kit(skeleton: &Map<String, Value>) -> Result<Value, String> {
    let strings = |key: &str| -> Vec<String> {
        skeleton
            .get(key)
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .filter(|text| !text.trim().is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    };
    let colors = strings("colors");
    let fonts = strings("fonts");
    let color = |index: usize| {
        colors
            .get(index)
            .cloned()
            .unwrap_or_else(|| DEFAULT_PALETTE[index].to_owned())
    };
    let heading = fonts
        .first()
        .cloned()
        .unwrap_or_else(|| se_domain::TextItem::DEFAULT_FONT_FAMILY.to_owned());
    let body = fonts.get(1).cloned().unwrap_or_else(|| heading.clone());
    let text_style = se_domain::TextStyle {
        font_family: heading.clone(),
        ..se_domain::TextStyle::default()
    };
    let style_id = se_domain::CaptionStyleId::parse(MIGRATED_BRAND_CAPTION_STYLE_ID)
        .map_err(|error| error.to_string())?;
    let mut caption_style =
        se_domain::caption_style_from_preset(se_domain::DEFAULT_CAPTION_PRESET, style_id)
            .map_err(|error| error.to_string())?;
    caption_style.font_family.clone_from(&body);
    Ok(json!({
        "source": null,
        "name": MIGRATED_BRAND_KIT_NAME,
        "fonts": { "heading": heading, "body": body },
        "colors": {
            "primary": color(0),
            "secondary": color(1),
            "accent": color(2),
            "text": color(3),
            "background": color(4),
        },
        "logos": [],
        "textStyle": serde_json::to_value(text_style).map_err(|error| error.to_string())?,
        "captionStyle": serde_json::to_value(caption_style).map_err(|error| error.to_string())?,
        "introTemplates": [],
        "outroTemplates": [],
    }))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn skeleton_kits_become_complete_kits() {
        let document = json!({
            "templates": [{ "id": "x" }],
            "brandKit": { "colors": ["#ff0000", "#00ff00"], "fonts": ["Montserrat"] }
        });
        let migrated = migrate_v9_to_v10(document).unwrap();
        assert_eq!(migrated["templates"][0]["category"], "other");
        assert_eq!(migrated["templates"][0]["library"], Value::Null);
        let kit: se_domain::BrandKit =
            serde_json::from_value(migrated["brandKit"].clone()).unwrap();
        assert_eq!(kit.name, MIGRATED_BRAND_KIT_NAME);
        assert_eq!(kit.fonts.heading, "Montserrat");
        assert_eq!(kit.fonts.body, "Montserrat");
        assert_eq!(kit.colors.primary, se_domain::Color::opaque(255, 0, 0));
        assert_eq!(kit.colors.secondary, se_domain::Color::opaque(0, 255, 0));
        assert_eq!(kit.colors.accent, se_domain::Color::opaque(0xff, 0xcc, 0));
        assert_eq!(kit.text_style.font_family, "Montserrat");
        assert_eq!(kit.problems(), Vec::<(String, String)>::new());
    }

    #[test]
    fn documents_without_a_kit_keep_none() {
        let migrated = migrate_v9_to_v10(json!({ "brandKit": null })).unwrap();
        assert_eq!(migrated["brandKit"], Value::Null);
    }
}
