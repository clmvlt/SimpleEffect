//! Resolving a [`CutRequest`] against a document: which ranges it removes, decided from the
//! sequence as it is now (words mapped through the current clips, silences read through them).

use std::collections::BTreeSet;

use se_domain::{AssetId, Flicks, Project, Sequence, TimeRange};
use se_protocol::{FillerList, FillerOccurrence};

pub use super::silence_cuts::silence_tracks;
use super::{
    CutMedia, CutRequest, CutTarget, ResolvedCuts,
    silence_cuts::{SilenceListening, silence_cuts},
    word_context::WordContext,
    words::words_in_ranges,
};
use crate::{AppError, edit_plan::find_sequence};

/// Less than this left between two cuts, or at a clip edge, is removed too.
pub const MIN_KEEP: Flicks = Flicks::from_millis(80);

/// The cuts `request` makes on `project`.
///
/// # Errors
/// Unknown sequence or tracks, a track of another kind for silences.
pub fn resolve(
    project: &Project,
    request: &CutRequest,
    media: &CutMedia,
) -> Result<ResolvedCuts, AppError> {
    let sequence = find_sequence(project, request.sequence_id)?;
    let Some(limit) = limit_of(sequence, request.limit) else {
        return Ok(ResolvedCuts::default());
    };
    match &request.target {
        CutTarget::Words { ranges } => {
            let context = WordContext::new(sequence, media, limit);
            let removed = words_in_ranges(&context.transcript.words, ranges);
            Ok(context.cuts(sequence, media, &removed))
        }
        CutTarget::Fillers {
            range,
            lists,
            max_duration,
        } => {
            let Some(range) = clipped(*range, limit) else {
                return Ok(ResolvedCuts::default());
            };
            let context = WordContext::new(sequence, media, range);
            let spans = context.fillers(media, lists, *max_duration, range);
            let removed = spans
                .iter()
                .flat_map(|span| span.first..=span.last)
                .collect();
            Ok(context.cuts(sequence, media, &removed))
        }
        CutTarget::Silences {
            range,
            params,
            tracks,
        } => {
            let (Some(whole), Some(range)) = (limit_of(sequence, None), clipped(*range, limit))
            else {
                return Ok(ResolvedCuts::default());
            };
            let listen = SilenceListening {
                params: *params,
                tracks: tracks.as_deref(),
                whole,
            };
            silence_cuts(sequence, media, &listen, range)
        }
    }
}

/// The filler words of `range` (`None`: the whole sequence) with `lists`, for the panel.
///
/// # Errors
/// Unknown sequence.
pub fn filler_occurrences(
    project: &Project,
    sequence_id: se_domain::SequenceId,
    range: Option<TimeRange>,
    media: &CutMedia,
    lists: &[FillerList],
    max_duration: Flicks,
) -> Result<(Vec<FillerOccurrence>, BTreeSet<AssetId>), AppError> {
    let sequence = find_sequence(project, sequence_id)?;
    let Some(range) = limit_of(sequence, None).and_then(|limit| clipped(range, limit)) else {
        return Ok((Vec::new(), BTreeSet::new()));
    };
    let context = WordContext::new(sequence, media, range);
    let words = &context.transcript.words;
    let occurrences = context
        .fillers(media, lists, max_duration, range)
        .into_iter()
        .filter_map(|span| {
            let (first, last) = (words.get(span.first)?, words.get(span.last)?);
            let text = words[span.first..=span.last]
                .iter()
                .map(|word| word.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            Some(FillerOccurrence {
                text,
                start: first.start,
                end: last.end,
                item_id: first.item_id,
                asset_id: first.asset_id,
            })
        })
        .collect();
    Ok((occurrences, context.untranscribed))
}

/// `limit` inside the sequence, or the whole sequence (from 0 to the end of its last item);
/// `None` when nothing is left.
fn limit_of(sequence: &Sequence, limit: Option<TimeRange>) -> Option<TimeRange> {
    let end = sequence
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .map(|item| item.timeline_range.end())
        .max()?;
    let whole = TimeRange::from_start_end(Flicks::ZERO, end).ok()?;
    match limit {
        Some(limit) => limit.intersection(whole),
        None => Some(whole),
    }
}

/// `range` inside `limit` (`None`: all of `limit`); `None` when they do not meet.
fn clipped(range: Option<TimeRange>, limit: TimeRange) -> Option<TimeRange> {
    match range {
        Some(range) => range.intersection(limit),
        None => Some(limit),
    }
}
