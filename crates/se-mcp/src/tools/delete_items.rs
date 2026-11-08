//! `delete_items`: removes items (and their linked partners), leaving gaps.

use std::collections::BTreeMap;

use schemars::JsonSchema;
use se_app::agent_edits::AgentEdit;
use serde::Deserialize;

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    writes::{Planned, WriteScene, WriteTool, run},
};

/// The tool.
pub struct DeleteItems;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The items to remove (`item_ids`, or `$name` in `apply_batch`). A clip's linked sound or
    /// picture goes with it.
    pub item_ids: Vec<String>,
    /// The revision you read of some of them (`{"<item_id>": 3}`): refused with
    /// `REVISION_CONFLICT` if one changed since.
    #[serde(default)]
    pub expected_revisions: BTreeMap<String, u64>,
}

impl McpTool for DeleteItems {
    type Input = Input;

    const NAME: &'static str = "delete_items";

    const DESCRIPTION: &'static str = "Removes items from the timeline (a clip's linked partner \
        goes with it). The time they took stays empty: nothing moves. To close the gap, use \
        ripple_delete_range instead (only when the task's scope reaches the end of the sequence).";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for DeleteItems {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        if input.item_ids.is_empty() {
            return Err(ToolError::invalid_arguments(
                "item_ids: list at least one item",
            ));
        }
        if let Some(unknown) = input
            .expected_revisions
            .keys()
            .find(|id| !input.item_ids.contains(id))
        {
            return Err(ToolError::invalid_arguments(format!(
                "expected_revisions names {unknown}, which is not in item_ids"
            )));
        }
        let items = input
            .item_ids
            .iter()
            .map(|id| Ok((scene.item(id)?, input.expected_revisions.get(id).copied())))
            .collect::<Result<Vec<_>, ToolError>>()?;
        Ok(Planned::one(AgentEdit::DeleteItems { items }))
    }
}
