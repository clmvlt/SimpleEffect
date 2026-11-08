//! `remove_words`: cuts words of the transcript (text-based editing) with natural margins, as one
//! ripple cut keeping linked picture and sound in sync.

use schemars::JsonSchema;
use se_app::smart_cuts::CutTarget;
use serde::Deserialize;

use crate::{
    context::{ToolContext, find_sequence},
    cuts::run_cut,
    output::{ToolError, ToolOutput, codes},
    registry::McpTool,
    time_io::RangeInput,
};

/// Most ranges per call.
const MAX_RANGES: usize = 500;

/// The tool.
pub struct RemoveWords;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Ranges of the words to remove, in sequence time, as `get_transcript` gives them: from the
    /// first word's start to the last word's end. A word is removed when its middle lies in a
    /// range (at most 500 ranges).
    pub ranges: Vec<RangeInput>,
    /// Cut every unlocked track (graphics and music included) instead of the speech's tracks
    /// and their linked picture; default false.
    pub all_tracks: Option<bool>,
}

impl McpTool for RemoveWords {
    type Input = Input;

    const NAME: &'static str = "remove_words";

    const DESCRIPTION: &'static str = "Cuts spoken words out of the sequence (text-based \
        editing): give the ranges of the words from get_transcript. Each cut keeps the longer \
        pause around the words and a short breath before the next word, never cuts inside a kept \
        word, and closes the gap on the speech's tracks and their linked picture (one undoable \
        transaction; later items move left). Cuts must lie in your task's range; items after it \
        may move. Returns the cuts made and the time gained.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        if input.ranges.is_empty() || input.ranges.len() > MAX_RANGES {
            return Err(ToolError::new(
                codes::INVALID_ARGUMENTS,
                "give from 1 to 500 ranges of words (from get_transcript)",
            ));
        }
        let project = context.project().await?;
        let sequence = find_sequence(&project, &context.agent, None)?;
        let rate = sequence.format.frame_rate;
        let ranges = input
            .ranges
            .iter()
            .map(|range| range.to_range(rate))
            .collect::<Result<Vec<_>, _>>()?;
        let target = CutTarget::Words { ranges };
        let all_tracks = input.all_tracks.unwrap_or(false);
        run_cut(&context, Self::NAME, sequence.id, target, all_tracks).await
    }
}
