//! `move_item`: moves an item in time and/or to another track; its linked partners move by the
//! same amount on their own tracks.

use schemars::JsonSchema;
use se_app::agent_edits::{AgentEdit, ItemRef};
use se_domain::Flicks;
use serde::Deserialize;

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::TimeInput,
    writes::{Planned, WriteScene, WriteTool, run},
};

/// The tool.
pub struct MoveItem;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The item (an `item_id`, or `$name` in `apply_batch`).
    pub item_id: String,
    /// Its new start, in sequence time; default: unchanged.
    pub start: Option<TimeInput>,
    /// Its new track (id or name, same kind); default: unchanged.
    pub track: Option<String>,
    /// The revision you read; refused (`REVISION_CONFLICT`) if the item changed since.
    pub expected_revision: Option<u64>,
}

impl McpTool for MoveItem {
    type Input = Input;

    const NAME: &'static str = "move_item";

    const DESCRIPTION: &'static str = "Moves an item to a new start and/or another track of the \
        same kind; a clip's linked sound or picture moves by the same amount. Refused if it would \
        overlap another item (trim or move that one first).";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for MoveItem {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        if input.start.is_none() && input.track.is_none() {
            return Err(ToolError::invalid_arguments(
                "give a new `start`, a new `track`, or both",
            ));
        }
        let item = scene.item(&input.item_id)?;
        let start = match &input.start {
            Some(start) => scene.time(start)?,
            None => current_start(scene, item)?,
        };
        let track = input
            .track
            .as_ref()
            .map(|track| {
                scene
                    .tracks(std::slice::from_ref(track))
                    .map(|tracks| tracks[0])
            })
            .transpose()?;
        Ok(Planned::one(AgentEdit::MoveItem {
            item,
            track,
            start,
            expected_revision: input.expected_revision,
        }))
    }
}

/// The start of an existing item (a move to another track keeps it).
fn current_start(scene: &WriteScene<'_>, item: ItemRef) -> Result<Flicks, ToolError> {
    let ItemRef::Id(id) = item else {
        return Err(ToolError::invalid_arguments(
            "give `start` when moving an item added in this batch",
        ));
    };
    scene
        .project
        .items()
        .find(|candidate| candidate.id == id)
        .map(|found| found.timeline_range.start())
        .ok_or_else(|| {
            ToolError::new(
                crate::output::codes::ITEM_NOT_FOUND,
                format!("no item {id}: list items with get_timeline"),
            )
        })
}
