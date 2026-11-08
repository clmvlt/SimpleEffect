//! `list_assets`: the media of the project (kind, length, size, analysis summaries, use count),
//! filtered and paginated.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::ToolContext,
    describe::{AssetSummary, asset_kind_name, asset_name, asset_summary},
    output::{ToolError, ToolOutput},
    registry::McpTool,
};

/// Assets per page by default.
const DEFAULT_LIMIT: usize = 50;
/// Most assets per page.
const MAX_LIMIT: usize = 200;

/// The tool.
pub struct ListAssets;

/// Kinds of media.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssetKindFilter {
    /// Video files.
    Video,
    /// Sound files.
    Audio,
    /// Still images.
    Image,
}

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Only this kind of media.
    pub kind: Option<AssetKindFilter>,
    /// Only media whose file name contains this text (case ignored).
    pub query: Option<String>,
    /// Assets to skip (from a previous page's `next_offset`); default 0.
    pub offset: Option<usize>,
    /// Assets per page; default 50, at most 200.
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
struct Listing {
    assets: Vec<AssetSummary>,
    total: usize,
    /// Pass it as `offset` for the next page; `null` when this is the last one.
    next_offset: Option<usize>,
}

impl McpTool for ListAssets {
    type Input = Input;

    const NAME: &'static str = "list_assets";

    const DESCRIPTION: &'static str = "The media files of the project: id, file name, kind, \
        duration, size, frame rate, sound channels, whether the file is missing, how many items \
        use it, and analysis summaries (loudness, silences, scene changes). Filter by kind or \
        name; paginated.";

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        let project = context.project().await?;
        let query = input.query.as_deref().map(str::to_lowercase);
        let matching: Vec<AssetSummary> = project
            .assets
            .iter()
            .filter(|asset| {
                input.kind.is_none_or(|kind| {
                    asset_kind_name(asset.kind)
                        == match kind {
                            AssetKindFilter::Video => "video",
                            AssetKindFilter::Audio => "audio",
                            AssetKindFilter::Image => "image",
                        }
                })
            })
            .filter(|asset| {
                query
                    .as_deref()
                    .is_none_or(|query| asset_name(asset).to_lowercase().contains(query))
            })
            .map(|asset| asset_summary(&project, asset))
            .collect();
        let total = matching.len();
        let offset = input.offset.unwrap_or(0).min(total);
        let limit = input.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
        let assets: Vec<AssetSummary> = matching.into_iter().skip(offset).take(limit).collect();
        let next = offset + assets.len();
        ToolOutput::json(&Listing {
            assets,
            total,
            next_offset: (next < total).then_some(next),
        })
    }
}
