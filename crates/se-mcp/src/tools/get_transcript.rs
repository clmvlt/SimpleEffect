//! `get_transcript`: the words spoken in a range of the sequence, in sequence time, grouped in
//! segments (phrases), paginated by words. Only what the sequence's sound clips play counts: a
//! removed passage has no words (`se_domain::sequence_transcript`).

use schemars::JsonSchema;
use se_domain::{Project, SequenceWord, TimeRange, Transcript};
use serde::{Deserialize, Serialize};

use crate::{
    context::{ToolContext, default_range, find_sequence},
    describe::asset_name,
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::{RangeInput, RangeOutput, TimeOutput, seconds},
};

/// Words per page by default.
const DEFAULT_LIMIT: usize = 400;
/// Most words per page.
const MAX_LIMIT: usize = 2000;

/// The tool.
pub struct GetTranscript;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// A range of the sequence; default: your task's range, else the whole sequence.
    pub range: Option<RangeInput>,
    /// Sequence id or exact name; default: your task's sequence.
    pub sequence: Option<String>,
    /// Whether to list each word with its times (default true); false gives segments only,
    /// shorter for long ranges.
    pub words: Option<bool>,
    /// Index of the first word of the page (a previous answer's `next_offset`); default 0.
    pub offset: Option<usize>,
    /// Words per page (default 400, at most 2000).
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
struct Answer {
    sequence_id: String,
    time_base: &'static str,
    range: RangeOutput,
    /// Whether at least one sound clip in the range is transcribed.
    available: bool,
    segments: Vec<SegmentOutput>,
    total_words: usize,
    next_offset: Option<usize>,
    /// Assets of sound clips in the range without a transcript (`transcribe_media` transcribes
    /// them).
    untranscribed: Vec<AssetRef>,
}

#[derive(Debug, Serialize)]
struct SegmentOutput {
    start: TimeOutput,
    end: TimeOutput,
    text: String,
    item_id: String,
    asset_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    words: Option<Vec<WordOutput>>,
}

#[derive(Debug, Serialize)]
struct WordOutput {
    text: String,
    start_seconds: f64,
    end_seconds: f64,
    /// Set when the model was unsure of this word.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    unsure: bool,
}

#[derive(Debug, Serialize)]
struct AssetRef {
    asset_id: String,
    name: String,
}

impl McpTool for GetTranscript {
    type Input = Input;

    const NAME: &'static str = "get_transcript";

    const DESCRIPTION: &'static str = "The words spoken in a range of the sequence (default: \
        your task's range, else the whole sequence), in sequence time: segments (phrases) with \
        their text, and each word's start and end in seconds (words=false for segments only). \
        Only what the sound clips play counts: cut passages have no words. Paginated by words \
        (offset/limit, next_offset). untranscribed lists media without a transcript yet: \
        transcribe them with transcribe_media.";

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        let project = context.project().await?;
        let sequence = find_sequence(&project, &context.agent, input.sequence.as_deref())?;
        let range = match &input.range {
            Some(range) => range.to_range(sequence.format.frame_rate)?,
            None => default_range(&context.agent, sequence)?,
        };
        let limit = input.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
        let offset = input.offset.unwrap_or(0);
        let sequence_words = context
            .deps
            .transcription
            .sequence_words(context.agent.project_id, sequence.id)
            .await?;
        let words: Vec<&SequenceWord> = sequence_words.transcript.words_in(range).collect();
        let page: Vec<&SequenceWord> = words.iter().skip(offset).take(limit).copied().collect();
        let next = offset + page.len();
        ToolOutput::json(&Answer {
            sequence_id: sequence.id.to_string(),
            time_base: "sequence",
            range: range.into(),
            available: !words.is_empty() || sequence_words.untranscribed.is_empty(),
            segments: segments(&page, input.words.unwrap_or(true)),
            total_words: words.len(),
            next_offset: (next < words.len()).then_some(next),
            untranscribed: untranscribed_in(
                &project,
                sequence,
                range,
                &sequence_words.untranscribed,
            ),
        })
    }
}

/// The page's words grouped by their sequence segment.
fn segments(page: &[&SequenceWord], with_words: bool) -> Vec<SegmentOutput> {
    let mut segments: Vec<(u32, Vec<&SequenceWord>)> = Vec::new();
    for word in page {
        match segments.last_mut() {
            Some((segment, words)) if *segment == word.segment => words.push(word),
            _ => segments.push((word.segment, vec![word])),
        }
    }
    segments
        .into_iter()
        .filter_map(|(_, words)| {
            let (first, last) = (words.first()?, words.last()?);
            Some(SegmentOutput {
                start: first.start.into(),
                end: last.end.into(),
                text: words
                    .iter()
                    .map(|word| word.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
                item_id: first.item_id.to_string(),
                asset_id: first.asset_id.to_string(),
                words: with_words.then(|| {
                    words
                        .iter()
                        .map(|word| WordOutput {
                            text: word.text.clone(),
                            start_seconds: seconds(word.start),
                            end_seconds: seconds(word.end),
                            unsure: word.confidence < Transcript::LOW_CONFIDENCE,
                        })
                        .collect()
                }),
            })
        })
        .collect()
}

/// The untranscribed assets of sound clips intersecting `range`.
fn untranscribed_in(
    project: &Project,
    sequence: &se_domain::Sequence,
    range: TimeRange,
    untranscribed: &[se_domain::AssetId],
) -> Vec<AssetRef> {
    let mut found = Vec::new();
    for item in sequence
        .tracks
        .iter()
        .filter(|track| track.kind == se_domain::TrackKind::Audio)
        .flat_map(|track| &track.items)
        .filter(|item| item.timeline_range.intersects(range))
    {
        let se_domain::ItemKind::MediaClip(clip) = &item.kind else {
            continue;
        };
        if untranscribed.contains(&clip.asset_id) && !found.contains(&clip.asset_id) {
            found.push(clip.asset_id);
        }
    }
    found
        .into_iter()
        .filter_map(|id| project.assets.iter().find(|asset| asset.id == id))
        .map(|asset| AssetRef {
            asset_id: asset.id.to_string(),
            name: asset_name(asset),
        })
        .collect()
}
