//! `edit_caption_text`: corrects or rewrites the text of one caption; unchanged words keep their
//! timing, so highlighting styles still follow the speech.

use schemars::JsonSchema;
use se_app::agent_edits::AgentEdit;
use serde::Deserialize;

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    writes::{Planned, WriteScene, WriteTool, run},
};

/// The tool.
pub struct EditCaptionText;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The caption (an `item_id`, or `$name` in `apply_batch`).
    pub item_id: String,
    /// Its new text (words separated by spaces, punctuation attached). Not empty: delete the
    /// caption with `delete_items` instead.
    pub text: String,
}

impl McpTool for EditCaptionText {
    type Input = Input;

    const NAME: &'static str = "edit_caption_text";

    const DESCRIPTION: &'static str = "Changes the text of one caption (fix a misheard word, \
        punctuation, a name). Words left unchanged keep their timing; a corrected word keeps the \
        timing of the word it replaces; added or removed words share the time of the words they \
        replace in proportion to their length. Returns the caption as it is now.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for EditCaptionText {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        expect_caption(scene, &input.item_id)?;
        if input.text.trim().is_empty() {
            return Err(ToolError::invalid_arguments(
                "text is empty: delete the caption with delete_items instead",
            ));
        }
        Ok(Planned::one(AgentEdit::EditCaptionText {
            item: scene.item(&input.item_id)?,
            text: input.text,
        }))
    }
}

/// Refuses an item that is not a caption.
///
/// # Errors
/// [`ToolError::invalid_arguments`] naming the item's kind.
pub fn expect_caption(scene: &WriteScene<'_>, reference: &str) -> Result<(), ToolError> {
    match scene.item_kind(reference)? {
        "caption" => Ok(()),
        kind => Err(ToolError::invalid_arguments(format!(
            "{reference} is a {kind}, not a caption; get_timeline lists the captions of the \
             caption track"
        ))),
    }
}
