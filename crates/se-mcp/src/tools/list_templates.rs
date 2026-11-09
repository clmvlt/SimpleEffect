//! `list_templates`: the motion templates of the project (prompt 24), or one of them with its
//! source for `update_motion_template`. The global library and brand kits come with prompt 25.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    template_io::{TemplateSummary, find_template, template_summary},
};

/// The tool.
pub struct ListTemplates;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// One template (`template_id`): answers its full source too, to write a new version.
    pub template_id: Option<String>,
}

/// Every template of the project.
#[derive(Debug, Serialize)]
struct Listing {
    templates: Vec<TemplateSummary>,
}

/// One template with its source.
#[derive(Debug, Serialize)]
struct Detail {
    template: TemplateSummary,
    source: String,
}

impl McpTool for ListTemplates {
    type Input = Input;

    const NAME: &'static str = "list_templates";

    const DESCRIPTION: &'static str = "The motion templates of the project (id, name, version, \
        default duration, parameters with types, defaults and ranges, number of instances). With \
        template_id: that template and its full TypeScript source, to write a new version with \
        update_motion_template.";

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        let project = context.project().await?;
        match input.template_id {
            Some(reference) => {
                let template = find_template(&project, &reference)?;
                ToolOutput::json(&Detail {
                    template: template_summary(&project, template),
                    source: template.source.clone(),
                })
            }
            None => ToolOutput::json(&Listing {
                templates: project
                    .templates
                    .iter()
                    .map(|template| template_summary(&project, template))
                    .collect(),
            }),
        }
    }
}
