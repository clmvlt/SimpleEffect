//! `transcribe_media`: transcribes media that have no transcript yet, locally with the installed
//! whisper.cpp model, then waits a while for the result (ADR-0034). Without arguments it takes the
//! media played by the sound clips of the task's range (else the whole sequence). It changes no
//! document item: transcripts are stored next to the project, like analyses.

use std::time::Duration;

use schemars::JsonSchema;
use se_app::transcription::{
    AssetTranscription, TranscriptionRound, TranscriptionState, UnavailableReason,
};
use se_domain::{AssetId, Project, TrackKind};
use serde::{Deserialize, Serialize};

use crate::{
    analyses::clips_in,
    context::{ToolContext, default_range, find_asset, find_sequence},
    describe::asset_name,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::RangeInput,
};

/// Seconds waited by default.
const DEFAULT_WAIT_SECONDS: f64 = 90.0;
/// Most seconds one call waits: under two minutes, after which some MCP clients (Claude Code)
/// move a call to the background instead of handing its result to the agent.
const MAX_WAIT_SECONDS: f64 = 110.0;

/// The tool.
pub struct TranscribeMedia;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Ids of the media (assets) to transcribe, as `list_assets` or `get_transcript`'s
    /// `untranscribed` give them; default: the media played by the sound clips of `range`.
    pub asset_ids: Option<Vec<String>>,
    /// A range of the sequence whose sound clips' media are transcribed (ignored with
    /// `asset_ids`); default: your task's range, else the whole sequence.
    pub range: Option<RangeInput>,
    /// Sequence id or exact name; default: your task's sequence.
    pub sequence: Option<String>,
    /// Seconds to wait for the transcriptions to finish (default 90, at most 110; 0 starts them
    /// and answers at once).
    pub wait_seconds: Option<f64>,
}

#[derive(Debug, Serialize)]
struct Answer {
    sequence_id: String,
    /// Whether no transcription is still queued or running.
    finished: bool,
    /// Media this call started transcribing.
    started: usize,
    assets: Vec<AssetOutput>,
    note: String,
}

#[derive(Debug, Serialize)]
struct AssetOutput {
    asset_id: String,
    name: String,
    /// `transcribed`, `queued`, `running`, `failed`, `cancelled`, `unavailable`, `not_started`.
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    word_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    progress: Option<f64>,
    /// Why it failed or cannot be transcribed, and what to do.
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

impl McpTool for TranscribeMedia {
    type Input = Input;

    const NAME: &'static str = "transcribe_media";

    const DESCRIPTION: &'static str = "Transcribes media that have no transcript yet (locally, \
        with the user's installed speech model), then waits up to wait_seconds (default 90) for \
        the result. Default media: those played by the sound clips of your task's range, else \
        the whole sequence; or give asset_ids. Already transcribed media cost nothing; failed \
        ones are tried again. Use it when get_transcript lists untranscribed media you need. \
        finished=false: call it again with the same arguments to keep waiting. Changes no item \
        of the timeline.";

    const READ_ONLY: bool = false;

    const DESTRUCTIVE: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        let wait = wait_duration(input.wait_seconds)?;
        let project = context.project().await?;
        let sequence = find_sequence(&project, &context.agent, input.sequence.as_deref())?;
        let asset_ids = if let Some(ids) = &input.asset_ids {
            listed_assets(&project, ids)?
        } else {
            let range = match &input.range {
                Some(range) => range.to_range(sequence.format.frame_rate)?,
                None => default_range(&context.agent, sequence)?,
            };
            let clips = clips_in(sequence, TrackKind::Audio, range);
            distinct(clips.iter().map(|clip| clip.asset_id))
        };
        let round = if asset_ids.is_empty() {
            TranscriptionRound {
                started: Vec::new(),
                assets: Vec::new(),
            }
        } else {
            context
                .deps
                .transcription
                .transcribe_and_wait(context.agent.project_id, &asset_ids, wait)
                .await?
        };
        ToolOutput::json(&Answer {
            sequence_id: sequence.id.to_string(),
            finished: round.finished(),
            started: round.started.len(),
            note: note(&round),
            assets: round
                .assets
                .iter()
                .map(|asset| asset_output(&project, asset))
                .collect(),
        })
    }
}

/// The wait asked for, capped at [`MAX_WAIT_SECONDS`].
fn wait_duration(seconds: Option<f64>) -> Result<Duration, ToolError> {
    let seconds = seconds.unwrap_or(DEFAULT_WAIT_SECONDS);
    if !seconds.is_finite() || seconds < 0.0 {
        return Err(ToolError::invalid_arguments(format!(
            "wait_seconds must be between 0 and {MAX_WAIT_SECONDS}, got {seconds}"
        )));
    }
    Ok(Duration::from_secs_f64(seconds.min(MAX_WAIT_SECONDS)))
}

/// The assets named by `ids`, each once, in the order given.
fn listed_assets(project: &Project, ids: &[String]) -> Result<Vec<AssetId>, ToolError> {
    if ids.is_empty() {
        return Err(ToolError::invalid_arguments(
            "asset_ids is empty: leave it out to transcribe the sequence's media",
        ));
    }
    let found = ids
        .iter()
        .map(|id| find_asset(project, id).map(|asset| asset.id))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(distinct(found.into_iter()))
}

