//! `get_brand_kit`: the project's colours, fonts and logos. Brand kits arrive with prompt 25;
//! until then the tool says plainly that they are not available.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
};

/// The tool.
pub struct GetBrandKit;

/// No arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {}

#[derive(Debug, Serialize)]
struct Unavailable {
    available: bool,
    message: &'static str,
}

impl McpTool for GetBrandKit {
    type Input = Input;

    const NAME: &'static str = "get_brand_kit";

    const DESCRIPTION: &'static str = "The project's brand kit (colours, fonts, logos). Not \
        available yet in this version of SimpleEffect: it answers available=false.";

    fn call(
        _context: ToolContext,
        _input: Input,
    ) -> impl Future<Output = Result<ToolOutput, ToolError>> + Send {
        std::future::ready(Self::answer())
    }
}

impl GetBrandKit {
    fn answer() -> Result<ToolOutput, ToolError> {
        // TODO(prompt-25): return the project's brand kit.
        ToolOutput::json(&Unavailable {
            available: false,
            message: "Brand kits are not available yet in this version of SimpleEffect: use the \
                      styles already in the timeline (get_item) as the reference.",
        })
    }
}
