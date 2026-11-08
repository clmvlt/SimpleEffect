//! `get_task_changes`: what the calling task changed so far (its open changeset), for the agent
//! to check and summarise its work.

use schemars::JsonSchema;
use se_app::agent_edits::TaskChanges;
use serde::{Deserialize, Serialize};

use crate::{
    context::ToolContext,
    describe::ItemSummary,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    writes::summaries,
};

/// The tool.
pub struct GetTaskChanges;

/// Arguments: none.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {}

#[derive(Debug, Serialize)]
struct ChangesOutput {
    /// The task's open changeset (`null` while it changed nothing).
    changeset_id: Option<String>,
    /// How many write calls it committed.
    edits: usize,
    /// Items it created that still exist.
    created: Vec<ItemSummary>,
    /// Items that existed before, changed by it.
    changed: Vec<ItemSummary>,
    /// Items it removed.
    removed: Vec<String>,
}

impl McpTool for GetTaskChanges {
    type Input = Input;

    const NAME: &'static str = "get_task_changes";

    const DESCRIPTION: &'static str = "What this task changed so far: items it created, changed \
        and removed (all its write calls, which the user can revert at once). Use it to check \
        your work before summarising it.";

    async fn call(context: ToolContext, _input: Input) -> Result<ToolOutput, ToolError> {
        let changes = context.deps.edits.changes(&context.agent).await?;
        ToolOutput::json(&output(&changes))
    }
}

fn output(changes: &TaskChanges) -> ChangesOutput {
    ChangesOutput {
        changeset_id: changes.changeset_id.map(|id| id.to_string()),
        edits: changes.edits,
        created: summaries(&changes.project, &changes.created),
        changed: summaries(&changes.project, &changes.changed),
        removed: changes.removed.iter().map(ToString::to_string).collect(),
    }
}
