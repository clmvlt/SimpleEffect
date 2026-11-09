//! `auto_reframe`: makes clips follow their subject (prompt 26, ADR-0037). Without
//! `target_format` it reframes media clips of the task's scope; with it, it adds a version of the
//! task's sequence in that format, after it, whose clips follow their subject (the one addition
//! the scope allows outside the task's sequence; reverting the task removes it). The faces of
//! the media are analysed first (once per media), within `wait_seconds`. Not batchable.

use std::{collections::BTreeMap, time::Duration};

use schemars::JsonSchema;
use se_app::{
    agent_edits::{AgentEdit, ItemRef, ReframeClips, ReframedVersion},
    reframing::{AnalysesRound, AssetAnalyses, video_clips},
};
use se_domain::{
    AssetId, Format, FormatPreset, Item, ItemId, ItemKind, Project, Sequence, TrackingSettings,
};
use serde::{Deserialize, Serialize};

use crate::{
    context::{ToolContext, find_sequence},
    describe::asset_name,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    writes::{EditOutput, apply},
};

/// Seconds waited by default for the face analyses.
const DEFAULT_WAIT_SECONDS: f64 = 90.0;
/// Most seconds one call waits (under Claude Code's two-minute auto-background).
const MAX_WAIT_SECONDS: f64 = 110.0;

/// The tool.
pub struct AutoReframe;

/// A format: a preset name or a size.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum FormatInput {
    /// `"9:16"` (1080 × 1920), `"16:9"` (1920 × 1080), `"1:1"` (1080 × 1080) or `"4:5"`
    /// (1080 × 1350).
    Preset(String),
    /// A size in pixels (even, 2–16384).
    Size {
        /// Width in pixels.
        width: u32,
        /// Height in pixels.
        height: u32,
    },
}

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Ids of media clips (on video tracks of your scope) to reframe; default: every media clip
    /// of a video track lying in your scope. Ignored with `target_format`.
    pub item_ids: Option<Vec<String>>,
    /// Create a version of your task's sequence in this format instead (`"9:16"`, `"1:1"`,
    /// `"4:5"`, `"16:9"` or `{width, height}`), at the sequence's frame rate, every clip
    /// filling the frame and following its subject.
    pub target_format: Option<FormatInput>,
    /// Name of the new version (default: the sequence's name and the format, `Interview 9:16`).
    pub name: Option<String>,
    /// How smooth the frame's moves are, 0 (follows closely) to 1 (slow, steady); default 0.5.
    pub smoothing: Option<f64>,
    /// Seconds to wait for the face analyses (default 90, at most 110; 0 starts them and
    /// answers at once).
    pub wait_seconds: Option<f64>,
}

#[derive(Debug, Serialize)]
struct Answer {
    /// Whether the faces of every media were analysed (false: call again to keep waiting).
    finished: bool,
    /// The version created, when `target_format` was given and the analyses finished.
    #[serde(skip_serializing_if = "Option::is_none")]
    version_sequence_id: Option<String>,
    /// The edit, when one was made.
    #[serde(skip_serializing_if = "Option::is_none")]
    edit: Option<EditOutput>,
    /// Media still being analysed.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    analysing: Vec<String>,
    /// Media whose faces could not be analysed (their clips stay as they are, or centred in a
    /// version), and why.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    failed: Vec<FailedMedia>,
    note: String,
}

#[derive(Debug, Serialize)]
struct FailedMedia {
    asset_id: String,
    name: String,
    reason: String,
}

impl McpTool for AutoReframe {
    type Input = Input;

    const NAME: &'static str = "auto_reframe";

