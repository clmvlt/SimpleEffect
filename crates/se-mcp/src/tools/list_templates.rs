//! `list_templates`: the motion templates of the project and the library. Motion templates
//! arrive with prompt 24; until then the tool says plainly that they are not available.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
};

/// The tool.
pub struct ListTemplates;

/// No arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {}

#[derive(Debug, Serialize)]
struct Unavailable {
    available: bool,
    templates: [(); 0],
    message: &'static str,
}

impl McpTool for ListTemplates {
    type Input = Input;

    const NAME: &'static str = "list_templates";

    const DESCRIPTION: &'static str = "The motion templates available (animated titles, lower \
        thirds…). Not available yet in this version of SimpleEffect: it answers \
        available=false with no templates.";

    fn call(
        _context: ToolContext,
        _input: Input,
    ) -> impl Future<Output = Result<ToolOutput, ToolError>> + Send {
        std::future::ready(Self::answer())
    }
}

impl ListTemplates {
    fn answer() -> Result<ToolOutput, ToolError> {
        // TODO(prompt-24): list project and library templates with their parameters.
        ToolOutput::json(&Unavailable {
            available: false,
            templates: [],
            message: "Motion templates are not available yet in this version of SimpleEffect: \
                      texts, shapes and animation presets (list_animation_presets) are.",
        })
    }
}
