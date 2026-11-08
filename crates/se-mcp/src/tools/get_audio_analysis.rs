//! `get_audio_analysis`: silences, loudness and peaks — of an asset (source time) or of a range
//! of the sequence, brought back to sequence time through each clip's source window and speed.

use schemars::JsonSchema;
use se_domain::{AudioAnalysis, Flicks, SilenceSettings, TimeRange, TrackKind};
use serde::{Deserialize, Serialize};

use crate::{
    analyses::{AnalysisCache, RangeClip, clips_in},
    context::{ToolContext, default_range, find_asset, find_sequence},
    describe::asset_name,
    output::{ToolError, ToolOutput, codes},
    registry::McpTool,
    time_io::{RangeInput, RangeOutput, TimeOutput},
    timing::{complement, merge},
};

/// Most ranges listed in one answer (silences).
const MAX_LISTED: usize = 100;
/// Loudness points by default.
const DEFAULT_POINTS: usize = 40;
/// Most loudness points.
const MAX_POINTS: usize = 200;
/// Loudest moments listed for an asset.
const LOUDEST_COUNT: usize = 5;

/// The tool.
pub struct GetAudioAnalysis;

/// Arguments: an `asset`, or a `range` of the sequence (the default: your task's range, else the
/// whole sequence).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// An asset's id: its whole analysis, in source time.
    pub asset: Option<String>,
    /// A range of the sequence: silences of the mix and of each sound clip, in sequence time.
    pub range: Option<RangeInput>,
    /// Sequence id or exact name (with `range`); default: your task's sequence.
    pub sequence: Option<String>,
    /// Points of the loudness curve of an asset; default 40, at most 200.
    pub max_points: Option<usize>,
}

#[derive(Debug, Serialize)]
struct AssetAudio {
    asset_id: String,
    name: String,
    time_base: &'static str,
    integrated_lufs: Option<f64>,
    loudness_range_lu: Option<f64>,
    true_peak_dbtp: Option<f64>,
    sample_peak_dbfs: Option<f64>,
    silence_threshold_dbfs: f64,
    min_silence: TimeOutput,
    silences: Vec<RangeOutput>,
    silence_total: usize,
    /// Loudest short-term loudness (LUFS) of each equal slice of the file.
    loudness: Vec<LoudnessPoint>,
    loudest: Vec<LoudnessPoint>,
}

#[derive(Debug, Serialize)]
struct LoudnessPoint {
    time: TimeOutput,
    lufs: f64,
}

#[derive(Debug, Serialize)]
struct SequenceAudio {
    sequence_id: String,
    time_base: &'static str,
    range: RangeOutput,
    /// Where nothing is heard: no sound clip on an unmuted track, or only their silences.
    silences: Vec<RangeOutput>,
    silence_total: usize,
    clips: Vec<ClipAudio>,
    /// Clips whose sound is not analysed yet: they count as sound in `silences`.
    unanalysed: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ClipAudio {
    item_id: String,
    track_id: String,
    asset_id: String,
    name: String,
    /// The clip's part inside the range.
    range: RangeOutput,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    muted: bool,
    analysed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    integrated_lufs: Option<f64>,
    /// Loudest short-term loudness (LUFS) of the part played.
    #[serde(skip_serializing_if = "Option::is_none")]
    loudest_lufs: Option<f64>,
    silences: Vec<RangeOutput>,
}

impl McpTool for GetAudioAnalysis {
    type Input = Input;

    const NAME: &'static str = "get_audio_analysis";

