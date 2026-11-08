//! `add_marker`: a labelled marker at an instant of the task's sequence (a beat, a chapter, a
//! note for the user).

use schemars::JsonSchema;
use se_app::agent_edits::AgentEdit;
use se_domain::MarkerColor;
use serde::Deserialize;

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::TimeInput,
    writes::{Planned, WriteScene, WriteTool, run},
};

/// The tool.
pub struct AddMarker;

/// Marker colours.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ColorInput {
    /// Default.
    Green,
    /// Red.
    Red,
    /// Purple.
    Purple,
    /// Orange.
    Orange,
    /// Yellow.
    Yellow,
    /// White.
    White,
    /// Blue.
    Blue,
    /// Cyan.
    Cyan,
}

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Where, in sequence time.
    pub time: TimeInput,
    /// Its label; default empty.
    #[serde(default)]
    pub label: String,
    /// Its colour; default green.
    pub color: Option<ColorInput>,
}

impl McpTool for AddMarker {
    type Input = Input;

    const NAME: &'static str = "add_marker";

    const DESCRIPTION: &'static str = "Adds a labelled marker at an instant of the task's \
        sequence (beats, chapters, notes for the user).";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for AddMarker {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        let color = match input.color.unwrap_or(ColorInput::Green) {
            ColorInput::Green => MarkerColor::Green,
            ColorInput::Red => MarkerColor::Red,
            ColorInput::Purple => MarkerColor::Purple,
            ColorInput::Orange => MarkerColor::Orange,
            ColorInput::Yellow => MarkerColor::Yellow,
            ColorInput::White => MarkerColor::White,
            ColorInput::Blue => MarkerColor::Blue,
            ColorInput::Cyan => MarkerColor::Cyan,
        };
        Ok(Planned::one(AgentEdit::AddMarker {
            time: scene.time(&input.time)?,
            label: input.label,
            color,
        }))
    }
}
