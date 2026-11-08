//! `get_timeline`: the items of a sequence that intersect a range, with their key properties, in
//! time order; transitions and markers of the range; paginated.

use schemars::JsonSchema;
use se_domain::{Flicks, Sequence, TimeRange, Track};
use serde::{Deserialize, Serialize};

use crate::{
    context::{ToolContext, default_range, find_sequence, find_tracks},
    describe::{ItemSummary, TrackSummary, item_summary, track_summary},
    output::{ToolError, ToolOutput},
    registry::McpTool,
    time_io::{RangeInput, RangeOutput, TimeOutput},
};

/// Items per page by default.
const DEFAULT_LIMIT: usize = 50;
/// Most items per page.
const MAX_LIMIT: usize = 200;

/// The tool.
pub struct GetTimeline;

/// Arguments.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Sequence id or exact name; default: your task's sequence.
    pub sequence: Option<String>,
    /// Range of the sequence to list; default: your task's range, else the whole sequence.
    pub range: Option<RangeInput>,
    /// Track ids or names to list; default: every track.
    pub tracks: Option<Vec<String>>,
    /// Items to skip (from a previous page's `next_offset`); default 0.
    pub offset: Option<usize>,
    /// Items per page; default 50, at most 200.
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
struct Timeline {
    sequence: SequenceHeader,
    range: RangeOutput,
    tracks: Vec<TrackSummary>,
    /// Items intersecting the range, by start then track (bottom first).
    items: Vec<ItemSummary>,
    total_items: usize,
    /// Pass it as `offset` for the next page; `null` when this is the last one.
    next_offset: Option<usize>,
    transitions: Vec<TransitionSummary>,
    markers: Vec<MarkerSummary>,
}

#[derive(Debug, Serialize)]
struct SequenceHeader {
    sequence_id: String,
    name: String,
    fps: f64,
    duration: TimeOutput,
}

#[derive(Debug, Serialize)]
struct TransitionSummary {
    transition_id: String,
    track_id: String,
    kind: String,
    left_item_id: String,
    right_item_id: String,
    /// The cut it covers (end of the left item).
    at: TimeOutput,
    duration: TimeOutput,
}

#[derive(Debug, Serialize)]
struct MarkerSummary {
    marker_id: String,
    time: TimeOutput,
    label: String,
}

impl McpTool for GetTimeline {
    type Input = Input;

    const NAME: &'static str = "get_timeline";

    const DESCRIPTION: &'static str = "Items of a sequence that intersect a range (default: your \
        task's range, else the whole sequence), in time order: ids, revisions, kind, track, \
        start/end, label (media name, text, shape), source range and speed of clips, animated \
        properties, origin. Also the tracks, transitions and markers of the range. Paginated: \
        follow next_offset.";

    async fn call(context: ToolContext, input: Input) -> Result<ToolOutput, ToolError> {
        let project = context.project().await?;
        let sequence = find_sequence(&project, &context.agent, input.sequence.as_deref())?;
        let tracks = find_tracks(sequence, input.tracks.as_deref())?;
        let rate = sequence.format.frame_rate;
        let range = match &input.range {
            Some(range) => range.to_range(rate)?,
            None => default_range(&context.agent, sequence)?,
        };
        let mut entries: Vec<(Flicks, usize, ItemSummary)> = Vec::new();
        for (order, track) in tracks.iter().enumerate() {
            for item in &track.items {
                if overlaps(item.timeline_range, range) {
                    let summary = item_summary(&project, track, item);
                    entries.push((item.timeline_range.start(), order, summary));
                }
            }
        }
        entries.sort_by_key(|(start, order, _)| (*start, *order));
        let total_items = entries.len();
        let offset = input.offset.unwrap_or(0).min(total_items);
        let limit = input.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
        let items: Vec<ItemSummary> = entries
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|(_, _, summary)| summary)
            .collect();
        let next = offset + items.len();
        ToolOutput::json(&Timeline {
            sequence: SequenceHeader {
                sequence_id: sequence.id.to_string(),
                name: sequence.name.clone(),
                fps: (rate.as_f64() * 1000.0).round() / 1000.0,
                duration: sequence.content_end().into(),
            },
            range: range.into(),
            tracks: tracks.iter().map(|track| track_summary(track)).collect(),
            items,
            total_items,
            next_offset: (next < total_items).then_some(next),
            transitions: tracks
                .iter()
                .flat_map(|track| transitions_in(track, range))
                .collect(),
            markers: markers_in(sequence, range),
        })
    }
}

fn overlaps(item: TimeRange, range: TimeRange) -> bool {
    item.intersects(range)
}

fn transitions_in(track: &Track, range: TimeRange) -> Vec<TransitionSummary> {
    track
        .transitions
        .iter()
        .filter_map(|transition| {
            let left = track
                .items
                .iter()
                .find(|item| item.id == transition.left_item)?;
            let cut = left.timeline_range.end();
            (range.contains(cut) || cut == range.end()).then(|| TransitionSummary {
                transition_id: transition.id.to_string(),
                track_id: track.id.to_string(),
                kind: serde_json::to_value(transition.kind)
                    .ok()
                    .and_then(|value| value.as_str().map(str::to_owned))
                    .unwrap_or_default(),
                left_item_id: transition.left_item.to_string(),
                right_item_id: transition.right_item.to_string(),
                at: cut.into(),
                duration: transition.duration.into(),
            })
        })
        .collect()
}

fn markers_in(sequence: &Sequence, range: TimeRange) -> Vec<MarkerSummary> {
    let mut markers: Vec<&se_domain::Marker> = sequence
        .markers
        .iter()
        .filter(|marker| range.contains(marker.time))
        .collect();
    markers.sort_by_key(|marker| marker.time);
    markers
        .into_iter()
        .map(|marker| MarkerSummary {
            marker_id: marker.id.to_string(),
            time: marker.time.into(),
            label: marker.label.clone(),
        })
        .collect()
}
