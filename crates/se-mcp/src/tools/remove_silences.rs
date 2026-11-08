//! `remove_silences`: cuts the pauses of a range, read from the sound's level, as one ripple cut.

use schemars::JsonSchema;
use se_app::smart_cuts::CutTarget;
use se_protocol::SilenceCutParams;
use serde::Deserialize;

use crate::{
    context::{ToolContext, default_range, find_sequence, find_tracks},
    cuts::{run_cut, seconds_in},
    output::{ToolError, ToolOutput, codes},
    registry::McpTool,
    time_io::RangeInput,
};

/// The tool.
pub struct RemoveSilences;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// A range of the sequence; default: your task's range, else the whole sequence.
    pub range: Option<RangeInput>,
    /// Level under which sound is silence, in dBFS (-80 to -10); default: the user's setting
    /// (-40 unless changed). Raise it (-35) for noisy rooms, lower it (-50) for quiet ones.
    pub threshold_db: Option<f64>,
    /// Shortest pause cut, in seconds (0.1 to 10); default: the user's setting (0.5).
    pub min_silence: Option<f64>,
    /// Silence kept before and after each cut, in seconds (0 to 1); default: the user's setting
    /// (0.12). Keep at least 0.08 so speech breathes.
    pub padding: Option<f64>,
    /// Sound tracks whose silences count (ids or names; all of them must be quiet); default:
    /// every unmuted sound track holding media (your task's sound tracks when it has tracks).
    /// Leave music tracks out: music is never silent.
    pub tracks: Option<Vec<String>>,
    /// Cut every unlocked track instead of the listened tracks and their linked picture;
    /// default false.
    pub all_tracks: Option<bool>,
}

impl McpTool for RemoveSilences {
    type Input = Input;

    const NAME: &'static str = "remove_silences";

    const DESCRIPTION: &'static str = "Cuts the pauses of a range (default: your task's range) \
        where every listened sound track is quieter than threshold_db for at least min_silence, \
        keeping padding of silence on each side, and closes the gaps on those tracks and their \
        linked picture (one undoable transaction; later items move left). Pauses come from the \
        audio analysis (see get_audio_analysis). Leave music tracks out with tracks. Returns the \
        cuts made and the time gained.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        let project = context.project().await?;
        let sequence = find_sequence(&project, &context.agent, None)?;
        let range = match &input.range {
            Some(range) => range.to_range(sequence.format.frame_rate)?,
            None => default_range(&context.agent, sequence)?,
        };
        let defaults = context.deps.smart_cuts.settings().await?.silence;
        let params = SilenceCutParams {
            threshold_dbfs: threshold(input.threshold_db, defaults.threshold_dbfs)?,
            min_silence: match input.min_silence {
                Some(value) => seconds_in("min_silence", value, 0.1, 10.0)?,
                None => defaults.min_silence,
            },
            padding: match input.padding {
                Some(value) => seconds_in("padding", value, 0.0, 1.0)?,
                None => defaults.padding,
            },
        };
        let tracks = match &input.tracks {
            Some(names) => Some(
                find_tracks(sequence, Some(names))?
                    .into_iter()
                    .map(|track| track.id)
                    .collect(),
            ),
            None => None,
        };
        let target = CutTarget::Silences {
            range: Some(range),
            params,
            tracks,
        };
        let all_tracks = input.all_tracks.unwrap_or(false);
        run_cut(&context, Self::NAME, sequence.id, target, all_tracks).await
    }
}

/// The threshold, checked.
fn threshold(value: Option<f64>, default: f64) -> Result<f64, ToolError> {
    let value = value.unwrap_or(default);
    if (-80.0..=-10.0).contains(&value) {
        Ok(value)
    } else {
        Err(ToolError::new(
            codes::INVALID_ARGUMENTS,
            format!("threshold_db must be from -80 to -10 dBFS (got {value})"),
        ))
    }
}
