//! `instantiate_template`: places a motion template of the project over a time range of the
//! task's sequence, with parameter values, placement and animations (prompt 24).

use schemars::JsonSchema;
use se_app::{
    agent_edits::{AgentEdit, NewAgentItem},
    template_library::instance_values,
};
use se_domain::{Animatable, ItemKind, TemplateItem, Transform2D, Vec2};
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    template_io::{find_template, param_values, template_range},
    time_io::TimeInput,
    write_inputs::{AnimationInput, PositionInput, presets},
    writes::{Planned, WriteScene, WriteTool, run},
};

/// The tool.
pub struct InstantiateTemplate;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The template (a `template_id` from `create_motion_template` or `list_templates`).
    pub template_id: String,
    /// When it starts, in sequence time.
    pub start: TimeInput,
    /// When it ends (excluded). Give `end` or `duration`, or neither for the template's default
    /// duration.
    pub end: Option<TimeInput>,
    /// How long it lasts.
    pub duration: Option<TimeInput>,
    /// Parameter values by name (the others take their defaults). Numbers and colours also take
    /// `{"keyframes": [{"time": 0, "value": …, "easing"?: "ease_out"}]}` (times relative to the
    /// item's start); assets take an image `asset_id`.
    pub params: Option<Map<String, Value>>,
    /// A video track (id or name); default: the highest free track of the task.
    pub track: Option<String>,
    /// Where the template's frame is centred; default "center" (the template draws over the
    /// whole frame: keep the centre unless you move the whole graphic).
    pub position: Option<PositionInput>,
    /// Scale of the whole frame (1 = as drawn).
    pub scale: Option<f64>,
    /// Opacity from 0 to 1; default 1.
    pub opacity: Option<f64>,
    /// Entrance animation of the whole frame (preset `in` phase); usually the template animates
    /// itself.
    pub animation_in: Option<AnimationInput>,
    /// Exit animation of the whole frame (preset `out` phase).
    pub animation_out: Option<AnimationInput>,
}

impl McpTool for InstantiateTemplate {
    type Input = Input;

    const NAME: &'static str = "instantiate_template";

    const DESCRIPTION: &'static str = "Places a motion template of the project on the task's \
        sequence over a time range (default length: the template's), with parameter values \
        (texts, numbers, colours… — the user edits them later in the inspector; parameters bound \
        to a brand kit role start with the project's kit value), and optional placement and \
        animations of the whole frame. Answers the new item_id; check it with get_frames.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for InstantiateTemplate {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        let template = find_template(scene.project, &input.template_id)?;
        let range = template_range(
            scene,
            &input.start,
            input.end.as_ref(),
            input.duration.as_ref(),
            template.default_duration,
        )?;
        let explicit = param_values(scene, template, &input.params.unwrap_or_default())?
            .into_iter()
            .filter_map(|(name, value)| value.map(|value| (name, value)))
            .collect();
        // Parameters bound to a brand kit role start with the project's kit values (prompt 25).
        let params = instance_values(scene.project, template, explicit);
        let mut transform = Transform2D::default();
        if let Some(position) = input.position {
            transform.position = Animatable::constant(position.to_vec2());
        }
        if let Some(scale) = input.scale {
            transform.scale = Animatable::constant(Vec2::splat(scale));
        }
        if let Some(opacity) = input.opacity {
            transform.opacity = Animatable::constant(opacity);
        }
        let new = NewAgentItem {
            kind: ItemKind::MotionTemplate(TemplateItem {
                template_id: template.id,
                params,
                transform,
            }),
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
