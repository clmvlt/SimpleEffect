//! `update_caption_style`: changes how captions look and animate — a predefined style and/or
//! single fields — for the captions of a style or for given captions (a style they share with
//! other captions is copied for them).

use std::collections::BTreeMap;

use schemars::JsonSchema;
use se_app::agent_edits::{AgentEdit, ItemRef};
use se_domain::{
    CaptionAnimation, CaptionStyle, CaptionStyleId, CaptionStylePatch, DesignPixels, ItemId,
    ItemKind, TextCase, caption_style_from_preset,
};
use serde::Deserialize;

use super::create_captions::CAPTION_PRESETS;
use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    write_inputs::{PositionInput, color},
    writes::{Planned, WriteScene, WriteTool, run},
};

/// The tool.
pub struct UpdateCaptionStyle;

/// How words animate while spoken.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnimationInput {
    /// Every word in the text colour.
    None,
    /// The word being spoken in the active colour.
    WordHighlight,
    /// The active word coloured and each word popping as it starts.
    PopPerWord,
    /// Each line on its own box, the active word coloured.
    BoxPerLine,
    /// Words appear as they are spoken.
    RevealPerWord,
}

/// Arguments: `style_id` or `item_ids`, then the fields to change.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Change this caption style (every caption using it must lie in your scope).
    pub style_id: Option<String>,
    /// Or change these captions only: a style they share with other captions is copied for them.
    pub item_ids: Option<Vec<String>>,
    /// First apply a predefined style: `tiktokBold`, `minimal`, `karaoke`, `box`.
    pub preset: Option<String>,
    /// Font family of the catalogue (e.g. `Montserrat`, `Inter`, `Bebas Neue`).
    pub font_family: Option<String>,
    /// Font size in design pixels (frame's shorter side = 1080).
    pub font_size: Option<f64>,
    /// Font weight 1–1000.
    pub font_weight: Option<u16>,
    /// Italic.
    pub italic: Option<bool>,
    /// Show in capitals.
    pub uppercase: Option<bool>,
    /// Colour of words not spoken yet (`#rrggbb[aa]`).
    pub text_color: Option<String>,
    /// Colour of the word being spoken.
    pub active_color: Option<String>,
    /// Colour of emphasised words.
    pub emphasis_color: Option<String>,
    /// Outline on or off.
    pub outline: Option<bool>,
    /// Outline colour.
    pub outline_color: Option<String>,
    /// Outline width in design pixels.
    pub outline_width: Option<f64>,
    /// Drop shadow on or off.
    pub shadow: Option<bool>,
    /// Box behind the text on or off (behind each line with `box_per_line`).
    pub background: Option<bool>,
    /// Box colour.
    pub background_color: Option<String>,
    /// Where the bottom centre of the captions sits (keep it out of platform interfaces: bottom
    /// around 0.85 for 16:9, around 0.7 for 9:16).
    pub position: Option<PositionInput>,
    /// How words animate.
    pub animation: Option<AnimationInput>,
    /// Spoken words keep the active colour (karaoke fill).
    pub highlight_spoken: Option<bool>,
    /// Longest line in characters (4–80).
    pub max_chars_per_line: Option<u32>,
    /// Lines shown at once (1–4).
    pub max_lines: Option<u32>,
}

impl McpTool for UpdateCaptionStyle {
    type Input = Input;

    const NAME: &'static str = "update_caption_style";

    const DESCRIPTION: &'static str = "Changes how captions look and animate: a predefined \
        style (`preset`) and/or single fields (font, size, colours of the text, the spoken word \
        and emphasised words, outline, shadow, box, position, animation, line limits). Give \
        `style_id` to change a whole style (every caption using it must be in your scope) or \
        `item_ids` for some captions (a shared style is copied for them). One undoable \
        transaction; returns the captions changed.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for UpdateCaptionStyle {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        let captions = target_captions(scene, &input)?;
        let mut edits = Vec::new();
        if let Some(preset) = &input.preset {
            let Some(&(_, placeholder)) = captions.first() else {
                return Err(ToolError::invalid_arguments("no caption to change"));
            };
            let base = caption_style_from_preset(preset, placeholder).map_err(|_| {
                ToolError::invalid_arguments(format!(
                    "preset '{preset}' does not exist; use one of: {}",
                    CAPTION_PRESETS.join(", ")
                ))
            })?;
            let items: Vec<ItemRef> = captions
                .iter()
                .map(|(item, _)| ItemRef::Id(*item))
                .collect();
            edits.push(AgentEdit::ApplyCaptionPreset {
                items: items.clone(),
                preset: preset.clone(),
            });
            let patch = patch(&input, &base)?;
            if !patch.is_empty() {
                edits.push(AgentEdit::UpdateCaptionStyle { items, patch });
            }
        } else {
            let mut groups: BTreeMap<CaptionStyleId, Vec<ItemId>> = BTreeMap::new();
            for (item, style) in &captions {
                groups.entry(*style).or_default().push(*item);
            }
            for (style_id, items) in groups {
                let patch = patch(&input, style(scene, style_id)?)?;
                if !patch.is_empty() {
                    edits.push(AgentEdit::UpdateCaptionStyle {
                        items: items.into_iter().map(ItemRef::Id).collect(),
                        patch,
                    });
                }
            }
        }
        if edits.is_empty() {
            return Err(ToolError::invalid_arguments(
                "nothing to change: give a preset or at least one field",
            ));
        }
        Ok(Planned {
            edits,
            creates: None,
        })
    }
}

