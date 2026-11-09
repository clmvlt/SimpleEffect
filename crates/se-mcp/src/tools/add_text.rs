//! `add_text`: a styled text over a time range of the task's sequence, with optional entrance
//! and exit animations, in one edit.

use schemars::JsonSchema;
use se_app::agent_edits::{AgentEdit, NewAgentItem};
use se_domain::{Animatable, DesignPixels, ItemKind, TextAlign, TextCase, TextItem, TextStyle};
use serde::Deserialize;

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::TimeInput,
    write_inputs::{AnimationInput, PositionInput, StrokeInput, color, presets, span},
    writes::{Planned, WriteScene, WriteTool, run},
};

/// The tool.
pub struct AddText;

/// Horizontal alignment of the lines.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlignInput {
    /// Lines start at the left.
    Left,
    /// Lines are centred (default).
    Center,
    /// Lines end at the right.
    Right,
}

impl AlignInput {
    /// The document's alignment.
    #[must_use]
    pub const fn to_align(self) -> TextAlign {
        match self {
            Self::Left => TextAlign::Left,
            Self::Center => TextAlign::Center,
            Self::Right => TextAlign::Right,
        }
    }
}

/// A box behind the text.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BackgroundInput {
    /// `#rrggbb[aa]`; default translucent black.
    pub color: Option<String>,
}

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The text (`\n` for line breaks).
    pub text: String,
    /// When it appears, in sequence time (inside the task's range).
    pub start: TimeInput,
    /// When it disappears (excluded). Give `end` or `duration`.
    pub end: Option<TimeInput>,
    /// How long it stays. Give `end` or `duration`.
    pub duration: Option<TimeInput>,
    /// Where its centre sits; default "center".
    pub position: Option<PositionInput>,
    /// Size in design pixels (1080 = the frame's shorter side); default: the brand kit's text
    /// style, else 96. Short videos: 80–140 for titles, 48–64 for captions.
    pub font_size: Option<f64>,
    /// Font family; default: the brand kit's text style, else "Inter".
    pub font_family: Option<String>,
    /// Weight 1–1000 (400 regular, 700 bold, 900 black); default: the brand kit's, else 600.
    pub font_weight: Option<u16>,
    /// Italic; default: the brand kit's, else false.
    pub italic: Option<bool>,
    /// Colour `#rrggbb[aa]`; default: the brand kit's text style, else white.
    pub color: Option<String>,
    /// Line alignment; default center.
    pub align: Option<AlignInput>,
    /// Wrapping width in design pixels; default none (one line per `\n`).
    pub max_width: Option<f64>,
    /// Show in capitals; default: the brand kit's, else false.
    pub uppercase: Option<bool>,
    /// An outline (improves legibility over video).
    pub stroke: Option<StrokeInput>,
    /// A soft drop shadow; default: the brand kit's, else false.
    pub shadow: Option<bool>,
    /// A box behind the text.
    pub background: Option<BackgroundInput>,
    /// A video track (id or name); default: the highest free track of the task.
    pub track: Option<String>,
    /// Entrance animation (preset `in` phase), e.g. {"preset": "pop"}.
    pub animation_in: Option<AnimationInput>,
    /// Exit animation (preset `out` phase), e.g. {"preset": "fade", "duration": 0.3}.
    pub animation_out: Option<AnimationInput>,
}

impl McpTool for AddText {
    type Input = Input;

    const NAME: &'static str = "add_text";

    const DESCRIPTION: &'static str = "Adds an editable text over a time range of the task's \
        sequence: content, place, size, font, colour, outline, shadow, box, and optional \
        entrance/exit animations (presets), all in one step. Unset style fields take the \
        project's brand kit text style (get_brand_kit). Answers the new item_id. Check the \
        result with get_frames.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for AddText {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        if input.text.trim().is_empty() {
            return Err(ToolError::invalid_arguments("text: give the words to show"));
        }
        let range = span(
            scene,
            &input.start,
            input.end.as_ref(),
            input.duration.as_ref(),
        )?;
        let mut text = TextItem::new(input.text.clone());
        let base = scene
            .project
            .brand_kit
            .as_ref()
            .map_or_else(TextStyle::default, |kit| kit.text_style.clone());
        text.style = style(&input, base)?;
        if let Some(position) = input.position {
            text.transform.position = Animatable::constant(position.to_vec2());
        }
        let new = NewAgentItem {
            kind: ItemKind::Text(text),
            range,
            track: input
                .track
                .as_deref()
                .map(|track| scene.video_track(track))
                .transpose()?,
            presets: presets(
                scene,
                input.animation_in.as_ref(),
                input.animation_out.as_ref(),
            )?,
        };
        Ok(Planned {
            edits: vec![AgentEdit::AddItem(Box::new(new))],
            creates: Some(0),
        })
    }
}

/// The text style the input asks for, over `base` (the brand kit's text style, or the defaults).
fn style(input: &Input, base: TextStyle) -> Result<TextStyle, ToolError> {
    let mut style = base;
    if let Some(family) = &input.font_family {
        style.font_family.clone_from(family);
    }
    if let Some(size) = input.font_size {
        style.font_size = Animatable::constant(DesignPixels(size));
    }
    if let Some(weight) = input.font_weight {
        style.font_weight = weight;
    }
    if let Some(italic) = input.italic {
        style.italic = italic;
    }
    if let Some(text) = &input.color {
        style.color = Animatable::constant(color("color", text)?);
    }
    if let Some(align) = input.align {
        style.align = align.to_align();
    }
    if let Some(width) = input.max_width {
        style.max_width = Some(DesignPixels(width));
    }
    if let Some(uppercase) = input.uppercase {
        style.text_case = if uppercase {
            TextCase::Uppercase
        } else {
            TextCase::Normal
        };
    }
    if let Some(stroke) = &input.stroke {
        style.stroke.enabled = true;
        if let Some(text) = &stroke.color {
            style.stroke.color = Animatable::constant(color("stroke.color", text)?);
        }
        if let Some(width) = stroke.width {
            style.stroke.width = Animatable::constant(DesignPixels(width));
        }
    }
    if let Some(shadow) = input.shadow {
        style.shadow.enabled = shadow;
    }
    if let Some(background) = &input.background {
        style.background.enabled = true;
        if let Some(text) = &background.color {
            style.background.color = Animatable::constant(color("background.color", text)?);
        }
    }
    Ok(style)
}
