//! `set_volume`: the gain of a sound clip — a static level or an automation curve — and its
//! fades, in one edit.

use schemars::JsonSchema;
use se_app::agent_edits::AgentEdit;
use se_domain::{AnimatableProperty, ItemPatch, MediaClipPatch};
use serde::Deserialize;
use serde_json::{Value, json};

use super::set_keyframes::animation;
use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::TimeInput,
    write_inputs::EasingInput,
    writes::{Planned, WriteScene, WriteTool, run},
};

/// The tool.
pub struct SetVolume;

/// A point of a volume automation.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VolumePoint {
    /// Time **relative to the clip's start**.
    pub time: TimeInput,
    /// Gain in dB (−96 to 24; 0 = unchanged, −6 ≈ half as loud).
    pub gain_db: f64,
    /// How the gain travels to the next point; default linear.
    pub easing: Option<EasingInput>,
}

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The clip (an `item_id`, or `$name` in `apply_batch`).
    pub item_id: String,
    /// A static gain in dB (−96 to 24), replacing any automation.
    pub gain_db: Option<f64>,
    /// An automation curve (ducking under a voice, swells), replacing the gain.
    pub automation: Option<Vec<VolumePoint>>,
    /// Fade-in length (0 removes it). Short fades (0.01–0.05 s) avoid clicks at cuts.
    pub fade_in: Option<TimeInput>,
    /// Fade-out length (0 removes it).
    pub fade_out: Option<TimeInput>,
    /// The revision you read; refused (`REVISION_CONFLICT`) if the item changed since.
    pub expected_revision: Option<u64>,
}

impl McpTool for SetVolume {
    type Input = Input;

    const NAME: &'static str = "set_volume";

    const DESCRIPTION: &'static str = "Sets the volume of a clip with sound: a static gain in dB \
        or an automation curve (times relative to the clip's start), and its fade-in and \
        fade-out lengths, in one step.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for SetVolume {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        let item = scene.item(&input.item_id)?;
        let mut edits = Vec::new();
        let gain = match (&input.gain_db, &input.automation) {
            (Some(gain), None) => Some(json!({ "kind": "static", "value": gain })),
            (None, Some(points)) if !points.is_empty() => Some(automation(scene, points)?),
            (None, None) => None,
            _ => {
                return Err(ToolError::invalid_arguments(
                    "give either `gain_db` or `automation` (at least one point), not both",
                ));
            }
        };
        if let Some(value) = gain {
            edits.push(AgentEdit::SetProperty {
                item,
                animation: animation(AnimatableProperty::VolumeDb, &value)?,
                expected_revision: input.expected_revision,
            });
        }
        if input.fade_in.is_some() || input.fade_out.is_some() {
            let length = |time: Option<&TimeInput>| time.map(|time| scene.time(time)).transpose();
            edits.push(AgentEdit::UpdateItem {
                item,
                patch: ItemPatch::MediaClip(MediaClipPatch {
                    fade_in: length(input.fade_in.as_ref())?,
                    fade_out: length(input.fade_out.as_ref())?,
                    ..MediaClipPatch::default()
                }),
                expected_revision: input.expected_revision,
            });
        }
        if edits.is_empty() {
            return Err(ToolError::invalid_arguments(
                "nothing to change: give gain_db, automation, fade_in or fade_out",
            ));
        }
        Ok(Planned {
            edits,
            creates: None,
        })
    }
}

fn automation(scene: &WriteScene<'_>, points: &[VolumePoint]) -> Result<Value, ToolError> {
    let keyframes = points
        .iter()
        .map(|point| {
            let easing = point
                .easing
                .map_or(se_domain::Easing::Linear, EasingInput::to_easing);
            Ok(json!({
                "time": scene.time(&point.time)?.get(),
                "value": point.gain_db,
                "easing": serde_json::to_value(easing)
                    .map_err(|error| ToolError::invalid_arguments(error.to_string()))?,
            }))
        })
        .collect::<Result<Vec<Value>, ToolError>>()?;
    Ok(json!({ "kind": "keyframed", "keyframes": keyframes }))
}