    const DESCRIPTION: &'static str = "Makes media clips follow their subject (faces): the \
        frame's window moves with the speaker, smoothly, with a jump at each shot change, as \
        editable reframeFocus keyframes (reframe tracked). Without target_format: reframes the \
        media clips of your scope (or item_ids). With target_format (\"9:16\", \"1:1\", \
        \"4:5\", \"16:9\"): creates a version of your task's sequence in that format whose clips \
        follow their subject — e.g. a vertical cut of a landscape interview. Faces are analysed \
        locally first (once per media; may take a while): finished=false means call again with \
        the same arguments. Check the result with get_frames. Not batchable.";

    const READ_ONLY: bool = false;

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        let wait = wait_duration(input.wait_seconds)?;
        let settings = settings(input.smoothing)?;
        let project = context.project().await?;
        let sequence = find_sequence(&project, &context.agent, None)?;
        let version = input
            .target_format
            .as_ref()
            .map(|format| target_format(format, sequence))
            .transpose()?;
        let clips = if version.is_some() {
            video_clips(sequence)
        } else {
            chosen_clips(&project, sequence, &context, input.item_ids.as_deref())?
        };
        let assets = clip_assets(&project, &clips);
        let round = context
            .deps
            .reframing
            .analyses_for(context.agent.project_id, &assets, wait)
            .await?;
        if !round.waiting.is_empty() {
            return ToolOutput::json(&waiting_answer(&project, &round));
        }
        let failed = failures(&project, &round);
        let is_version = version.is_some();
        let edit = if let Some(format) = version {
            let name = input
                .name
                .clone()
                .unwrap_or_else(|| default_name(sequence, input.target_format.as_ref()));
            AgentEdit::AddReframedVersion(Box::new(ReframedVersion {
                name,
                format,
                settings,
                analyses: round.ready,
            }))
        } else {
            let ready = ready_clips(&project, &clips, &round.ready);
            if ready.is_empty() {
                return ToolOutput::json(&nothing_answer(failed));
            }
            AgentEdit::ReframeClips(Box::new(ReframeClips {
                items: ready.into_iter().map(ItemRef::Id).collect(),
                settings,
                analyses: round.ready,
            }))
        };
        let report = apply(&context, Self::NAME, &[edit], |_| None).await?;
        let created: Vec<ItemId> = report.created.iter().flatten().copied().collect();
        let version_sequence_id = is_version
            .then(|| version_of(&report.project, &created))
            .flatten();
        ToolOutput::json(&Answer {
            finished: true,
            note: done_note(is_version, failed.len()),
            version_sequence_id,
            edit: Some(EditOutput::new(&report, &created)),
            analysing: Vec::new(),
            failed,
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

/// The default settings with `smoothing`.
fn settings(smoothing: Option<f64>) -> Result<TrackingSettings, ToolError> {
    let settings = TrackingSettings {
        smoothing: smoothing.unwrap_or(TrackingSettings::default().smoothing),
        ..TrackingSettings::default()
    };
    if settings.problems().is_empty() {
        Ok(settings)
    } else {
        Err(ToolError::invalid_arguments(format!(
            "smoothing must be between 0 and 1, got {}",
            settings.smoothing
        )))
    }
}

/// The format of `input` at `sequence`'s frame and sample rates.
fn target_format(input: &FormatInput, sequence: &Sequence) -> Result<Format, ToolError> {
    let (width, height) = match input {
        FormatInput::Preset(name) => {
            let preset = match name.trim() {
                "9:16" => FormatPreset::Vertical9x16,
                "16:9" => FormatPreset::Hd16x9,
                "1:1" => FormatPreset::Square1x1,
                "4:5" => FormatPreset::Portrait4x5,
                other => {
                    return Err(ToolError::invalid_arguments(format!(
                        "unknown format '{other}': use \"9:16\", \"16:9\", \"1:1\", \"4:5\" or \
                         {{\"width\": …, \"height\": …}}"
                    )));
                }
            };
            let format = Format::from_preset(preset, sequence.format.frame_rate)
                .ok_or_else(|| ToolError::invalid_arguments(format!("no size for {name}")))?;
            (format.width, format.height)
        }
        FormatInput::Size { width, height } => (*width, *height),
    };
    let even = |side: u32| (2..=16_384).contains(&side) && side.is_multiple_of(2);
    if !even(width) || !even(height) {
        return Err(ToolError::invalid_arguments(format!(
            "{width} × {height} is not a valid size: give even sides between 2 and 16384"
        )));
    }
    Ok(Format {
        width,
        height,
        ..sequence.format
    })
}

/// `Interview 9:16`.
fn default_name(sequence: &Sequence, format: Option<&FormatInput>) -> String {
    let suffix = match format {
        Some(FormatInput::Preset(name)) => name.trim().to_owned(),
        Some(FormatInput::Size { width, height }) => format!("{width}×{height}"),
        None => String::new(),
    };
    format!("{} {suffix}", sequence.name).trim().to_owned()
}

/// The clips named by `ids`, or the media clips of video tracks lying in the task's scope.
fn chosen_clips(
    project: &Project,
    sequence: &Sequence,
    context: &ToolContext,
    ids: Option<&[String]>,
) -> Result<Vec<ItemId>, ToolError> {
    if let Some(ids) = ids {
        if ids.is_empty() {
            return Err(ToolError::invalid_arguments(
                "item_ids is empty: leave it out to reframe every clip of your scope",
            ));
        }
        return ids
            .iter()
            .map(|id| {
                let (_, _, item) = crate::context::find_item(project, id)?;
                Ok(item.id)
            })
            .collect();
    }
    let scope = &context.agent.scope;
    let in_scope = |track: &se_domain::Track, item: &Item| {
        scope
            .tracks
            .as_ref()
            .is_none_or(|tracks| tracks.contains(&track.id))
            && scope
                .range
                .is_none_or(|range| range.contains_range(item.timeline_range))
    };
    Ok(sequence
        .tracks
        .iter()
        .filter(|track| track.kind == se_domain::TrackKind::Video)
        .flat_map(|track| track.items.iter().map(move |item| (track, item)))
        .filter(|(track, item)| {
            matches!(item.kind, ItemKind::MediaClip(_)) && in_scope(track, item)
        })
        .map(|(_, item)| item.id)
        .collect())
}

/// The distinct media of `clips`.
fn clip_assets(project: &Project, clips: &[ItemId]) -> Vec<AssetId> {
    let mut assets: Vec<AssetId> = project
        .items()
        .filter(|item| clips.contains(&item.id))
        .filter_map(|item| match &item.kind {
            ItemKind::MediaClip(clip) => Some(clip.asset_id),
            _ => None,
        })
        .collect();
    assets.sort_unstable();
    assets.dedup();
    assets
}

/// The clips of `clips` whose media is analysed.
fn ready_clips(
    project: &Project,
    clips: &[ItemId],
    ready: &BTreeMap<AssetId, AssetAnalyses>,
) -> Vec<ItemId> {
    project
        .items()
        .filter(|item| clips.contains(&item.id))
        .filter(|item| matches!(&item.kind, ItemKind::MediaClip(clip) if ready.contains_key(&clip.asset_id)))
        .map(|item| item.id)
        .collect()
}

/// The sequence holding the first created item.
fn version_of(project: &Project, created: &[ItemId]) -> Option<String> {
    let first = created.first()?;
    project
        .sequences
        .iter()
        .find(|sequence| {
            sequence
                .tracks
                .iter()
                .any(|track| track.items.iter().any(|item| item.id == *first))
        })
        .map(|sequence| sequence.id.to_string())
}

/// The file name of a media of the project (its id when it is gone).
fn media_name(project: &Project, asset_id: AssetId) -> String {
    project
        .assets
        .iter()
        .find(|asset| asset.id == asset_id)
        .map_or_else(|| asset_id.to_string(), asset_name)
}

fn failures(project: &Project, round: &AnalysesRound) -> Vec<FailedMedia> {
    round
        .failed
        .iter()
        .map(|failure| FailedMedia {
            asset_id: failure.asset_id.to_string(),
            name: media_name(project, failure.asset_id),
            reason: failure.message.clone(),
        })
        .collect()
}

fn waiting_answer(project: &Project, round: &AnalysesRound) -> Answer {
    Answer {
        finished: false,
        version_sequence_id: None,
        edit: None,
        analysing: round
            .waiting
            .iter()
            .map(|asset| media_name(project, *asset))
            .collect(),
        failed: failures(project, round),
        note: "the faces of these media are still being analysed: call auto_reframe again with \
               the same arguments to keep waiting (nothing changed yet)"
            .to_owned(),
    }
}

fn nothing_answer(failed: Vec<FailedMedia>) -> Answer {
    Answer {
        finished: true,
        version_sequence_id: None,
        edit: None,
        analysing: Vec::new(),
        note: if failed.is_empty() {
            "no media clip of a video track lies in your scope: nothing to reframe".to_owned()
        } else {
            "no clip could be reframed: see failed".to_owned()
        },
        failed,
    }
}

fn done_note(version: bool, failures: usize) -> String {
    let what = if version {
        "the version was created; its clips follow their subject (reframe tracked, editable \
         reframeFocus keyframes)"
    } else {
        "the clips follow their subject (reframe tracked, editable reframeFocus keyframes)"
    };
    if failures == 0 {
        format!("{what}: check with get_frames")
    } else {
        format!("{what}, except those of the failed media: check with get_frames")
    }
}

#[cfg(test)]
mod tests {
    use se_domain::test_support::example_project;

    use super::*;

    #[test]
    fn formats_keep_the_sequence_rate() {
        let project = example_project();
        let sequence = &project.sequences[0];
        let vertical = target_format(&FormatInput::Preset("9:16".to_owned()), sequence).unwrap();
        assert_eq!((vertical.width, vertical.height), (1080, 1920));
        assert_eq!(vertical.frame_rate, sequence.format.frame_rate);
        let size = target_format(
            &FormatInput::Size {
                width: 720,
                height: 1280,
            },
            sequence,
        )
        .unwrap();
        assert_eq!(size.height, 1280);
        assert!(target_format(&FormatInput::Preset("21:9".to_owned()), sequence).is_err());
        assert!(
            target_format(
                &FormatInput::Size {
                    width: 721,
                    height: 1280
                },
                sequence
            )
            .is_err()
        );
        assert_eq!(
            default_name(sequence, Some(&FormatInput::Preset("9:16".to_owned()))),
            format!("{} 9:16", sequence.name)
        );
    }

    #[test]
    fn smoothing_is_checked() {
        assert!((settings(None).unwrap().smoothing - 0.5).abs() < 1e-12);
        assert!(settings(Some(1.5)).is_err());
        assert!(wait_duration(Some(500.0)).unwrap() <= Duration::from_secs(110));
        assert!(wait_duration(Some(-1.0)).is_err());
    }
}
