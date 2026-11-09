//! `create_motion_template`: a motion template written by the agent (a TypeScript module with
//! `meta`, `params` and `render`), checked by the renderer, added to the project (prompt 24).

use schemars::JsonSchema;
use se_app::agent_edits::AgentEdit;
use serde::{Deserialize, Serialize};

use crate::{
    context::{ToolContext, find_sequence},
    output::{ToolError, ToolOutput},
    registry::McpTool,
    template_io::{TemplateSummary, find_template, frame_times, push_frames, template_summary},
    time_io::TimeOutput,
    writes::apply,
};

/// The tool.
pub struct CreateMotionTemplate;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The whole TypeScript module: `export const meta = { name, description, defaultDuration }`,
    /// `export const params = { … }` (JSON Schema subset) and `export function render(ctx, time,
    /// params)`. Rules and API: the motion templates instructions.
    pub source: String,
}

/// What the tool answers (then one picture per checked instant).
#[derive(Debug, Serialize)]
pub struct Output {
    /// The project's revision after the change.
    pub revision: u64,
    /// The new template.
    pub template: TemplateSummary,
    /// Longest render of one frame during the check, in milliseconds (budget: 50).
    pub slowest_frame_ms: f64,
    /// Instants of the pictures that follow (entrance, middle, exit), relative to an instance.
    pub frames: Vec<TimeOutput>,
    /// What to do next.
    pub next: &'static str,
}

impl McpTool for CreateMotionTemplate {
    type Input = Input;

    const NAME: &'static str = "create_motion_template";

    const DESCRIPTION: &'static str = "Adds a motion template written as a TypeScript module \
        (meta, params, render) to the project. The editor compiles it, runs it in a sandbox, \
        checks its time budget and determinism, and answers the template_id with pictures of the \
        entrance, middle and exit — or TEMPLATE_INVALID listing every problem with its line, to \
        fix and send again. A template is not on the timeline yet: then call \
        instantiate_template.";

    const READ_ONLY: bool = false;
    const DESTRUCTIVE: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        let project = context.project().await?;
        let format = find_sequence(&project, &context.agent, None)?.format;
        let checked = context
            .deps
            .template_checks
            .check(context.agent.project_id, &format, &input.source)
            .await?;
        let edit = AgentEdit::CreateTemplate {
            definition: Box::new(checked.definition.clone()),
            source: input.source,
        };
        let report = apply(&context, Self::NAME, &[edit], |_| None).await?;
        let change = report.templates.first().ok_or_else(|| {
            ToolError::new(
                crate::output::codes::INTERNAL,
                "the template was added without a report",
            )
        })?;
        let template = find_template(&report.project, &change.template_id.to_string())?;
        let mut output = ToolOutput::json(&Output {
            revision: report.revision,
            template: template_summary(&report.project, template),
            slowest_frame_ms: checked.slowest_frame_ms,
            frames: frame_times(&checked),
            next: "Look at the pictures; then place it with instantiate_template and check the \
                   result with get_frames.",
        })?;
        push_frames(&mut output, &checked);
        Ok(output)
    }
}
