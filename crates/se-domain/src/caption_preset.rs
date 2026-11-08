//! Predefined caption styles (prompt 23): data files in `packages/engine/src/caption-styles/`,
//! shared with the renderer's style panel, embedded here so the core can create styles from them
//! (generation, agents).
//!
//! A preset file is `{ "id": "tiktokBold", "style": { …every CaptionStyle field but id and
//! preset… } }`. Ids are stable (documents keep them in `CaptionStyle::preset`, agents and the
//! renderer's translations `editor.captions.presets.<id>` use them): add presets, never rename one.

use std::sync::LazyLock;

use serde::Deserialize;

use crate::{CaptionStyle, CaptionStyleId, DomainError};

/// The preset files, in the order the interface lists them.
const PRESET_FILES: [&str; 4] = [
    include_str!("../../../packages/engine/src/caption-styles/tiktok-bold.json"),
    include_str!("../../../packages/engine/src/caption-styles/minimal.json"),
    include_str!("../../../packages/engine/src/caption-styles/karaoke.json"),
    include_str!("../../../packages/engine/src/caption-styles/box.json"),
];

/// Preset used when a request names none.
pub const DEFAULT_CAPTION_PRESET: &str = "minimal";

/// A predefined caption style.
#[derive(Debug, Clone, PartialEq)]
pub struct CaptionStylePreset {
    /// Stable id (`tiktokBold`, `minimal`, `karaoke`, `box`).
    pub id: String,
    /// The style it gives, with a placeholder id and `preset` set to the preset's id.
    pub style: CaptionStyle,
}

#[derive(Deserialize)]
struct PresetFile {
    id: String,
    style: serde_json::Map<String, serde_json::Value>,
}

/// The placeholder id of preset styles (replaced by [`caption_style_from_preset`]).
const PLACEHOLDER_ID: &str = "00000000-0000-7000-8000-000000000000";

fn parse_preset(text: &str) -> Result<CaptionStylePreset, String> {
    let file: PresetFile = serde_json::from_str(text).map_err(|error| error.to_string())?;
    let mut fields = file.style;
    fields.insert("id".to_owned(), PLACEHOLDER_ID.into());
    fields.insert("preset".to_owned(), file.id.clone().into());
    let style: CaptionStyle = serde_json::from_value(fields.into())
        .map_err(|error| format!("preset {}: {error}", file.id))?;
    if let Some((field, rule)) = style.problems().first() {
        return Err(format!("preset {}: {field} {rule}", file.id));
    }
    Ok(CaptionStylePreset { id: file.id, style })
}

static PRESETS: LazyLock<Result<Vec<CaptionStylePreset>, String>> =
    LazyLock::new(|| PRESET_FILES.iter().map(|text| parse_preset(text)).collect());

/// Every predefined caption style.
///
/// # Errors
/// [`DomainError::CaptionPresetsUnavailable`] if an embedded file is broken (a test guards
/// against it).
pub fn caption_style_presets() -> Result<&'static [CaptionStylePreset], DomainError> {
    PRESETS
        .as_deref()
        .map_err(|reason| DomainError::CaptionPresetsUnavailable {
            reason: reason.clone(),
        })
}

/// A new style with id `id` made from the preset `preset`.
///
/// # Errors
/// [`DomainError::UnknownCaptionPreset`] (listing the presets) or
/// [`DomainError::CaptionPresetsUnavailable`].
pub fn caption_style_from_preset(
    preset: &str,
    id: CaptionStyleId,
) -> Result<CaptionStyle, DomainError> {
    let presets = caption_style_presets()?;
    let found = presets.iter().find(|candidate| candidate.id == preset);
    let Some(found) = found else {
        return Err(DomainError::UnknownCaptionPreset {
            preset: preset.to_owned(),
            known: presets
                .iter()
                .map(|candidate| candidate.id.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        });
    };
    Ok(CaptionStyle {
        id,
        ..found.style.clone()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_four_presets_load_in_order() {
        let ids: Vec<_> = caption_style_presets()
            .unwrap()
            .iter()
            .map(|preset| preset.id.as_str())
            .collect();
        assert_eq!(ids, ["tiktokBold", "minimal", "karaoke", "box"]);
        assert!(ids.contains(&DEFAULT_CAPTION_PRESET));
    }

    #[test]
    fn a_style_from_a_preset_gets_the_given_id() {
        let id = CaptionStyleId::from_test_index(9);
        let style = caption_style_from_preset("karaoke", id).unwrap();
        assert_eq!(style.id, id);
        assert_eq!(style.preset.as_deref(), Some("karaoke"));
        let error = caption_style_from_preset("disco", id).unwrap_err();
        assert_eq!(
            error.to_string(),
            "there is no caption style preset 'disco'; use one of: tiktokBold, minimal, \
             karaoke, box"
        );
    }
}
