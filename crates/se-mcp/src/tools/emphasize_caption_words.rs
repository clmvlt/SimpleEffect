//! `emphasize_caption_words`: puts words of a caption forward (the style's emphasis colour) or
//! back.

use schemars::JsonSchema;
use se_app::agent_edits::AgentEdit;
use serde::Deserialize;

use super::edit_caption_text::expect_caption;
use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    writes::{Planned, WriteScene, WriteTool, run},
};

/// The tool.
pub struct EmphasizeCaptionWords;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The caption (an `item_id`, or `$name` in `apply_batch`).
    pub item_id: String,
    /// Indices of its words, from 0 (`get_item` lists them in `properties.words`).
    pub words: Vec<u32>,
    /// `true` (default) puts them forward, `false` back to normal.
    pub emphasis: Option<bool>,
}

impl McpTool for EmphasizeCaptionWords {
    type Input = Input;

    const NAME: &'static str = "emphasize_caption_words";

    const DESCRIPTION: &'static str = "Puts words of a caption forward — drawn in the style's \
        emphasis colour, like keywords of social videos — or back to normal. Use it sparingly: \
        one or two key words per caption at most. Returns the caption as it is now.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for EmphasizeCaptionWords {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        expect_caption(scene, &input.item_id)?;
        if input.words.is_empty() {
            return Err(ToolError::invalid_arguments(
                "words is empty: give the indices of the words (from 0)",
            ));
        }
        Ok(Planned::one(AgentEdit::SetCaptionEmphasis {
            item: scene.item(&input.item_id)?,
            words: input.words,
            emphasis: input.emphasis.unwrap_or(true),
        }))
    }
}