/// `ids` without repeats, in first-seen order.
fn distinct(ids: impl Iterator<Item = AssetId>) -> Vec<AssetId> {
    let mut unique = Vec::new();
    for id in ids {
        if !unique.contains(&id) {
            unique.push(id);
        }
    }
    unique
}

/// What the agent reads about one asset.
fn asset_output(project: &Project, asset: &AssetTranscription) -> AssetOutput {
    let name = project
        .assets
        .iter()
        .find(|candidate| candidate.id == asset.asset_id)
        .map_or_else(|| asset.asset_id.to_string(), asset_name);
    let mut output = AssetOutput {
        asset_id: asset.asset_id.to_string(),
        name,
        status: status_name(&asset.state),
        word_count: None,
        progress: None,
        reason: None,
    };
    match &asset.state {
        TranscriptionState::Transcribed { word_count } => output.word_count = Some(*word_count),
        TranscriptionState::Running { progress } => {
            output.progress = progress.map(|value| (value * 100.0).round() / 100.0);
        }
        TranscriptionState::Failed { message } => output.reason = Some(message.clone()),
        TranscriptionState::Cancelled => {
            output.reason = Some("cancelled (by the user, or the project closed)".to_owned());
        }
        TranscriptionState::Unavailable { reason } => {
            output.reason = Some(unavailable_reason(*reason).to_owned());
        }
        TranscriptionState::Queued | TranscriptionState::NotStarted => {}
    }
    output
}

/// The `status` of a state.
const fn status_name(state: &TranscriptionState) -> &'static str {
    match state {
        TranscriptionState::Transcribed { .. } => "transcribed",
        TranscriptionState::Queued => "queued",
        TranscriptionState::Running { .. } => "running",
        TranscriptionState::Failed { .. } => "failed",
        TranscriptionState::Cancelled => "cancelled",
        TranscriptionState::Unavailable { .. } => "unavailable",
        TranscriptionState::NotStarted => "not_started",
    }
}

/// Why an asset cannot be transcribed, with what to do.
const fn unavailable_reason(reason: UnavailableReason) -> &'static str {
    match reason {
        UnavailableReason::MediaMissing => {
            "media_missing: the original file is missing; ask the user to relink it"
        }
        UnavailableReason::NoSound => "no_sound: this media has no sound to transcribe",
        UnavailableReason::NotReady => {
            "not_ready: the media is still being imported; call transcribe_media again shortly"
        }
    }
}

/// What to do next, from where the round stands.
fn note(round: &TranscriptionRound) -> String {
    if round.assets.is_empty() {
        return "no sound clip in the range: nothing to transcribe".to_owned();
    }
    if !round.finished() {
        return "still transcribing: call transcribe_media again with the same arguments to keep \
            waiting (nothing starts twice), or do other work meanwhile"
            .to_owned();
    }
    let all_transcribed = round
        .assets
        .iter()
        .all(|asset| matches!(asset.state, TranscriptionState::Transcribed { .. }));
    if all_transcribed {
        "done: read what is said with get_transcript or search_transcript".to_owned()
    } else {
        "some media have no transcript: see each reason and tell the user what to do".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use se_domain::test_support::example_project;

    use super::*;

    #[test]
    fn waits_are_capped_and_checked() {
        assert_eq!(wait_duration(None).unwrap(), Duration::from_secs(90));
        assert_eq!(wait_duration(Some(0.0)).unwrap(), Duration::ZERO);
        assert_eq!(
            wait_duration(Some(9_999.0)).unwrap(),
            Duration::from_secs(110)
        );
        assert!(wait_duration(Some(-1.0)).is_err());
        assert!(wait_duration(Some(f64::NAN)).is_err());
    }

    #[test]
    fn listed_assets_are_found_once_each() {
        let project = example_project();
        let id = project.assets[0].id.to_string();
        let found = listed_assets(&project, &[id.clone(), id]).unwrap();
        assert_eq!(found, [project.assets[0].id]);
        assert_eq!(
            listed_assets(&project, &["nope".to_owned()])
                .unwrap_err()
                .code,
            "ASSET_NOT_FOUND"
        );
        assert_eq!(
            listed_assets(&project, &[]).unwrap_err().code,
            "INVALID_ARGUMENTS"
        );
    }

    #[test]
    fn the_note_says_what_to_do_next() {
        let asset = |state| AssetTranscription {
            asset_id: example_project().assets[0].id,
            state,
        };
        let round = |assets| TranscriptionRound {
            started: Vec::new(),
            assets,
        };
        assert!(note(&round(Vec::new())).starts_with("no sound clip"));
        let running = round(vec![asset(TranscriptionState::Running { progress: None })]);
        assert!(note(&running).starts_with("still transcribing"));
        let done = round(vec![asset(TranscriptionState::Transcribed {
            word_count: 3,
        })]);
        assert!(note(&done).starts_with("done"));
        let failed = round(vec![asset(TranscriptionState::Failed {
            message: "boom".to_owned(),
        })]);
        assert!(note(&failed).starts_with("some media"));
    }
}
