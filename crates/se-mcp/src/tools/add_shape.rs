//! `add_shape`: a rectangle, rounded rectangle, ellipse or line over a time range of the task's
//! sequence (bars behind titles, highlights, dividers), with optional animations.

use schemars::JsonSchema;
use se_app::agent_edits::{AgentEdit, NewAgentItem};
use se_domain::{Animatable, DesignPixels, ItemKind, ShapeGeometry, ShapeItem, ShapeKind};
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
pub struct AddShape;

/// The kind of shape.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ShapeInput {
    /// Sharp corners (600 × 338 px by default).
    Rectangle,
    /// Rounded corners (radius 32 px by default).
    RoundedRectangle,
    /// An ellipse (400 × 400 px by default).
    Ellipse,
    /// A straight line (600 px long, 8 px thick by default).
    Line,
}

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The shape.
    pub shape: ShapeInput,
    /// When it appears, in sequence time.
    pub start: TimeInput,
    /// When it disappears (excluded). Give `end` or `duration`.
    pub end: Option<TimeInput>,
    /// How long it stays. Give `end` or `duration`.
    pub duration: Option<TimeInput>,
    /// Where its centre sits; default "center".
    pub position: Option<PositionInput>,
    /// Width in design pixels (a line's length).
    pub width: Option<f64>,
    /// Height in design pixels (a line's thickness).
    pub height: Option<f64>,
    /// Corner radius in design pixels (rectangles).
    pub corner_radius: Option<f64>,
    /// Fill colour `#rrggbb[aa]`; default white.
    pub color: Option<String>,
    /// An outline.
    pub stroke: Option<StrokeInput>,
    /// Stretch it over the whole frame (backgrounds, colour washes); default false.
    pub cover_frame: Option<bool>,
    /// Opacity from 0 to 1; default 1.
    pub opacity: Option<f64>,
    /// A video track (id or name); default: the highest free track of the task.
    pub track: Option<String>,
    /// Entrance animation (preset `in` phase).
    pub animation_in: Option<AnimationInput>,
    /// Exit animation (preset `out` phase).
    pub animation_out: Option<AnimationInput>,
}

impl McpTool for AddShape {
    type Input = Input;

    const NAME: &'static str = "add_shape";

    const DESCRIPTION: &'static str = "Adds an editable shape (rectangle, rounded rectangle, \
        ellipse, line) over a time range of the task's sequence, with its size, colour, outline, \
        opacity and optional entrance/exit animations. Answers the new item_id.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for AddShape {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        let range = span(
            scene,
            &input.start,
            input.end.as_ref(),
            input.duration.as_ref(),
        )?;
        let mut shape = ShapeItem::new(shape_kind(input.shape));
        shape.geometry = geometry(shape.geometry, &input);
        if let Some(text) = &input.color {
            shape.fill = Animatable::constant(color("color", text)?);
        }
        if let Some(stroke) = &input.stroke {
            shape.stroke.enabled = true;
            if let Some(text) = &stroke.color {
                shape.stroke.color = Animatable::constant(color("stroke.color", text)?);
            }
            if let Some(width) = stroke.width {
                shape.stroke.width = Animatable::constant(DesignPixels(width));
            }
        }
        shape.cover_frame = input.cover_frame.unwrap_or(false);
        if let Some(position) = input.position {
            shape.transform.position = Animatable::constant(position.to_vec2());
        }
        if let Some(opacity) = input.opacity {
            shape.transform.opacity = Animatable::constant(opacity);
        }
        let new = NewAgentItem {
            kind: ItemKind::Shape(shape),
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

const fn shape_kind(shape: ShapeInput) -> ShapeKind {
    match shape {
        ShapeInput::Rectangle => ShapeKind::Rectangle,
        ShapeInput::RoundedRectangle => ShapeKind::RoundedRectangle,
        ShapeInput::Ellipse => ShapeKind::Ellipse,
        ShapeInput::Line => ShapeKind::Line,
    }
}

/// `default` resized by the input's sizes (absent sizes keep the default).
#[must_use]
pub fn geometry(default: ShapeGeometry, input: &Input) -> ShapeGeometry {
    resized(default, input.width, input.height, input.corner_radius)
}

/// `geometry` with the given width, height and corner radius (absent ones kept).
#[must_use]
pub fn resized(
    geometry: ShapeGeometry,
    width: Option<f64>,
    height: Option<f64>,
    radius: Option<f64>,
) -> ShapeGeometry {
    let pick = |value: Option<f64>, current: DesignPixels| value.map_or(current, DesignPixels);
    match geometry {
        ShapeGeometry::Rectangle {
            width: w,
            height: h,
            corner_radius,
        } => ShapeGeometry::Rectangle {
            width: pick(width, w),
            height: pick(height, h),
            corner_radius: pick(radius, corner_radius),
        },
        ShapeGeometry::Ellipse {
            width: w,
            height: h,
        } => ShapeGeometry::Ellipse {
            width: pick(width, w),
            height: pick(height, h),
        },
        ShapeGeometry::Line { length, thickness } => ShapeGeometry::Line {
            length: pick(width, length),
            thickness: pick(height, thickness),
        },
    }
}
