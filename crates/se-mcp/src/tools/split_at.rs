//! `split_at`: cuts items in two at an instant (linked partners crossing it too).

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
pub struct SplitAt;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The instant, in sequence time.
    pub at: TimeInput,
    /// The items to cut (`item_ids` or `$name`); default: every item crossing `at` on the task's
    /// unlocked tracks.
    pub item_ids: Option<Vec<String>>,
}

impl McpTool for SplitAt {
    type Input = Input;

    const NAME: &'static str = "split_at";

    const DESCRIPTION: &'static str = "Cuts items in two at an instant (every item crossing it on \
        the task's tracks, or the ones listed); linked partners crossing it are cut too and the \
        right parts stay linked. Answers the right parts' ids in `created`. Prefer cutting in \
        silences (get_audio_analysis).";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for SplitAt {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        let items = input
            .item_ids
            .as_ref()
            .map(|ids| ids.iter().map(|id| scene.item(id)).collect())
            .transpose()?;
        Ok(Planned {
            edits: vec![AgentEdit::Split {
                items,
                at: scene.time(&input.at)?,
            }],
            creates: Some(0),
        })
    }
}
