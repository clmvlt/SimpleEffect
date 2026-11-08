//! `update_item`: changes static properties of one item — text content and style, shape size
//! and colour, image fit, clip speed, fades and reframing, and static placement (position,
//! scale, rotation, opacity, blur) — in one edit.

use schemars::JsonSchema;
use se_app::agent_edits::AgentEdit;
use se_domain::{
    Animatable, Assign, DesignPixels, ImagePatch, ItemPatch, MediaClipPatch, PropertyAnimation,
    Reframe, ShapePatch, TextCase, TextPatch, Vec2,
};
use serde::Deserialize;

use super::{add_image::FitInput, add_shape::resized, add_text::AlignInput};
use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::TimeInput,
    write_inputs::{PositionInput, ValueInput, color},
    writes::{Planned, WriteScene, WriteTool, run},
};

/// The tool.
pub struct UpdateItem;

/// How a clip fills the frame.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReframeInput {
    /// Whole picture inside the frame (bars where aspects differ).
    Fit,
    /// Covers the frame, cropped around the centre (move the kept area with `set_keyframes`
    /// `reframeFocus`).
    Fill,
    /// Natural size (one picture pixel per design pixel).
    Custom,
}

/// Arguments: give only the fields to change.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The item (an `item_id`, or `$name` in `apply_batch`).
    pub item_id: String,
    /// The revision you read; the change is refused (`REVISION_CONFLICT`) if the item changed since.
    pub expected_revision: Option<u64>,
    /// Texts: new content.
    pub text: Option<String>,
    /// Texts: font family.
    pub font_family: Option<String>,
    /// Texts: size in design pixels (static; replaces keyframes).
    pub font_size: Option<f64>,
    /// Texts: weight 1–1000.
    pub font_weight: Option<u16>,
    /// Texts: italic.
    pub italic: Option<bool>,
    /// Texts: text colour; shapes: fill colour (`#rrggbb[aa]`, static).
    pub color: Option<String>,
    /// Texts: line alignment.
    pub align: Option<AlignInput>,
    /// Texts: wrapping width in design pixels (0 removes wrapping).
    pub max_width: Option<f64>,
    /// Texts: show in capitals.
    pub uppercase: Option<bool>,
    /// Texts and shapes: outline on or off.
    pub stroke: Option<bool>,
    /// Texts: drop shadow on or off.
    pub shadow: Option<bool>,
    /// Texts: box behind the text on or off.
    pub background: Option<bool>,
    /// Shapes: width in design pixels (a line's length).
    pub width: Option<f64>,
    /// Shapes: height in design pixels (a line's thickness).
    pub height: Option<f64>,
    /// Shapes: corner radius (rectangles).
    pub corner_radius: Option<f64>,
    /// Images: how the image is sized.
    pub fit: Option<FitInput>,
    /// Clips: playback speed (0.25–4).
    pub speed: Option<f64>,
    /// Clips: fade-in length.
    pub fade_in: Option<TimeInput>,
    /// Clips: fade-out length.
    pub fade_out: Option<TimeInput>,
    /// Clips: how the picture fills the frame.
    pub reframe: Option<ReframeInput>,
    /// Visual items: where the centre sits (static; replaces keyframes).
    pub position: Option<PositionInput>,
    /// Visual items: scale, a number or `[x, y]` (1 = natural; static).
    pub scale: Option<ValueInput>,
    /// Visual items: rotation in degrees, clockwise (static).
    pub rotation: Option<f64>,
    /// Visual items: opacity 0–1 (static).
    pub opacity: Option<f64>,
    /// Visual items: blur radius in design pixels (static).
    pub blur: Option<f64>,
}

impl McpTool for UpdateItem {
    type Input = Input;

    const NAME: &'static str = "update_item";

