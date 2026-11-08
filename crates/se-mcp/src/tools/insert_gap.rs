//! `insert_gap`: opens empty time on tracks, so what follows moves right.

use schemars::JsonSchema;
use se_app::agent_edits::AgentEdit;
use serde::Deserialize;

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::TimeInput,
    writes::{Planned, WriteScene, WriteTool, run},
};

/// The tool.
pub struct InsertGap;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Where the empty time opens, in sequence time (an item crossing it is cut there).
    pub at: TimeInput,
    /// How long it is.
    pub duration: TimeInput,
    /// The tracks (ids or names); default: the task's tracks (every unlocked track when it may
    /// use them all). Tracks holding linked partners are added so sound and picture stay in sync.
    pub tracks: Option<Vec<String>>,
}

impl McpTool for InsertGap {
    type Input = Input;

    const NAME: &'static str = "insert_gap";

    const DESCRIPTION: &'static str = "Opens empty time at an instant: items crossing it are cut, \
        everything after it moves right. Works only when nothing outside the task's scope would \
        move (a scope reaching the end of the sequence, or the whole sequence).";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for InsertGap {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        let tracks = input
            .tracks
            .as_deref()
            .map(|tracks| scene.tracks(tracks))
            .transpose()?;
        Ok(Planned::one(AgentEdit::InsertGap {
            at: scene.time(&input.at)?,
            duration: scene.duration(&input.duration, "the gap")?,
            tracks,
        }))
    }
}
