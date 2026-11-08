//! `apply_batch`: several write actions applied as **one** transaction — all of them, or none —
//! with references between them (`"as": "title"` names the item an action creates, `"$title"`
//! refers to it in later actions).

use std::collections::HashMap;

use rmcp::model::JsonObject;
use schemars::JsonSchema;
use se_app::agent_edits::AgentEdit;
use se_domain::ItemId;
use serde::{Deserialize, Serialize};

use super::{
    add_image::AddImage, add_marker::AddMarker, add_shape::AddShape, add_text::AddText,
    add_transition::AddTransition, apply_animation_preset::ApplyAnimationPreset,
    delete_items::DeleteItems, edit_caption_text::EditCaptionText,
    emphasize_caption_words::EmphasizeCaptionWords, insert_gap::InsertGap, move_item::MoveItem,
    ripple_delete_range::RippleDeleteRange, set_keyframes::SetKeyframes, set_volume::SetVolume,
    split_at::SplitAt, trim_item::TrimItem, update_caption_style::UpdateCaptionStyle,
    update_item::UpdateItem,
};
use crate::{
    context::{ToolContext, find_sequence},
    describe::ItemSummary,
    output::{ToolError, ToolOutput},
    registry::{McpTool, parse},
    writes::{NamedItem, Planned, WriteScene, WriteTool, apply, summaries},
};

/// Most actions in one batch.
pub const MAX_ACTIONS: usize = 50;

/// The tools a batch can run.
const BATCHABLE: [&str; 18] = [
    "add_text",
    "add_shape",
    "add_image",
    "update_item",
    "delete_items",
    "move_item",
    "trim_item",
    "split_at",
    "ripple_delete_range",
    "insert_gap",
    "set_keyframes",
    "apply_animation_preset",
    "set_volume",
    "add_transition",
    "add_marker",
    "update_caption_style",
    "edit_caption_text",
    "emphasize_caption_words",
];

/// The tool.
pub struct ApplyBatch;

/// One action of the batch.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ActionInput {
    /// A write tool: `add_text`, `add_shape`, `add_image`, `update_item`, `delete_items`, `move_item`,
    /// `trim_item`, `split_at`, `ripple_delete_range`, `insert_gap`, `set_keyframes`,
    /// `apply_animation_preset`, `set_volume`, `add_transition`, `add_marker`,
    /// `update_caption_style`, `edit_caption_text`, `emphasize_caption_words`.
    pub tool: String,
    /// Its arguments, exactly as for the tool itself.
    pub arguments: JsonObject,
    /// A name for the item this action creates (add_*, `split_at`), so later actions can refer to
    /// it as `"$name"` in their item fields.
    #[serde(rename = "as")]
    pub name: Option<String>,
}

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The actions, applied in order (1–50).
    pub actions: Vec<ActionInput>,
}

#[derive(Debug, Serialize)]
struct ActionOutput {
    tool: String,
    created: Vec<String>,
}

#[derive(Debug, Serialize)]
struct BatchOutput {
    revision: u64,
    actions: Vec<ActionOutput>,
    items: Vec<ItemSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    removed: Vec<String>,
}

impl McpTool for ApplyBatch {
    type Input = Input;

    const NAME: &'static str = "apply_batch";

