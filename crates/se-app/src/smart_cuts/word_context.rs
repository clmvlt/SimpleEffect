//! [`WordContext`]: the words of a sequence in sequence time and what word and filler cuts need
//! around them (each word's clip and track, untranscribed media).

use std::collections::{BTreeMap, BTreeSet};

use se_domain::{
    AssetId, Flicks, ItemId, ItemKind, Sequence, SequenceTranscript, SequenceWord, SilenceSettings,
    TimeRange, TrackId, TrackKind, sequence_transcript,
};
use se_protocol::FillerList;

use super::{
    CutMedia, ResolvedCuts,
    fillers::{FillerPhrases, FillerSpan, find_fillers},
    plan::{CutParams, FrameSnap, plan_cuts},
    resolve::MIN_KEEP,
    silences::sequence_silences,
    words::{SNAP_REACH, snap_to_quiet, word_cut_ranges, words_in_ranges},
};

/// Quiet enough to snap word cuts to: the speech threshold, from one 20 ms window.
const QUIET: SilenceSettings = SilenceSettings {
    threshold_dbfs: SilenceSettings::SPEECH.threshold_dbfs,
    min_duration: Flicks::from_millis(20),
};

/// The words of a sequence and what word cuts need around them.
pub(super) struct WordContext {
    pub(super) transcript: SequenceTranscript,
    /// The sequence range of each sound clip, by item.
    clips: BTreeMap<ItemId, (TrackId, TimeRange)>,
    /// Sound media of the range without a transcript.
    pub(super) untranscribed: BTreeSet<AssetId>,
    limit: TimeRange,
}

impl WordContext {
    pub(super) fn new(sequence: &Sequence, media: &CutMedia, limit: TimeRange) -> Self {
        let transcript = sequence_transcript(sequence, |asset| {
            media.transcripts.get(&asset).map(AsRef::as_ref)
        });
        let mut clips = BTreeMap::new();
        let mut untranscribed = BTreeSet::new();
        for track in sequence
            .tracks
            .iter()
            .filter(|track| track.kind == TrackKind::Audio)
        {
            for item in &track.items {
                let ItemKind::MediaClip(clip) = &item.kind else {
                    continue;
                };
                clips.insert(item.id, (track.id, item.timeline_range));
                if item.timeline_range.intersects(limit)
                    && !media.transcripts.contains_key(&clip.asset_id)
                {
                    untranscribed.insert(clip.asset_id);
                }
            }
        }
        Self {
            transcript,
            clips,
            untranscribed,
            limit,
        }
    }

    /// The filler spans among the words of `range`.
    pub(super) fn fillers(
        &self,
        media: &CutMedia,
        lists: &[FillerList],
        max_duration: Flicks,
        range: TimeRange,
    ) -> Vec<FillerSpan> {
        let words = &self.transcript.words;
        let candidates: Vec<usize> = words_in_ranges(words, &[range]).into_iter().collect();
        let mut phrases: BTreeMap<AssetId, FillerPhrases> = BTreeMap::new();
        for word in &self.transcript.words {
            phrases.entry(word.asset_id).or_insert_with(|| {
                let language = media
                    .transcripts
                    .get(&word.asset_id)
                    .map_or("", |transcript| transcript.language.as_str());
                FillerPhrases::for_language(lists, language)
            });
        }
        let empty = FillerPhrases::default();
        find_fillers(
            words,
            &candidates,
            |asset| phrases.get(&asset).unwrap_or(&empty),
            max_duration,
        )
    }

    /// The cuts removing the words at `removed`.
    pub(super) fn cuts(
        &self,
        sequence: &Sequence,
        media: &CutMedia,
        removed: &BTreeSet<usize>,
    ) -> ResolvedCuts {
        let words: &[SequenceWord] = &self.transcript.words;
        let windows: BTreeMap<_, _> = self
            .clips
            .iter()
            .map(|(item, (_, range))| (*item, *range))
            .collect();
        let candidates = word_cut_ranges(words, removed, &windows);
        let seeds: BTreeSet<TrackId> = removed
            .iter()
            .filter_map(|index| words.get(*index))
            .filter_map(|word| self.clips.get(&word.item_id).map(|(track, _)| *track))
            .collect();
        let bounds: Vec<TimeRange> = self
            .clips
            .values()
            .filter(|(track, _)| seeds.contains(track))
            .map(|(_, range)| *range)
            .collect();
        let params = CutParams {
            padding: Flicks::ZERO,
            min_cut: sequence.format.frame_rate.frame_duration(),
            min_keep: MIN_KEEP,
            frame_rate: sequence.format.frame_rate,
            snap: FrameSnap::Nearest,
        };
        let quiet = sequence_silences(
            sequence,
            &seeds,
            |asset| media.levels.get(&asset).map(AsRef::as_ref),
            QUIET,
            self.limit,
        );
        let snapped = snap_to_quiet(&candidates, &quiet.silences, SNAP_REACH);
        let cuts = plan_cuts(&snapped, &params, &bounds, self.limit);
        let cuts = if bounds.is_empty() { Vec::new() } else { cuts };
        ResolvedCuts {
            cuts,
            seeds,
            word_count: u32::try_from(removed.len()).unwrap_or(u32::MAX),
            untranscribed: self.untranscribed.clone(),
            unanalysed: BTreeSet::new(),
        }
    }
}
