//! `ripple_delete_range`: removes a time range from tracks and closes it, so what follows moves
//! left.

use schemars::JsonSchema;
use se_app::agent_edits::AgentEdit;
use serde::Deserialize;

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::RangeInput,
    writes::{Planned, WriteScene, WriteTool, run},
};

/// The tool.
pub struct RippleDeleteRange;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The time to remove, in sequence time (end excluded).
    pub range: RangeInput,
    /// The tracks (ids or names); default: the task's tracks (every unlocked track when it may
    /// use them all). Tracks holding linked partners are added so sound and picture stay in sync.
    pub tracks: Option<Vec<String>>,
}

impl McpTool for RippleDeleteRange {
    type Input = Input;

    const NAME: &'static str = "ripple_delete_range";

    const DESCRIPTION: &'static str = "Removes a time range from tracks and closes the gap: \
        items crossing it are cut, everything after it moves left. Because later items move, it \
        works only when nothing outside the task's scope would move (a scope reaching the end of \
        the sequence, or the whole sequence); otherwise use delete_items or trim_item.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for RippleDeleteRange {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        let range = input.range.to_range(scene.sequence.format.frame_rate)?;
        let tracks = input
            .tracks
            .as_deref()
            .map(|tracks| scene.tracks(tracks))
            .transpose()?;
        Ok(Planned::one(AgentEdit::RippleDelete { range, tracks }))
    }
}