    const DESCRIPTION: &'static str = "Silences, loudness (EBU R128) and peaks. With `asset`: \
        the file's analysis in source time (silences, loudness curve, loudest moments). \
        Otherwise for a range of the sequence (default: your task's range, else the whole \
        sequence), in sequence time: where nothing is heard, and each sound clip's silences and \
        loudness. Analyses run in the background after import: NOT_ANALYSED until done.";

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
            let analysis = cache.audio(asset.id).await?.ok_or_else(|| {
                ToolError::new(
                    codes::NOT_ANALYSED,
                    format!(
                        "the sound of {} is not analysed (yet, or it has none): analyses run in \
                         the background after import; try again in a moment",
                        asset_name(asset)
                    ),
                )
            })?;
            let points = input
                .max_points
                .unwrap_or(DEFAULT_POINTS)
                .clamp(1, MAX_POINTS);
            return ToolOutput::json(&asset_audio(
                asset.id.to_string(),
                asset_name(asset),
                analysis,
                points,
            ));
        }
        let sequence = find_sequence(&project, &context.agent, input.sequence.as_deref())?;
        let range = match &input.range {
            Some(range) => range.to_range(sequence.format.frame_rate)?,
            None => default_range(&context.agent, sequence)?,
        };
        let clips = clips_in(sequence, TrackKind::Audio, range);
        let mut described = Vec::with_capacity(clips.len());
        let mut sounding = Vec::new();
        let mut unanalysed = Vec::new();
        for clip in &clips {
            let analysis = cache.audio(clip.asset_id).await?;
            let name = project
                .assets
                .iter()
                .find(|asset| asset.id == clip.asset_id)
                .map(asset_name)
                .unwrap_or_default();
            let silences = analysis.map(|analysis| clip_silences(clip, analysis, range));
            if !clip.track.muted {
                if let Some(silences) = &silences {
                    sounding.extend(complement(silences.clone(), clip.visible));
                } else {
                    sounding.push(clip.visible);
                    unanalysed.push(clip.item.id.to_string());
                }
            }
            described.push(clip_audio(
                clip,
                name,
                analysis,
                silences.unwrap_or_default(),
            ));
        }
        let min = SilenceSettings::SPEECH.min_duration;
        let quiet: Vec<TimeRange> = complement(sounding, range)
            .into_iter()
            .filter(|silence| silence.duration() >= min)
            .collect();
        ToolOutput::json(&SequenceAudio {
            sequence_id: sequence.id.to_string(),
            time_base: "sequence",
            range: range.into(),
            silence_total: quiet.len(),
            silences: quiet.into_iter().take(MAX_LISTED).map(Into::into).collect(),
            clips: described,
            unanalysed,
        })
    }
}

/// The silences of `clip`'s source, in sequence time, inside `range`.
fn clip_silences(
    clip: &RangeClip<'_>,
    analysis: &AudioAnalysis,
    range: TimeRange,
) -> Vec<TimeRange> {
    merge(
        analysis
            .silences
            .iter()
            .filter_map(|silence| clip.window.map_range(*silence, range))
            .collect(),
    )
}

fn clip_audio(
    clip: &RangeClip<'_>,
    name: String,
    analysis: Option<&AudioAnalysis>,
    silences: Vec<TimeRange>,
) -> ClipAudio {
    let played = clip
        .window
        .source
        .intersection(source_of(clip))
        .unwrap_or(clip.window.source);
    ClipAudio {
        item_id: clip.item.id.to_string(),
        track_id: clip.track.id.to_string(),
        asset_id: clip.asset_id.to_string(),
        name,
        range: clip.visible.into(),
        muted: clip.track.muted,
        analysed: analysis.is_some(),
        integrated_lufs: analysis
            .and_then(|analysis| analysis.integrated_lufs)
            .map(round1),
        loudest_lufs: analysis.and_then(|analysis| loudest_in(analysis, played)),
        silences: silences
            .into_iter()
            .take(MAX_LISTED)
            .map(Into::into)
            .collect(),
    }
}

