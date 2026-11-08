//! `get_source_frames`: pictures of a media file itself at source times (extracted by ffmpeg
//! and cached, `MediaService::frame_jpeg`), returned as MCP images.

use schemars::JsonSchema;
use se_domain::{AssetKind, Flicks, FrameRate, format_clock};
use serde::{Deserialize, Serialize};

use crate::{
    context::{ToolContext, find_asset, find_sequence},
    describe::asset_name,
    output::{ToolError, ToolOutput, codes},
    registry::McpTool,
    time_io::{TimeInput, TimeOutput},
    tools::get_frames::DEFAULT_MAX_WIDTH,
};

/// Most pictures one call returns.
const MAX_SOURCE_FRAMES: usize = 8;

/// The tool.
pub struct GetSourceFrames;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The asset's id (from `list_assets` or `get_timeline`).
    pub asset: String,
    /// Source times (from the media file's first frame), at most 8.
    pub times: Vec<TimeInput>,
    /// Largest picture width in pixels (16–3840, never enlarged); default 768.
    pub max_width: Option<u32>,
}

#[derive(Debug, Serialize)]
struct SourceFramesHeader {
    asset_id: String,
    name: String,
    /// Times are source times (in the media file).
    time_base: &'static str,
    frames: Vec<TimeOutput>,
}

impl McpTool for GetSourceFrames {
    type Input = Input;

    const NAME: &'static str = "get_source_frames";

    const DESCRIPTION: &'static str = "Pictures of a media file itself (not the edit) at source \
        times, e.g. to see what a clip could show beyond its edges or to pick a moment before \
        placing it. Returns a JSON header then one image per time (images: one picture).";

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        let project = context.project().await?;
        let asset = find_asset(&project, &input.asset)?;
        if asset.kind == AssetKind::Audio {
            return Err(ToolError::new(
                codes::INVALID_ARGUMENTS,
                format!("{} is a sound file: it has no picture", asset_name(asset)),
            ));
        }
        if input.times.is_empty() || input.times.len() > MAX_SOURCE_FRAMES {
            return Err(ToolError::new(
                codes::TOO_MANY_FRAMES,
                format!("ask for 1 to {MAX_SOURCE_FRAMES} times per call"),
            ));
        }
        let rate = asset
            .media_info
            .as_ref()
            .and_then(|info| info.video_stream.as_ref())
            .and_then(|video| video.frame_rate)
            .or_else(|| {
                find_sequence(&project, &context.agent, None)
                    .ok()
                    .map(|sequence| sequence.format.frame_rate)
            })
            .unwrap_or(FrameRate::FPS_25);
        let times = input
            .times
            .iter()
            .map(|time| time.to_flicks(rate))
            .collect::<Result<Vec<Flicks>, _>>()?;
        let max_width = input.max_width.unwrap_or(DEFAULT_MAX_WIDTH);
        let mut pictures = Vec::with_capacity(times.len());
        for time in &times {
            let jpeg = context
                .deps
                .media
                .frame_jpeg(context.agent.project_id, asset.id, *time, max_width)
                .await?;
            pictures.push(jpeg);
        }
        let mut output = ToolOutput::json(&SourceFramesHeader {
            asset_id: asset.id.to_string(),
            name: asset_name(asset),
            time_base: "source",
            frames: times.iter().copied().map(Into::into).collect(),
        })?;
        for (index, (time, jpeg)) in times.iter().zip(&pictures).enumerate() {
            output.push_text(format!(
                "Source frame {} at {}",
                index + 1,
                format_clock(*time)
            ));
            output.push_jpeg(jpeg);
        }
        Ok(output)
    }
}
