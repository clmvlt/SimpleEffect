//! `apply_animation_preset`: an animation preset (fade, pop, slides…) expanded into keyframes
//! of an item, as an entrance, an exit or an emphasis.

use schemars::JsonSchema;
use se_app::agent_edits::{AgentEdit, PresetSpec};
use se_domain::AnimationPhase;
use serde::Deserialize;

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::TimeInput,
    writes::{Planned, WriteScene, WriteTool, run},
};

/// The tool.
pub struct ApplyAnimationPreset;

/// When the animation plays.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PhaseInput {
    /// An entrance, at the item's start.
    In,
    /// An exit, ending at the item's end.
    Out,
    /// An emphasis (presets with the `emphasis` phase), at `start`.
    Emphasis,
}

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The item (an `item_id`, or `$name` in `apply_batch`).
    pub item_id: String,
    /// A preset id from `list_animation_presets` (`fade`, `pop`, `slideFromLeft`, `pulse`…).
    pub preset: String,
    /// Entrance, exit or emphasis.
    pub phase: PhaseInput,
    /// Its length; default: the preset's.
    pub duration: Option<TimeInput>,
    /// Where it starts, **relative to the item's start**; default: at the start for `in` and
    /// `emphasis`, ending at the item's end for `out`.
    pub start: Option<TimeInput>,
    /// The revision you read; refused (`REVISION_CONFLICT`) if the item changed since.
    pub expected_revision: Option<u64>,
}

impl McpTool for ApplyAnimationPreset {
    type Input = Input;

    const NAME: &'static str = "apply_animation_preset";

    const DESCRIPTION: &'static str = "Animates an item with a preset from \
        list_animation_presets (entrance, exit or emphasis), written as ordinary keyframes the \
        user can edit. Keyframes of the same properties inside the animation's span are \
        replaced.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for ApplyAnimationPreset {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        Ok(Planned::one(AgentEdit::ApplyPreset {
            item: scene.item(&input.item_id)?,
            preset: PresetSpec {
                preset: input.preset.clone(),
                phase: match input.phase {
                    PhaseInput::In => AnimationPhase::In,
                    PhaseInput::Out => AnimationPhase::Out,
                    PhaseInput::Emphasis => AnimationPhase::Emphasis,
                },
                duration: input
                    .duration
                    .as_ref()
                    .map(|duration| scene.duration(duration, "the animation"))
                    .transpose()?,
            },
            start: input
                .start
                .as_ref()
                .map(|start| scene.time(start))
                .transpose()?,
            expected_revision: input.expected_revision,
        }))
    }
}