/// The captions to change with their style: those using `style_id`, or `item_ids`.
fn target_captions(
    scene: &WriteScene<'_>,
    input: &Input,
) -> Result<Vec<(ItemId, CaptionStyleId)>, ToolError> {
    let all: Vec<(ItemId, CaptionStyleId)> = scene
        .project
        .items()
        .filter_map(|item| match &item.kind {
            ItemKind::Caption(caption) => Some((item.id, caption.style_id)),
            _ => None,
        })
        .collect();
    match (&input.style_id, &input.item_ids) {
        (Some(style), None) => {
            let style = CaptionStyleId::parse(style)
                .map_err(|error| ToolError::invalid_arguments(error.to_string()))?;
            let users: Vec<_> = all.into_iter().filter(|(_, used)| *used == style).collect();
            if users.is_empty() {
                return Err(ToolError::invalid_arguments(format!(
                    "no caption uses style {style}; get_item of a caption shows its style_id"
                )));
            }
            Ok(users)
        }
        (None, Some(items)) if !items.is_empty() => items
            .iter()
            .map(|reference| {
                let id = ItemId::parse(reference)
                    .map_err(|error| ToolError::invalid_arguments(error.to_string()))?;
                all.iter()
                    .find(|(item, _)| *item == id)
                    .copied()
                    .ok_or_else(|| {
                        ToolError::invalid_arguments(format!(
                            "{reference} is not a caption; get_timeline lists the captions"
                        ))
                    })
            })
            .collect(),
        _ => Err(ToolError::invalid_arguments(
            "give either style_id or a non-empty item_ids",
        )),
    }
}

fn style<'a>(scene: &WriteScene<'a>, id: CaptionStyleId) -> Result<&'a CaptionStyle, ToolError> {
    scene
        .project
        .caption_styles
        .iter()
        .find(|style| style.id == id)
        .ok_or_else(|| ToolError::invalid_arguments(format!("caption style {id} does not exist")))
}

/// The patch of `input`'s fields over `base` (whole outline, shadow and box built from it).
fn patch(input: &Input, base: &CaptionStyle) -> Result<CaptionStylePatch, ToolError> {
    let colour = |field: &str, value: &Option<String>| {
        value.as_deref().map(|text| color(field, text)).transpose()
    };
    let mut outline = base.outline.clone();
    let mut background = base.background.clone();
    let mut shadow = base.shadow.clone();
    if let Some(enabled) = input.outline {
        outline.enabled = enabled;
    }
    if let Some(value) = colour("outline_color", &input.outline_color)? {
        outline.color = value;
        outline.enabled = true;
    }
    if let Some(width) = input.outline_width {
        outline.width = DesignPixels(width);
        outline.enabled = width > 0.0;
    }
    if let Some(enabled) = input.background {
        background.enabled = enabled;
    }
    if let Some(value) = colour("background_color", &input.background_color)? {
        background.color = value;
        background.enabled = true;
    }
    if let Some(enabled) = input.shadow {
        shadow.enabled = enabled;
    }
    Ok(CaptionStylePatch {
        font_family: input.font_family.clone(),
        font_weight: input.font_weight,
        italic: input.italic,
        font_size: input.font_size.map(DesignPixels),
        text_case: input.uppercase.map(|upper| {
            if upper {
                TextCase::Uppercase
            } else {
                TextCase::Normal
            }
        }),
        text_color: colour("text_color", &input.text_color)?,
        active_color: colour("active_color", &input.active_color)?,
        emphasis_color: colour("emphasis_color", &input.emphasis_color)?,
        outline: (outline != base.outline).then_some(outline),
        shadow: (shadow != base.shadow).then_some(shadow),
        background: (background != base.background).then_some(background),
        position: input.position.map(PositionInput::to_vec2),
        animation: input.animation.map(animation),
        highlight_spoken: input.highlight_spoken,
        max_chars_per_line: input.max_chars_per_line,
        max_lines: input.max_lines,
        ..CaptionStylePatch::default()
    })
}

const fn animation(input: AnimationInput) -> CaptionAnimation {
    match input {
        AnimationInput::None => CaptionAnimation::None,
        AnimationInput::WordHighlight => CaptionAnimation::WordHighlight,
        AnimationInput::PopPerWord => CaptionAnimation::PopPerWord,
        AnimationInput::BoxPerLine => CaptionAnimation::BoxPerLine,
        AnimationInput::RevealPerWord => CaptionAnimation::RevealPerWord,
    }
}
