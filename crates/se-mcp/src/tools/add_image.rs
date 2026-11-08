//! `add_image`: an image asset of the project (logo, sticker, picture) over a time range of the
//! task's sequence, with optional animations.

use schemars::JsonSchema;
use se_app::agent_edits::{AgentEdit, NewAgentItem};
use se_domain::{Animatable, AssetKind, ImageFit, ImageItem, ItemKind, Vec2};
use serde::Deserialize;

use crate::{
    context::{ToolContext, find_asset},
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::TimeInput,
    write_inputs::{AnimationInput, PositionInput, presets, span},
    writes::{Planned, WriteScene, WriteTool, run},
};

/// The tool.
pub struct AddImage;

/// How the image is sized in the frame.
#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FitInput {
    /// Whole image inside the frame (default).
    Fit,
    /// Covers the frame, cropping what overflows.
    Fill,
    /// Its own size (one image pixel per design pixel).
    Natural,
}

impl FitInput {
    /// The document's fit.
    #[must_use]
    pub const fn to_fit(self) -> ImageFit {
        match self {
            Self::Fit => ImageFit::Fit,
            Self::Fill => ImageFit::Fill,
            Self::Natural => ImageFit::Natural,
        }
    }
}

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// The image asset (an `asset_id` of kind `image` from `list_assets`).
    pub asset: String,
    /// When it appears, in sequence time.
    pub start: TimeInput,
    /// When it disappears (excluded). Give `end` or `duration`.
    pub end: Option<TimeInput>,
    /// How long it stays. Give `end` or `duration`.
    pub duration: Option<TimeInput>,
    /// Where its centre sits; default "center".
    pub position: Option<PositionInput>,
    /// How it is sized; default fit.
    pub fit: Option<FitInput>,
    /// Scale factor (1 = as fitted); e.g. 0.3 for a corner logo.
    pub scale: Option<f64>,
    /// Opacity from 0 to 1; default 1.
    pub opacity: Option<f64>,
    /// A video track (id or name); default: the highest free track of the task.
    pub track: Option<String>,
    /// Entrance animation (preset `in` phase).
    pub animation_in: Option<AnimationInput>,
    /// Exit animation (preset `out` phase).
    pub animation_out: Option<AnimationInput>,
}

impl McpTool for AddImage {
    type Input = Input;

    const NAME: &'static str = "add_image";

    const DESCRIPTION: &'static str = "Places an image asset of the project (logo, sticker, \
        picture) over a time range of the task's sequence, with its place, fit, scale, opacity \
        and optional entrance/exit animations. Answers the new item_id.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        run::<Self>(context, input).await
    }
}

impl WriteTool for AddImage {
    fn plan(scene: &WriteScene<'_>, input: Input) -> Result<Planned, ToolError> {
        let found = find_asset(scene.project, &input.asset)?;
        if found.kind != AssetKind::Image {
            return Err(ToolError::invalid_arguments(format!(
                "asset {} is not an image: add_image places pictures (logos, stickers); \
                 list images with list_assets {{\"kind\": \"image\"}}",
                input.asset
            )));
        }
        let asset = found.id;
        let range = span(
            scene,
            &input.start,
            input.end.as_ref(),
            input.duration.as_ref(),
        )?;
        let mut image = ImageItem::new(asset);
        if let Some(fit) = input.fit {
            image.fit = fit.to_fit();
        }
        if let Some(position) = input.position {
            image.transform.position = Animatable::constant(position.to_vec2());
        }
        if let Some(scale) = input.scale {
            image.transform.scale = Animatable::constant(Vec2::splat(scale));
        }
        if let Some(opacity) = input.opacity {
            image.transform.opacity = Animatable::constant(opacity);
        }
        let new = NewAgentItem {
            kind: ItemKind::Image(image),
            range,
            track: input
                .track
                .as_deref()
                .map(|track| scene.video_track(track))
                .transpose()?,
            presets: presets(
                scene,
                input.animation_in.as_ref(),
                input.animation_out.as_ref(),
            )?,
        };
        Ok(Planned {
            edits: vec![AgentEdit::AddItem(Box::new(new))],
            creates: Some(0),
        })
    }
}
