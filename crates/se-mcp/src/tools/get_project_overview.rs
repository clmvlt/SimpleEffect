//! `get_project_overview`: the project at a glance — sequences, the task's tracks, media counts,
//! the task's scope. The first call of an agent that does not know the project.

use schemars::JsonSchema;
use se_domain::{AssetKind, Project, Sequence};
use serde::{Deserialize, Serialize};

use crate::{
    context::{ToolContext, find_sequence},
    describe::{TrackSummary, track_summary},
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::{RangeOutput, TimeOutput},
};

/// The tool.
pub struct GetProjectOverview;

/// No arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {}

#[derive(Debug, Serialize)]
struct Overview {
    project: ProjectHeader,
    scope: Scope,
    sequences: Vec<SequenceSummary>,
    /// Tracks of the task's sequence, bottom video track first.
    tracks: Vec<TrackSummary>,
    assets: AssetCounts,
    brand_kit: bool,
    templates: usize,
}

#[derive(Debug, Serialize)]
struct ProjectHeader {
    project_id: String,
    name: String,
    revision: u64,
}

#[derive(Debug, Serialize)]
struct Scope {
    task_id: String,
    sequence_id: String,
    sequence_name: String,
    /// `null`: the whole sequence.
    range: Option<RangeOutput>,
    /// `null`: every track.
    tracks: Option<Vec<ScopeTrack>>,
}

#[derive(Debug, Serialize)]
struct ScopeTrack {
    track_id: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct SequenceSummary {
    sequence_id: String,
    name: String,
    width: u32,
    height: u32,
    fps: f64,
    duration: TimeOutput,
    track_count: usize,
    item_count: usize,
    marker_count: usize,
    is_task_sequence: bool,
}

#[derive(Debug, Serialize)]
struct AssetCounts {
    total: usize,
    video: usize,
    audio: usize,
    image: usize,
    missing: usize,
}

impl McpTool for GetProjectOverview {
    type Input = Input;

    const NAME: &'static str = "get_project_overview";

    const DESCRIPTION: &'static str = "The project at a glance: its sequences (format, duration, \
        counts), the tracks of your task's sequence, media counts, and your task's scope \
        (sequence, time range, tracks). Call it first.";

    async fn call(context: ToolContext, _input: Input) -> Result<ToolOutput, ToolError> {
        let project = context.project().await?;
        ToolOutput::json(&overview(&project, &context)?)
    }
}

fn overview(project: &Project, context: &ToolContext) -> Result<Overview, ToolError> {
    let agent = &context.agent;
    let sequence = find_sequence(project, agent, None)?;
    let scope_tracks = agent.scope.tracks.as_ref().map(|ids| {
        ids.iter()
            .map(|id| ScopeTrack {
                track_id: id.to_string(),
                name: sequence
                    .tracks
                    .iter()
                    .find(|track| track.id == *id)
                    .map(|track| track.name.clone())
                    .unwrap_or_default(),
            })
            .collect()
    });
    let count = |kind: AssetKind| {
        project
            .assets
            .iter()
            .filter(|asset| asset.kind == kind)
            .count()
    };
    Ok(Overview {
        project: ProjectHeader {
            project_id: project.id.to_string(),
            name: project.name.clone(),
            revision: project.revision,
        },
        scope: Scope {
            task_id: agent.task_id.to_string(),
            sequence_id: sequence.id.to_string(),
            sequence_name: sequence.name.clone(),
            range: agent.scope.range.map(Into::into),
            tracks: scope_tracks,
        },
        sequences: project
            .sequences
            .iter()
            .map(|candidate| sequence_summary(candidate, candidate.id == sequence.id))
            .collect(),
        tracks: sequence.tracks.iter().map(track_summary).collect(),
        assets: AssetCounts {
            total: project.assets.len(),
            video: count(AssetKind::Video),
            audio: count(AssetKind::Audio),
            image: count(AssetKind::Image),
            missing: project.assets.iter().filter(|asset| asset.missing).count(),
        },
        brand_kit: project.brand_kit.is_some(),
        templates: project.templates.len(),
    })
}

fn sequence_summary(sequence: &Sequence, is_task_sequence: bool) -> SequenceSummary {
    SequenceSummary {
        sequence_id: sequence.id.to_string(),
        name: sequence.name.clone(),
        width: sequence.format.width,
        height: sequence.format.height,
        fps: (sequence.format.frame_rate.as_f64() * 1000.0).round() / 1000.0,
        duration: sequence.content_end().into(),
        track_count: sequence.tracks.len(),
        item_count: sequence.tracks.iter().map(|track| track.items.len()).sum(),
        marker_count: sequence.markers.len(),
        is_task_sequence,
    }
}
