//! `get_transcript`: the words spoken in a range. Transcription arrives with prompt 21; until
//! then the tool says plainly that it is not available, so agents tell the user.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::ToolContext,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::RangeInput,
};

/// The tool.
pub struct GetTranscript;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// A range of the sequence; default: your task's range, else the whole sequence.
    pub range: Option<RangeInput>,
    /// Sequence id or exact name; default: your task's sequence.
    pub sequence: Option<String>,
}

#[derive(Debug, Serialize)]
struct Unavailable {
    available: bool,
    message: &'static str,
}

impl McpTool for GetTranscript {
    type Input = Input;

    const NAME: &'static str = "get_transcript";

    const DESCRIPTION: &'static str = "The words spoken in a range of the sequence, with their \
        times. Not available yet in this version of SimpleEffect: it answers available=false.";

    fn call(
        _context: ToolContext,
        _input: Input,
    ) -> impl Future<Output = Result<ToolOutput, ToolError>> + Send {
        std::future::ready(Self::answer())
    }
}

impl GetTranscript {
    fn answer() -> Result<ToolOutput, ToolError> {
        // TODO(prompt-21): read the transcripts of the clips in the range.
        ToolOutput::json(&Unavailable {
            available: false,
            message: "Transcription is not available yet in this version of SimpleEffect. Use \
                      get_audio_analysis for silences and loudness, and get_frames to see the \
                      sequence; tell the user that transcripts are not available.",
        })
    }
}
