//! `update_motion_template`: a new version of a motion template's module (prompt 24). Instances
//! in the task's scope render it; when other instances lie outside the scope, the new version is
//! a copy only the scope's instances switch to (copy on write).

use schemars::JsonSchema;
use se_app::agent_edits::AgentEdit;
use serde::{Deserialize, Serialize};

use crate::{
    context::{ToolContext, find_sequence},
    output::{ToolError, ToolOutput, codes},
    registry::McpTool,
    template_io::{TemplateSummary, find_template, frame_times, push_frames, template_summary},
    time_io::TimeOutput,
    writes::apply,
};

/// The tool.
pub struct UpdateMotionTemplate;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The template (a `template_id` from `list_templates` or `create_motion_template`).
    pub template_id: String,
    /// The whole new module (read the current one with `list_templates {"template_id"}`).
    pub source: String,
}

/// Values of an instance that fell back to the new defaults.
#[derive(Debug, Serialize)]
pub struct ResetValues {
    /// The instance.
    pub item_id: String,
    /// The parameters reset (gone, another type, or out of the new range).
    pub params: Vec<String>,
}

/// What the tool answers (then one picture per checked instant).
#[derive(Debug, Serialize)]
pub struct Output {
    /// The project's revision after the change.
    pub revision: u64,
    /// The template holding the new version (a new id when copied).
    pub template: TemplateSummary,
    /// The template copied because some of its instances lie outside the task's scope (they
    /// keep it); absent when changed in place.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copied_from: Option<String>,
    /// Instances now rendering the new version.
    pub instances: Vec<String>,
    /// Instances whose values were reset.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reset: Vec<ResetValues>,
    /// Longest render of one frame during the check, in milliseconds (budget: 50).
    pub slowest_frame_ms: f64,
    /// Instants of the pictures that follow (entrance, middle, exit), relative to an instance.
    pub frames: Vec<TimeOutput>,
}

impl McpTool for UpdateMotionTemplate {
    type Input = Input;

    const NAME: &'static str = "update_motion_template";

    const DESCRIPTION: &'static str = "Replaces the module of a motion template with a new \
        version, checked like create_motion_template (pictures, or TEMPLATE_INVALID with every \
        problem). Instances in the task's scope render the new version; if others lie outside \
        the scope, the new version is a copy (copied_from) only the scope's instances use. \
        Values the new parameters refuse fall back to their defaults (reset).";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        let project = context.project().await?;
        let template_id = find_template(&project, &input.template_id)?.id;
        let format = find_sequence(&project, &context.agent, None)?.format;
        let checked = context
            .deps
            .template_checks
            .check(context.agent.project_id, &format, &input.source)
            .await?;
        let edit = AgentEdit::UpdateTemplate {
            template: template_id,
            definition: Box::new(checked.definition.clone()),
            source: input.source,
        };
        let report = apply(&context, Self::NAME, &[edit], |_| None).await?;
        let change = report.templates.first().ok_or_else(|| {
            ToolError::new(codes::INTERNAL, "the template changed without a report")
        })?;
        let template = find_template(&report.project, &change.template_id.to_string())?;
        let mut output = ToolOutput::json(&Output {
            revision: report.revision,
            template: template_summary(&report.project, template),
            copied_from: change.copied_from.map(|id| id.to_string()),
            instances: change.instances.iter().map(ToString::to_string).collect(),
            reset: change
                .reset
                .iter()
                .map(|(item, params)| ResetValues {
                    item_id: item.to_string(),
                    params: params.clone(),
                })
                .collect(),
            slowest_frame_ms: checked.slowest_frame_ms,
            frames: frame_times(&checked),
        })?;
        push_frames(&mut output, &checked);
        Ok(output)
    }
}
