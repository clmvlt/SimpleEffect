//! `get_scene_changes`: shot changes — of an asset (source time) or of a range of the sequence
//! (each picture clip's changes brought back to sequence time, plus the edit's own cuts).

use std::collections::BTreeSet;

use schemars::JsonSchema;
use se_domain::{Flicks, TimeRange, TrackKind};
use serde::{Deserialize, Serialize};

use crate::{
    analyses::{AnalysisCache, clips_in},
    context::{ToolContext, default_range, find_asset, find_sequence},
    describe::asset_name,
    output::{ToolError, ToolOutput, codes},
    registry::McpTool,
    time_io::{RangeInput, RangeOutput, TimeOutput},
};

/// Most changes listed in one answer.
const MAX_LISTED: usize = 200;

/// The tool.
pub struct GetSceneChanges;

/// Arguments: an `asset`, or a `range` of the sequence (the default: your task's range, else the
/// whole sequence).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// An asset's id: its shot changes, in source time.
    pub asset: Option<String>,
    /// A range of the sequence: shot changes inside its picture clips and the edit's cuts, in
    /// sequence time.
    pub range: Option<RangeInput>,
    /// Sequence id or exact name (with `range`); default: your task's sequence.
    pub sequence: Option<String>,
}

#[derive(Debug, Serialize)]
struct AssetScenes {
    asset_id: String,
    name: String,
    time_base: &'static str,
    /// Detection sensitivity (0–1, lower finds more changes).
    threshold: f64,
    changes: Vec<TimeOutput>,
    total: usize,
}

#[derive(Debug, Serialize)]
struct SequenceScenes {
    sequence_id: String,
    time_base: &'static str,
    range: RangeOutput,
    /// Shot changes inside the media of visible picture clips.
    changes: Vec<SceneChange>,
    total: usize,
    /// Where the edit itself cuts: a picture clip starts or ends on a visible video track.
    cuts: Vec<TimeOutput>,
    /// Clips whose pictures are not analysed yet.
    unanalysed: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SceneChange {
    time: TimeOutput,
    item_id: String,
    asset_id: String,
}

impl McpTool for GetSceneChanges {
    type Input = Input;

    const NAME: &'static str = "get_scene_changes";

    const DESCRIPTION: &'static str = "Shot changes found by analysis. With `asset`: the file's \
        changes in source time. Otherwise for a range of the sequence (default: your task's \
        range, else the whole sequence), in sequence time: changes inside the visible picture \
        clips, plus the edit's own cuts. NOT_ANALYSED until the background analysis is done.";

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        let project = context.project().await?;
        let mut cache = AnalysisCache::new(&context);
        if let Some(asset_id) = &input.asset {
            if input.range.is_some() {
                return Err(ToolError::invalid_arguments(
                    "give either asset or range, not both",
                ));
            }
            let asset = find_asset(&project, asset_id)?;
            let detection = cache.scenes(asset.id).await?.ok_or_else(|| {
                ToolError::new(
                    codes::NOT_ANALYSED,
                    format!(
                        "the pictures of {} are not analysed (yet, or it has none): analyses run \
                         in the background after import; try again in a moment",
                        asset_name(asset)
                    ),
                )
            })?;
            return ToolOutput::json(&AssetScenes {
                asset_id: asset.id.to_string(),
                name: asset_name(asset),
                time_base: "source",
                threshold: detection.threshold,
                total: detection.changes.len(),
                changes: detection
                    .changes
                    .iter()
                    .take(MAX_LISTED)
                    .copied()
                    .map(Into::into)
                    .collect(),
            });
        }
        let sequence = find_sequence(&project, &context.agent, input.sequence.as_deref())?;
        let range = match &input.range {
            Some(range) => range.to_range(sequence.format.frame_rate)?,
            None => default_range(&context.agent, sequence)?,
        };
        let clips: Vec<_> = clips_in(sequence, TrackKind::Video, range)
            .into_iter()
            .filter(|clip| !clip.track.hidden)
            .collect();
        let mut changes = Vec::new();
        let mut unanalysed = Vec::new();
        let mut cuts = BTreeSet::new();
        for clip in &clips {
            for edge in [
                clip.item.timeline_range.start(),
                clip.item.timeline_range.end(),
            ] {
                if edge > range.start() && edge < range.end() {
                    cuts.insert(edge);
                }
            }
            let Some(detection) = cache.scenes(clip.asset_id).await? else {
                unanalysed.push(clip.item.id.to_string());
                continue;
            };
            for change in &detection.changes {
                if let Some(time) = played_at(&clip.window, clip.visible, *change) {
                    changes.push((time, clip.item.id.to_string(), clip.asset_id.to_string()));
                }
            }
        }
        changes.sort_by_key(|(time, _, _)| *time);
        ToolOutput::json(&SequenceScenes {
            sequence_id: sequence.id.to_string(),
            time_base: "sequence",
            range: range.into(),
            total: changes.len(),
            changes: changes
                .into_iter()
                .take(MAX_LISTED)
                .map(|(time, item_id, asset_id)| SceneChange {
                    time: time.into(),
                    item_id,
                    asset_id,
                })
                .collect(),
            cuts: cuts.into_iter().map(Into::into).collect(),
            unanalysed,
        })
    }
}

/// When source time `change` shows in the sequence, if inside `visible`.
fn played_at(
    window: &crate::timing::ClipWindow,
    visible: TimeRange,
    change: Flicks,
) -> Option<Flicks> {
    if !window.source.contains(change) {
        return None;
    }
    window
        .to_timeline(change)
        .filter(|time| visible.contains(*time))
}
