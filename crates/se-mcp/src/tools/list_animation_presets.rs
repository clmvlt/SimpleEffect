//! `list_animation_presets`: the animation presets items can get (fades, slides, pop…): ids,
//! phases, default durations and the properties they animate.

use schemars::JsonSchema;
use se_app::project_service::animation_preset_infos;
use serde::{Deserialize, Serialize};

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::TimeOutput,
};

/// The tool.
pub struct ListAnimationPresets;

/// No arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {}

#[derive(Debug, Serialize)]
struct Listing {
    presets: Vec<PresetSummary>,
}

#[derive(Debug, Serialize)]
struct PresetSummary {
    preset_id: String,
    name: String,
    description: String,
    /// `in` (entrance), `out` (exit) and/or `emphasis`.
    phases: Vec<serde_json::Value>,
    default_duration: TimeOutput,
    /// Properties it animates (`opacity`, `position`…).
    properties: Vec<&'static str>,
}

impl McpTool for ListAnimationPresets {
    type Input = Input;

    const NAME: &'static str = "list_animation_presets";

    const DESCRIPTION: &'static str = "The animation presets (fade, slides, pop, blur, \
        typewriter, bounce, shake, pulse…): id, name, description, phases (in, out, emphasis), \
        default duration and the properties each one animates.";

    fn call(
        _context: ToolContext,
        _input: Input,
    ) -> impl Future<Output = Result<ToolOutput, ToolError>> + Send {
        std::future::ready(Self::answer())
    }
}

impl ListAnimationPresets {
    fn answer() -> Result<ToolOutput, ToolError> {
        let presets = animation_preset_infos()?
            .into_iter()
            .map(|preset| PresetSummary {
                preset_id: preset.id,
                name: preset.name,
                description: preset.description,
                phases: preset
                    .phases
                    .iter()
                    .filter_map(|phase| serde_json::to_value(phase).ok())
                    .collect(),
                default_duration: preset.default_duration.into(),
                properties: preset
                    .properties
                    .iter()
                    .map(|property| property.name())
                    .collect(),
            })
            .collect();
        ToolOutput::json(&Listing { presets })
    }
}
