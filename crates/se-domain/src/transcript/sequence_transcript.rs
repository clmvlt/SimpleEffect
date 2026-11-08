//! [`sequence_transcript`]: the words of a sequence, brought from their assets' transcripts into
//! sequence time through the clips that play them.
//!
//! Rules (mirrored by `apps/desktop/src/shared/transcript/sequence-words.ts`, both tested on
//! `docs/test-vectors/transcript-mapping.json`):
//! - only clips of **sound** tracks count: speech is heard through them (a picture clip whose sound
//!   was removed says nothing); muted tracks still count (muting is temporary);
//! - a word belongs to a clip when its **middle** lies in the part of the source the clip plays,
//!   so a cut inside a word gives it to one side only, and a removed passage loses its words;
//! - its times go through the clip's speed and are clamped to the clip;
//! - the same word of the same asset at the same sequence time (one sound on two tracks) appears
//!   once; words are sorted by start, then track order;
//! - consecutive words of one clip and one source segment form a [`SequenceSegment`].

use std::collections::HashSet;

use super::{ClipWindow, Transcript, TranscriptWord};
use crate::{AssetId, Flicks, ItemId, ItemKind, Sequence, TimeRange, TrackKind};

/// A word of a sequence, in sequence time.
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceWord {
    /// The word as written.
    pub text: String,
    /// When it starts in the sequence.
    pub start: Flicks,
    /// When it ends in the sequence.
    pub end: Flicks,
    /// How sure the model is (0–1).
    pub confidence: f64,
    /// The asset it comes from.
    pub asset_id: AssetId,
    /// The clip that plays it.
    pub item_id: ItemId,
    /// Its index in the asset's transcript.
    pub word_index: u32,
    /// Its index in [`SequenceTranscript::segments`].
    pub segment: u32,
}

/// Consecutive words of one clip and one source segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceSegment {
    /// Start of its first word.
    pub start: Flicks,
    /// End of its last word.
    pub end: Flicks,
    /// The clip.
    pub item_id: ItemId,
    /// The asset.
    pub asset_id: AssetId,
    /// Index of its first word in [`SequenceTranscript::words`].
    pub first_word: u32,
    /// How many words it has.
    pub word_count: u32,
    /// Index of the segment of the asset's transcript its words come from.
    pub source_segment: u32,
}

/// The words of a sequence, in sequence time.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SequenceTranscript {
    /// Sorted by start, then track order.
    pub words: Vec<SequenceWord>,
    /// In word order.
    pub segments: Vec<SequenceSegment>,
}

impl SequenceTranscript {
    /// The words whose middle lies in `range`.
    pub fn words_in(&self, range: TimeRange) -> impl Iterator<Item = &SequenceWord> {
        words_in(&self.words, range)
    }
}

/// The words of `words` whose middle lies in `range`.
pub fn words_in(words: &[SequenceWord], range: TimeRange) -> impl Iterator<Item = &SequenceWord> {
    words
        .iter()
        .filter(move |word| range.contains(middle(word.start, word.end)))
}

/// The words of `sequence`'s sound clips whose assets have a transcript (`transcript_of`), in
/// sequence time.
pub fn sequence_transcript<'a>(
    sequence: &Sequence,
    transcript_of: impl Fn(AssetId) -> Option<&'a Transcript>,
) -> SequenceTranscript {
    let mut mapped = Vec::new();
    for (track_index, track) in sequence.tracks.iter().enumerate() {
        if track.kind != TrackKind::Audio {
            continue;
        }
        for item in &track.items {
            let ItemKind::MediaClip(clip) = &item.kind else {
                continue;
            };
            let (Some(transcript), Some(window)) =
                (transcript_of(clip.asset_id), ClipWindow::of(item, clip))
            else {
                continue;
            };
            let clip_words = ClipWords {
                window,
                item_id: item.id,
                asset_id: clip.asset_id,
                track_index,
            };
            clip_words.collect(transcript, &mut mapped);
        }
    }
    let mut seen = HashSet::new();
    mapped.retain(|word: &Mapped| {
        seen.insert((word.word.asset_id, word.word.word_index, word.word.start))
    });
    mapped.sort_by_key(|word| (word.word.start, word.track_index, word.word.word_index));
    group(mapped)
}

/// A mapped word before grouping: its source segment and track decide segments and ties.
struct Mapped {
    word: SequenceWord,
    track_index: usize,
    source_segment: u32,
}

/// One clip playing a transcribed asset.
struct ClipWords {
    window: ClipWindow,
    item_id: ItemId,
    asset_id: AssetId,
    track_index: usize,
}

impl ClipWords {
    /// Pushes the words whose middle the clip plays.
    fn collect(&self, transcript: &Transcript, out: &mut Vec<Mapped>) {
        let source = self.window.source;
        let first = transcript
            .words
            .partition_point(|word| middle(word.start, word.end) < source.start());
        for (index, word) in transcript.words.iter().enumerate().skip(first) {
            if middle(word.start, word.end) >= source.end() {
                break;
            }
            if let Some(mapped) = self.map(index, word) {
                out.push(mapped);
            }
        }
    }

    fn map(&self, index: usize, word: &TranscriptWord) -> Option<Mapped> {
        let timeline = self.window.timeline;
        let start = self.window.to_timeline(word.start)?.max(timeline.start());
        let end = self.window.to_timeline(word.end)?.min(timeline.end());
        Some(Mapped {
            word: SequenceWord {
                text: word.text.clone(),
                start,
                end: end.max(start),
                confidence: word.confidence,
                asset_id: self.asset_id,
                item_id: self.item_id,
                word_index: u32::try_from(index).ok()?,
                segment: 0,
            },
            track_index: self.track_index,
            source_segment: word.segment,
        })
    }
}

/// Numbers the words' segments: a new one whenever the clip or the source segment changes.
fn group(mapped: Vec<Mapped>) -> SequenceTranscript {
    let mut transcript = SequenceTranscript::default();
    let mut previous: Option<(ItemId, u32)> = None;
    for (
        index,
        Mapped {
            mut word,
            source_segment,
            ..
        },
    ) in mapped.into_iter().enumerate()
    {
        let key = (word.item_id, source_segment);
        if previous != Some(key) {
            transcript.segments.push(SequenceSegment {
                start: word.start,
                end: word.end,
                item_id: word.item_id,
                asset_id: word.asset_id,
                first_word: u32::try_from(index).unwrap_or(u32::MAX),
                word_count: 0,
                source_segment,
            });
            previous = Some(key);
        }
        if let Some(segment) = transcript.segments.last_mut() {
            segment.end = segment.end.max(word.end);
            segment.word_count += 1;
        }
        word.segment = u32::try_from(transcript.segments.len() - 1).unwrap_or(u32::MAX);
        transcript.words.push(word);
    }
    transcript
}

/// The middle of `[start, end]`, rounded down to the flick.
fn middle(start: Flicks, end: Flicks) -> Flicks {
    let half = (end.get() - start.get()) / 2;
    Flicks::new(start.get() + half).unwrap_or(start)
}