/// The source range of the clip's visible part.
fn source_of(clip: &RangeClip<'_>) -> TimeRange {
    let speed = clip.window.speed;
    let offset = clip
        .visible
        .start()
        .checked_sub(clip.window.timeline.start())
        .and_then(|offset| offset.checked_mul_f64(speed))
        .unwrap_or(Flicks::ZERO);
    let length = clip
        .visible
        .duration()
        .checked_mul_f64(speed)
        .unwrap_or(Flicks::ZERO);
    clip.window
        .source
        .start()
        .checked_add(offset)
        .and_then(|start| TimeRange::new(start, length).ok())
        .unwrap_or(clip.window.source)
}

fn asset_audio(
    asset_id: String,
    name: String,
    analysis: &AudioAnalysis,
    points: usize,
) -> AssetAudio {
    let curve = &analysis.short_term;
    let samples: Vec<(Flicks, f64)> = curve
        .values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            let time = curve.step.checked_mul(i64::try_from(index).ok()?)?;
            value.map(|lufs| (time, lufs))
        })
        .collect();
    let mut loudest = samples.clone();
    loudest.sort_by(|a, b| b.1.total_cmp(&a.1));
    AssetAudio {
        asset_id,
        name,
        time_base: "source",
        integrated_lufs: analysis.integrated_lufs.map(round1),
        loudness_range_lu: analysis.loudness_range_lu.map(round1),
        true_peak_dbtp: analysis.true_peak_dbtp.map(round1),
        sample_peak_dbfs: analysis.sample_peak_dbfs.map(round1),
        silence_threshold_dbfs: analysis.silence_settings.threshold_dbfs,
        min_silence: analysis.silence_settings.min_duration.into(),
        silence_total: analysis.silences.len(),
        silences: analysis
            .silences
            .iter()
            .take(MAX_LISTED)
            .copied()
            .map(Into::into)
            .collect(),
        loudness: downsample(&samples, points),
        loudest: loudest
            .into_iter()
            .take(LOUDEST_COUNT)
            .map(|(time, lufs)| LoudnessPoint {
                time: time.into(),
                lufs: round1(lufs),
            })
            .collect(),
    }
}

/// At most `points` points: the loudest sample of each equal slice of `samples`.
fn downsample(samples: &[(Flicks, f64)], points: usize) -> Vec<LoudnessPoint> {
    if samples.is_empty() {
        return Vec::new();
    }
    let slices = points.min(samples.len());
    (0..slices)
        .filter_map(|slice| {
            let from = slice * samples.len() / slices;
            let to = ((slice + 1) * samples.len() / slices).max(from + 1);
            let (time, _) = samples.get(from)?;
            let lufs = samples
                .get(from..to)?
                .iter()
                .map(|(_, lufs)| *lufs)
                .fold(f64::NEG_INFINITY, f64::max);
            Some(LoudnessPoint {
                time: (*time).into(),
                lufs: round1(lufs),
            })
        })
        .collect()
}

/// The loudest short-term loudness within source range `played`.
fn loudest_in(analysis: &AudioAnalysis, played: TimeRange) -> Option<f64> {
    let step = analysis.short_term.step;
    analysis
        .short_term
        .values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            let time = step.checked_mul(i64::try_from(index).ok()?)?;
            played.contains(time).then_some(*value).flatten()
        })
        .fold(None, |loudest: Option<f64>, lufs| {
            Some(loudest.map_or(lufs, |loudest| loudest.max(lufs)))
        })
        .map(round1)
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use se_domain::test_support::millis;

    use super::*;

    #[test]
    fn downsampling_keeps_the_loudest_of_each_slice() {
        let samples: Vec<(Flicks, f64)> = (0..10)
            .map(|index| (millis(index * 500), f64::from(u8::try_from(index).unwrap())))
            .collect();
        let points = downsample(&samples, 5);
        let values: Vec<f64> = points.iter().map(|point| point.lufs).collect();
        assert_eq!(values, [1.0, 3.0, 5.0, 7.0, 9.0]);
        assert_eq!(points[1].time.timecode, "00:00:01.000");
        assert_eq!(downsample(&samples, 50).len(), 10);
        assert!(downsample(&[], 5).is_empty());
    }
}