    const DESCRIPTION: &'static str = "Changes static properties of one item: text content and \
        style, shape size and colour, image fit, clip speed, fades and reframing, and placement \
        (position, scale, rotation, opacity, blur — static values replacing keyframes). Give only \
        the fields to change. Timing is changed by move_item and trim_item, animations by \
        set_keyframes and apply_animation_preset.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for UpdateItem {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        let item = scene.item(&input.item_id)?;
        let kind = scene.item_kind(&input.item_id)?;
        let mut edits = Vec::new();
        if let Some(patch) = patch(scene, kind, &input)? {
            edits.push(AgentEdit::UpdateItem {
                item,
                patch,
                expected_revision: input.expected_revision,
            });
        }
        for animation in placement(&input)? {
            edits.push(AgentEdit::SetProperty {
                item,
                animation,
                expected_revision: input.expected_revision,
            });
        }
        if edits.is_empty() {
            return Err(ToolError::invalid_arguments(
                "nothing to change: give at least one property",
            ));
        }
        Ok(Planned {
            edits,
            creates: None,
        })
    }
}

/// The names of the kind-specific fields `input` sets.
fn fields_set(input: &Input) -> Vec<&'static str> {
    let flags = [
        ("text", input.text.is_some()),
        ("font_family", input.font_family.is_some()),
        ("font_size", input.font_size.is_some()),
        ("font_weight", input.font_weight.is_some()),
        ("italic", input.italic.is_some()),
        ("color", input.color.is_some()),
        ("align", input.align.is_some()),
        ("max_width", input.max_width.is_some()),
        ("uppercase", input.uppercase.is_some()),
        ("stroke", input.stroke.is_some()),
        ("shadow", input.shadow.is_some()),
        ("background", input.background.is_some()),
        ("width", input.width.is_some()),
        ("height", input.height.is_some()),
        ("corner_radius", input.corner_radius.is_some()),
        ("fit", input.fit.is_some()),
        ("speed", input.speed.is_some()),
        ("fade_in", input.fade_in.is_some()),
        ("fade_out", input.fade_out.is_some()),
        ("reframe", input.reframe.is_some()),
    ];
    flags
        .into_iter()
        .filter_map(|(name, set)| set.then_some(name))
        .collect()
}

/// The fields each kind accepts.
fn accepted(kind: &str) -> &'static [&'static str] {
    match kind {
        "text" => &[
            "text",
            "font_family",
            "font_size",
            "font_weight",
            "italic",
            "color",
            "align",
            "max_width",
            "uppercase",
            "stroke",
            "shadow",
            "background",
        ],
        "shape" => &["color", "stroke", "width", "height", "corner_radius"],
        "image" => &["fit"],
        "media clip" => &["speed", "fade_in", "fade_out", "reframe"],
        _ => &[],
    }
}

/// The typed patch of the kind-specific fields, `None` when none is set.
fn patch(
    scene: &WriteScene<'_>,
    kind: &str,
    input: &Input,
) -> Result<Option<ItemPatch>, ToolError> {
    let set = fields_set(input);
    if set.is_empty() {
        return Ok(None);
    }
    let refused: Vec<&str> = set
        .iter()
        .copied()
        .filter(|field| !accepted(kind).contains(field))
        .collect();
    if !refused.is_empty() {
        return Err(ToolError::invalid_arguments(format!(
            "a {kind} has no {}; a {kind} accepts {}",
            refused.join(", "),
            accepted(kind).join(", ")
        )));
    }
    Ok(Some(match kind {
        "text" => ItemPatch::Text(text_patch(input)?),
        "shape" => ItemPatch::Shape(shape_patch(scene, input)?),
        "image" => ItemPatch::Image(ImagePatch {
            asset_id: None,
            fit: input.fit.map(FitInput::to_fit),
        }),
        _ => ItemPatch::MediaClip(clip_patch(scene, input)?),
    }))
}

