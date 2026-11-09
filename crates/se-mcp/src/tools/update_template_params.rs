//! `update_template_params`: parameter values of a motion template instance (prompt 24).

use schemars::JsonSchema;
use se_app::agent_edits::AgentEdit;
use se_domain::{ItemKind, ItemPatch, TemplatePatch};
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    template_io::{find_template, param_values},
    writes::{Planned, WriteScene, WriteTool, run},
};

/// The tool.
pub struct UpdateTemplateParams;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The instance (an `item_id` of kind `motion_template`, or `$name` in `apply_batch`).
    pub item_id: String,
    /// Values by parameter name; `null` resets one to its default. Numbers and colours also take
    /// `{"keyframes": [{"time": 0, "value": …, "easing"?}]}` (times relative to the item's
    /// start).
    pub params: Map<String, Value>,
    /// The item's revision you read (refused with `REVISION_CONFLICT` when it changed since).
    pub expected_revision: Option<u64>,
}

impl McpTool for UpdateTemplateParams {
    type Input = Input;

    const NAME: &'static str = "update_template_params";

    const DESCRIPTION: &'static str = "Changes parameter values of a motion template instance \
        (texts, numbers, colours, options, fonts, images; numbers and colours may be keyframed); \
        null resets a value to the template's default. The template's code is unchanged \
        (update_motion_template changes it).";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for UpdateTemplateParams {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        if input.params.is_empty() {
            return Err(ToolError::invalid_arguments(
                "give at least one parameter in `params`",
            ));
        }
        let item = scene.item(&input.item_id)?;
        let item_id = match item {
            se_app::agent_edits::ItemRef::Id(id) => id,
            se_app::agent_edits::ItemRef::Created { .. } => {
                return Err(ToolError::invalid_arguments(
                    "give the values of a template created in this batch to instantiate_template \
                     itself (its `params`)",
                ));
            }
        };
        let found = scene
            .project
            .items()
            .find(|candidate| candidate.id == item_id)
            .ok_or_else(|| {
                ToolError::new(
                    crate::output::codes::ITEM_NOT_FOUND,
                    format!("no item {item_id}: list items with get_timeline"),
                )
            })?;
        let ItemKind::MotionTemplate(instance) = &found.kind else {
            return Err(ToolError::invalid_arguments(format!(
                "item {item_id} is a {}, not a motion template instance",
                found.kind.label()
            )));
        };
        let template = find_template(scene.project, &instance.template_id.to_string())?;
        let params = param_values(scene, template, &input.params)?;
        Ok(Planned::one(AgentEdit::UpdateItem {
            item,
            patch: ItemPatch::MotionTemplate(TemplatePatch {
                template_id: None,
                params,
            }),
            expected_revision: input.expected_revision,
        }))
    }
}
