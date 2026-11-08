//! `remove_fillers`: cuts the filler words (« euh », « um »…) of a range, as one ripple cut.

use schemars::JsonSchema;
use se_app::smart_cuts::CutTarget;
use se_protocol::FillerList;
use serde::Deserialize;

use crate::{
    context::{ToolContext, default_range, find_sequence},
    cuts::run_cut,
    output::{ToolError, ToolOutput, codes},
    registry::McpTool,
    time_io::RangeInput,
};

/// The tool.
pub struct RemoveFillers;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// A range of the sequence; default: your task's range, else the whole sequence.
    pub range: Option<RangeInput>,
    /// Fillers to remove instead of the user's lists (words or short phrases such as « you
    /// know »; case, accents and punctuation ignored, « euuuh » matches « euh »). Default: the
    /// user's lists for each transcript's language.
    pub words: Option<Vec<String>>,
    /// Cut every unlocked track instead of the speech's tracks and their linked picture;
    /// default false.
    pub all_tracks: Option<bool>,
}

impl McpTool for RemoveFillers {
    type Input = Input;

    const NAME: &'static str = "remove_fillers";

    const DESCRIPTION: &'static str = "Cuts the filler words of a range (default: your task's \
        range): « euh », « hum », « um », « uh »… from the user's lists for each transcript's \
        language, or your own words. Same margins and sync as remove_words (one undoable \
        transaction). Only transcribed fillers can be found: if none come back, the transcript \
        may not contain them. Returns the cuts made and the time gained.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        let project = context.project().await?;
        let sequence = find_sequence(&project, &context.agent, None)?;
        let range = match &input.range {
            Some(range) => range.to_range(sequence.format.frame_rate)?,
            None => default_range(&context.agent, sequence)?,
        };
        let settings = context.deps.smart_cuts.settings().await?;
        let lists = match input.words {
            Some(words) if words.iter().all(|word| word.trim().is_empty()) => {
                return Err(ToolError::new(
                    codes::INVALID_ARGUMENTS,
                    "words is empty: give fillers to remove, or leave it out to use the user's \
                     lists",
                ));
            }
            Some(words) => vec![FillerList {
                language: "any".to_owned(),
                words,
            }],
            None => settings.fillers,
        };
        let target = CutTarget::Fillers {
            range: Some(range),
            lists,
            max_duration: settings.filler_max_duration,
        };
        let all_tracks = input.all_tracks.unwrap_or(false);
        run_cut(&context, Self::NAME, sequence.id, target, all_tracks).await
    }
}