fn text_patch(input: &Input) -> Result<TextPatch, ToolError> {
    Ok(TextPatch {
        content: input.text.clone(),
        font_family: input.font_family.clone(),
        font_weight: input.font_weight,
        italic: input.italic,
        font_size: input.font_size.map(DesignPixels),
        color: input
            .color
            .as_deref()
            .map(|text| color("color", text))
            .transpose()?,
        align: input.align.map(AlignInput::to_align),
        max_width: input.max_width.map(|width| Assign {
            value: (width > 0.0).then_some(DesignPixels(width)),
        }),
        text_case: input.uppercase.map(|upper| {
            if upper {
                TextCase::Uppercase
            } else {
                TextCase::Normal
            }
        }),
        stroke_enabled: input.stroke,
        shadow_enabled: input.shadow,
        background_enabled: input.background,
        ..TextPatch::default()
    })
}

fn shape_patch(scene: &WriteScene<'_>, input: &Input) -> Result<ShapePatch, ToolError> {
    let resizes = input.width.is_some() || input.height.is_some() || input.corner_radius.is_some();
    let geometry = if resizes {
        let current = scene
            .project
            .items()
            .find(|item| item.id.to_string() == input.item_id)
            .and_then(|item| match &item.kind {
                se_domain::ItemKind::Shape(shape) => Some(shape.geometry),
                _ => None,
            })
            .ok_or_else(|| {
                ToolError::invalid_arguments(
                    "a shape added in this batch is resized by add_shape (width, height)",
                )
            })?;
        Some(resized(
            current,
            input.width,
            input.height,
            input.corner_radius,
        ))
    } else {
        None
    };
    Ok(ShapePatch {
        geometry,
        fill: input
            .color
            .as_deref()
            .map(|text| color("color", text))
            .transpose()?,
        stroke_enabled: input.stroke,
        cover_frame: None,
    })
}

fn clip_patch(scene: &WriteScene<'_>, input: &Input) -> Result<MediaClipPatch, ToolError> {
    let length = |time: Option<&TimeInput>| time.map(|time| scene.time(time)).transpose();
    Ok(MediaClipPatch {
        speed: input.speed,
        fade_in: length(input.fade_in.as_ref())?,
        fade_out: length(input.fade_out.as_ref())?,
        reframe: input.reframe.map(|mode| match mode {
            ReframeInput::Fit => Reframe::Fit,
            ReframeInput::Fill => Reframe::Fill {
                focus: Animatable::constant(Vec2::splat(0.5)),
            },
            ReframeInput::Custom => Reframe::Custom,
        }),
        ..MediaClipPatch::default()
    })
}

/// Static placement values as property changes.
fn placement(input: &Input) -> Result<Vec<PropertyAnimation>, ToolError> {
    let mut animations = Vec::new();
    if let Some(position) = input.position {
        animations.push(PropertyAnimation::Position {
            value: Animatable::constant(position.to_vec2()),
        });
    }
    if let Some(scale) = &input.scale {
        let value = match scale {
            ValueInput::Number(factor) => Vec2::splat(*factor),
            ValueInput::Pair([x, y]) => Vec2::new(*x, *y),
            ValueInput::Point(point) => Vec2::new(point.x, point.y),
            ValueInput::Text(_) => {
                return Err(ToolError::invalid_arguments(
                    "scale: give a number (1 = natural) or [x, y]",
                ));
            }
        };
        animations.push(PropertyAnimation::Scale {
            value: Animatable::constant(value),
        });
    }
    if let Some(rotation) = input.rotation {
        animations.push(PropertyAnimation::Rotation {
            value: Animatable::constant(rotation),
        });
    }
    if let Some(opacity) = input.opacity {
        animations.push(PropertyAnimation::Opacity {
            value: Animatable::constant(opacity),
        });
    }
    if let Some(blur) = input.blur {
        animations.push(PropertyAnimation::Blur {
            value: Animatable::constant(DesignPixels(blur)),
        });
    }
    Ok(animations)
}
