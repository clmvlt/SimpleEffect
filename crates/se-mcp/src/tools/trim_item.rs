//! `trim_item`: moves the start or end edge of an item (the media played follows); linked
//! partners whose edge sits at the same instant follow.

use schemars::JsonSchema;
use se_app::agent_edits::AgentEdit;
use se_domain::ItemEdge;
use serde::Deserialize;

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::TimeInput,
    writes::{Planned, WriteScene, WriteTool, run},
};

/// The tool.
pub struct TrimItem;

/// Which edge of an item.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EdgeInput {
    /// Its first instant.
    Start,
    /// Its end.
    End,
}

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The item (an `item_id`, or `$name` in `apply_batch`).
    pub item_id: String,
    /// The edge to move.
    pub edge: EdgeInput,
    /// Where the edge goes, in sequence time.
    pub to: TimeInput,
    /// The revision you read; refused (`REVISION_CONFLICT`) if the item changed since.
    pub expected_revision: Option<u64>,
}

impl McpTool for TrimItem {
    type Input = Input;

    const NAME: &'static str = "trim_item";

    const DESCRIPTION: &'static str = "Moves the start or end edge of an item, in sequence time \
        (a clip plays less or more of its media; nothing else moves). A clip's linked partner \
        whose edge sits at the same instant follows. Refused beyond the media or onto a \
        neighbour.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for TrimItem {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        Ok(Planned::one(AgentEdit::TrimItem {
            item: scene.item(&input.item_id)?,
            edge: match input.edge {
                EdgeInput::Start => ItemEdge::Start,
                EdgeInput::End => ItemEdge::End,
            },
            to: scene.time(&input.to)?,
            expected_revision: input.expected_revision,
        }))
    }
}