    const DESCRIPTION: &'static str = "Applies several write actions as one change: all of them \
        or none (if one is refused, nothing changes and the error names it). Use it for a \
        coherent set of edits (a title with its background bar and animations, several cuts). \
        An action can name the item it creates (\"as\": \"title\") and later actions refer to it \
        as \"$title\".";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        if input.actions.is_empty() || input.actions.len() > MAX_ACTIONS {
            return Err(ToolError::invalid_arguments(format!(
                "give between 1 and {MAX_ACTIONS} actions (got {})",
                input.actions.len()
            )));
        }
        let project = context.project().await?;
        let sequence = find_sequence(&project, &context.agent, None)?;
        let mut names: HashMap<String, NamedItem> = HashMap::new();
        let mut edits: Vec<AgentEdit> = Vec::new();
        let mut action_of_edit: Vec<usize> = Vec::new();
        for (index, action) in input.actions.iter().enumerate() {
            let scene = WriteScene {
                project: &project,
                sequence,
                agent: &context.agent,
                names: &names,
            };
            let planned = plan_action(&action.tool, action.arguments.clone(), &scene)
                .map_err(|error| prefixed(error, index, &action.tool))?;
            if let Some(name) = &action.name {
                let created_item = named_item(&planned, edits.len())
                    .ok_or_else(|| prefixed(nothing_to_name(name), index, &action.tool))?;
                names.insert(name.clone(), created_item);
            }
            action_of_edit.extend(std::iter::repeat_n(index, planned.edits.len()));
            edits.extend(planned.edits);
        }
        let tools: Vec<&str> = input
            .actions
            .iter()
            .map(|action| action.tool.as_str())
            .collect();
        let report = apply(&context, Self::NAME, &edits, |edit| {
            let action = *action_of_edit.get(edit)?;
            Some(format!("action {} ({})", action + 1, tools.get(action)?))
        })
        .await?;
        let mut actions: Vec<ActionOutput> = tools
            .iter()
            .map(|tool| ActionOutput {
                tool: (*tool).to_owned(),
                created: Vec::new(),
            })
            .collect();
        for (edit, created) in report.created.iter().enumerate() {
            if let Some(action) = action_of_edit.get(edit).and_then(|&a| actions.get_mut(a)) {
                action.created.extend(created.iter().map(ItemId::to_string));
            }
        }
        ToolOutput::json(&BatchOutput {
            revision: report.revision,
            actions,
            items: summaries(&report.project, &report.changed),
            removed: report.removed.iter().map(ToString::to_string).collect(),
        })
    }
}

/// The item a planned action creates, for `"as"` (its edits start at `first_edit`).
fn named_item(planned: &Planned, first_edit: usize) -> Option<NamedItem> {
    let local = planned.creates?;
    let kind = match planned.edits.get(local)? {
        AgentEdit::AddItem(new) => Some(new.kind.label()),
        _ => None,
    };
    Some(NamedItem {
        edit: first_edit + local,
        kind,
    })
}

fn nothing_to_name(name: &str) -> ToolError {
    ToolError::invalid_arguments(format!(
        "\"as\": \"{name}\" names the item an action creates, but this action creates none \
         (only add_text, add_shape, add_image and split_at do)"
    ))
}

fn prefixed(mut error: ToolError, index: usize, tool: &str) -> ToolError {
    error.message = format!("action {} ({tool}): {}", index + 1, error.message);
    error
}

/// Whether `tool` can run in a batch.
#[must_use]
pub fn batchable(tool: &str) -> bool {
    BATCHABLE.contains(&tool)
}

/// Plans one action with the tool it names.
fn plan_action(
    tool: &str,
    arguments: JsonObject,
    scene: &WriteScene<'_>,
) -> Result<Planned, ToolError> {
    match tool {
        "add_text" => plan_with::<AddText>(arguments, scene),
        "add_shape" => plan_with::<AddShape>(arguments, scene),
        "add_image" => plan_with::<AddImage>(arguments, scene),
        "update_item" => plan_with::<UpdateItem>(arguments, scene),
        "delete_items" => plan_with::<DeleteItems>(arguments, scene),
        "move_item" => plan_with::<MoveItem>(arguments, scene),
        "trim_item" => plan_with::<TrimItem>(arguments, scene),
        "split_at" => plan_with::<SplitAt>(arguments, scene),
        "ripple_delete_range" => plan_with::<RippleDeleteRange>(arguments, scene),
        "insert_gap" => plan_with::<InsertGap>(arguments, scene),
        "set_keyframes" => plan_with::<SetKeyframes>(arguments, scene),
        "apply_animation_preset" => plan_with::<ApplyAnimationPreset>(arguments, scene),
        "set_volume" => plan_with::<SetVolume>(arguments, scene),
        "add_transition" => plan_with::<AddTransition>(arguments, scene),
        "add_marker" => plan_with::<AddMarker>(arguments, scene),
        "update_caption_style" => plan_with::<UpdateCaptionStyle>(arguments, scene),
        "edit_caption_text" => plan_with::<EditCaptionText>(arguments, scene),
        "emphasize_caption_words" => plan_with::<EmphasizeCaptionWords>(arguments, scene),
        _ => Err(ToolError::invalid_arguments(format!(
            "{tool} cannot run in a batch; batchable tools are {}",
            BATCHABLE.join(", ")
        ))),
    }
}

fn plan_with<T: WriteTool>(
    arguments: JsonObject,
    scene: &WriteScene<'_>,
) -> Result<Planned, ToolError> {
    T::plan(scene, parse::<T::Input>(arguments)?)
}
