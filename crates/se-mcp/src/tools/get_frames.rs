//! `get_frames`: pictures of the sequence as the viewer shows them (texts, transformations,
//! animations, transitions composited), drawn by the editor window with `@se/engine`
//! (`FrameRenderService`, ADR-0028) and returned as MCP images.

use schemars::JsonSchema;
use se_app::{
    FrameRenderSpec,
    frame_renders::{MAX_RENDER_FRAMES, frame_size},
};
use se_domain::{Flicks, Sequence, format_clock};
use serde::{Deserialize, Serialize};

use crate::{
    context::{ToolContext, default_range, find_sequence},
    output::{ToolError, ToolOutput, codes},
    registry::McpTool,
    time_io::{RangeInput, TimeInput, TimeOutput},
    timing::spread,
};

/// Frames returned by default when spreading over a range.
const DEFAULT_COUNT: usize = 6;
/// Most frames one call returns (each costs the agent hundreds of tokens).
pub const MAX_FRAMES: usize = 12;
/// Picture width by default, in pixels.
pub const DEFAULT_MAX_WIDTH: u32 = 768;

/// The tool.
pub struct GetFrames;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Sequence times to show (at most 12). Give either `times` or `range`.
    pub times: Option<Vec<TimeInput>>,
    /// A range to spread frames over (the middle of `count` equal slices); default: your task's
    /// range, else the whole sequence.
    pub range: Option<RangeInput>,
    /// Frames to spread over `range`; default 6, at most 12.
    pub count: Option<usize>,
    /// Largest picture width in pixels (64–1920); default 768.
    pub max_width: Option<u32>,
    /// Sequence id or exact name; default: your task's sequence.
    pub sequence: Option<String>,
}

#[derive(Debug, Serialize)]
struct FramesHeader {
    sequence_id: String,
    width: u32,
    height: u32,
    /// The instants shown, in the order of the pictures that follow.
    frames: Vec<TimeOutput>,
}

impl McpTool for GetFrames {
    type Input = Input;

    const NAME: &'static str = "get_frames";

    const DESCRIPTION: &'static str = "Look at the sequence: pictures exactly as the viewer shows \
        them (texts, shapes, transformations, animations and transitions included), at given \
        sequence times or spread over a range (default: 6 frames over your task's range, else \
        the whole sequence). Returns a JSON header then one image per time. Needs the project \
        open in a SimpleEffect window.";

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        let project = context.project().await?;
        let sequence = find_sequence(&project, &context.agent, input.sequence.as_deref())?;
        let times = requested_times(&input, &context, sequence)?;
        let (width, height) = frame_size(
            &sequence.format,
            input.max_width.unwrap_or(DEFAULT_MAX_WIDTH),
        );
        let frames = context
            .deps
            .renders
            .render(FrameRenderSpec {
                project_id: context.agent.project_id,
                sequence_id: sequence.id,
                times: times.clone(),
                width,
                height,
            })
            .await?;
        let mut output = ToolOutput::json(&FramesHeader {
            sequence_id: sequence.id.to_string(),
            width,
            height,
            frames: times.iter().copied().map(Into::into).collect(),
        })?;
        for (index, (time, jpeg)) in times.iter().zip(&frames).enumerate() {
            output.push_text(format!("Frame {} at {}", index + 1, format_clock(*time)));
            output.push_jpeg(jpeg);
        }
        Ok(output)
    }
}

/// The sequence times to draw, checked against the sequence's length.
fn requested_times(
    input: &Input,
    context: &ToolContext,
    sequence: &Sequence,
) -> Result<Vec<Flicks>, ToolError> {
    let rate = sequence.format.frame_rate;
    let times = match (&input.times, &input.range) {
        (Some(_), Some(_)) => {
            return Err(ToolError::invalid_arguments(
                "give either times or range (with count), not both",
            ));
        }
        (Some(times), None) => times
            .iter()
            .map(|time| time.to_flicks(rate))
            .collect::<Result<Vec<_>, _>>()?,
        (None, range) => {
            let range = match range {
                Some(range) => range.to_range(rate)?,
                None => default_range(&context.agent, sequence)?,
            };
            spread(range, input.count.unwrap_or(DEFAULT_COUNT).max(1))
        }
    };
    check_times(&times, sequence)?;
    Ok(times)
}

fn check_times(times: &[Flicks], sequence: &Sequence) -> Result<(), ToolError> {
    let limit = MAX_FRAMES.min(MAX_RENDER_FRAMES);
    if times.is_empty() {
        return Err(ToolError::invalid_arguments("ask for at least one time"));
    }
    if times.len() > limit {
        return Err(ToolError::new(
            codes::TOO_MANY_FRAMES,
            format!(
                "{} frames asked; ask for at most {limit} per call (spread fewer over the range, \
                 or call again for another part)",
                times.len()
            ),
        ));
    }
    let end = sequence.content_end();
    if let Some(time) = times
        .iter()
        .find(|time| time.is_negative() || (*time >= &end && end > Flicks::ZERO))
    {
        return Err(ToolError::new(
            codes::INVALID_TIME,
            format!(
                "{} is outside « {} », which lasts {}",
                format_clock(*time),
                sequence.name,
                format_clock(end)
            ),
        ));
    }
    Ok(())
}
