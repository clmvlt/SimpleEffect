//! Shared plumbing of the smart-cut tools (prompt 22): running a cut through
//! `AgentEditService::apply_cuts` and the summary agents get back (cuts, time gained, tracks,
//! media that could not be read).

use std::collections::BTreeSet;

use se_app::smart_cuts::{CutRequest, CutTarget};
use se_domain::{AssetId, Flicks, Project, SequenceId, TimeRange};
use serde::Serialize;

use crate::{
    context::ToolContext,
    describe::asset_name,
    output::{ToolError, ToolOutput},
    time_io::{RangeOutput, TimeOutput},
    writes::app_error,
};

/// Most ranges listed in an answer (the count and the total are always complete).
const MAX_LISTED: usize = 50;

/// What a smart-cut tool answers.
#[derive(Debug, Serialize)]
struct CutAnswer {
    /// Number of cuts made.
    cuts: usize,
    /// Total time removed.
    removed: TimeOutput,
    /// Words removed (words and fillers).
    words_removed: u32,
    /// The ranges removed, in sequence time **before** the cut, in order (first 50).
    ranges: Vec<RangeOutput>,
    /// Tracks edited.
    tracks: Vec<String>,
    /// The project revision after the cut; absent when nothing was cut.
    #[serde(skip_serializing_if = "Option::is_none")]
    revision: Option<u64>,
    /// Why nothing was cut, or what could not be read.
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
    /// Media of the range not transcribed: their words could not be cut.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    untranscribed: Vec<AssetRef>,
    /// Media of the range whose sound is not analysed yet: no silence was found there.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    unanalysed: Vec<AssetRef>,
}

#[derive(Debug, Serialize)]
struct AssetRef {
    asset_id: String,
    name: String,
}

/// Cuts `target` on the task's sequence (one AI transaction labelled `label`) and summarises it.
///
/// # Errors
/// The refusal (`OUT_OF_SCOPE`, `LOCKED_TRACK`, `VALIDATION_ERROR`…) as a [`ToolError`].
pub async fn run_cut(
    context: &ToolContext,
    label: &str,
    sequence_id: SequenceId,
    target: CutTarget,
    all_tracks: bool,
) -> Result<ToolOutput, ToolError> {
    let request = CutRequest {
        sequence_id,
        target,
        all_tracks,
        limit: None,
    };
    let media = context
        .deps
        .smart_cuts
        .media(context.agent.project_id, sequence_id)
        .await?;
    let outcome = context
        .deps
        .edits
        .apply_cuts(&context.agent, label, &request, &media)
        .await
        .map_err(app_error)?;
    let summary = &outcome.summary;
    let project = &outcome.project;
    let track_names = project
        .sequences
        .iter()
        .find(|sequence| sequence.id == sequence_id)
        .map(|sequence| {
            sequence
                .tracks
                .iter()
                .filter(|track| summary.track_ids.contains(&track.id))
                .map(|track| track.name.clone())
                .collect()
        })
        .unwrap_or_default();
    let untranscribed = asset_refs(project, &summary.untranscribed_assets);
    let unanalysed = asset_refs(project, &summary.unanalysed_assets);
    ToolOutput::json(&CutAnswer {
        cuts: summary.cuts.len(),
        removed: summary.removed_duration.into(),
        words_removed: summary.word_count,
        ranges: listed(&summary.cuts),
        tracks: track_names,
        revision: outcome.revision,
        note: note(summary.cuts.is_empty(), &untranscribed, &unanalysed),
        untranscribed,
        unanalysed,
    })
}

/// The first ranges, as agents read them.
fn listed(cuts: &[TimeRange]) -> Vec<RangeOutput> {
    cuts.iter()
        .take(MAX_LISTED)
        .map(|cut| (*cut).into())
        .collect()
}

fn asset_refs(project: &Project, assets: &[AssetId]) -> Vec<AssetRef> {
    let wanted: BTreeSet<&AssetId> = assets.iter().collect();
    project
        .assets
        .iter()
        .filter(|asset| wanted.contains(&asset.id))
        .map(|asset| AssetRef {
            asset_id: asset.id.to_string(),
            name: asset_name(asset),
        })
        .collect()
}

fn note(nothing: bool, untranscribed: &[AssetRef], unanalysed: &[AssetRef]) -> Option<String> {
    let mut parts = Vec::new();
    if nothing {
        parts.push("nothing matched, so nothing was cut".to_owned());
    }
    if !untranscribed.is_empty() {
        parts.push(
            "some media are not transcribed: ask the user to transcribe them (« Transcrire » in \
             the media bin)"
                .to_owned(),
        );
    }
    if !unanalysed.is_empty() {
        parts.push(
            "some media have no sound analysis yet (it runs in the background): try again in a \
             moment"
                .to_owned(),
        );
    }
    (!parts.is_empty()).then(|| parts.join("; "))
}

/// A duration given in seconds by an agent, checked against `[min, max]` seconds.
///
/// # Errors
/// `INVALID_ARGUMENTS` naming `name` and the accepted range.
pub fn seconds_in(name: &str, value: f64, min: f64, max: f64) -> Result<Flicks, ToolError> {
    if !(min..=max).contains(&value) {
        return Err(ToolError::new(
            crate::output::codes::INVALID_ARGUMENTS,
            format!("{name} must be from {min} to {max} seconds (got {value})"),
        ));
    }
    Flicks::from_seconds_f64(value).map_err(|error| {
        ToolError::new(
            crate::output::codes::INVALID_ARGUMENTS,
            format!("{name}: {error}"),
        )
    })
}
