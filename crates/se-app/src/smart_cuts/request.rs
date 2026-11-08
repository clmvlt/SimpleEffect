//! What a smart cut asks for ([`CutRequest`]), the media facts it needs ([`CutMedia`]) and what
//! its resolution against a document gives ([`ResolvedCuts`]).

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use se_domain::{AssetId, Flicks, LevelEnvelope, SequenceId, TimeRange, TrackId, Transcript};
use se_protocol::{CutSummary, FillerList, SilenceCutParams};

/// Facts about the media a cut reads: transcripts and level envelopes of the assets the sequence's
/// sound clips play. They describe file contents, not the document, so they stay valid while a
/// cut is planned again on a newer document.
#[derive(Debug, Clone, Default)]
pub struct CutMedia {
    /// Transcripts of the assets' current content.
    pub transcripts: BTreeMap<AssetId, Arc<Transcript>>,
    /// Level envelopes of the assets' current content (audio analyses with levels).
    pub levels: BTreeMap<AssetId, Arc<LevelEnvelope>>,
}

/// What to cut.
#[derive(Debug, Clone, PartialEq)]
pub enum CutTarget {
    /// The words whose middle lies in these ranges (sequence time).
    Words {
        /// Ranges holding the words.
        ranges: Vec<TimeRange>,
    },
    /// The filler words of a range.
    Fillers {
        /// Where to look (`None`: the whole sequence).
        range: Option<TimeRange>,
        /// The lists to use (settings, or an agent's own words).
        lists: Vec<FillerList>,
        /// Longest occurrence.
        max_duration: Flicks,
    },
    /// The silences of a range.
    Silences {
        /// Where to look (`None`: the whole sequence).
        range: Option<TimeRange>,
        /// Threshold, minimum silence, padding.
        params: SilenceCutParams,
        /// Sound tracks whose silences count (`None`: every unmuted sound track holding media).
        tracks: Option<Vec<TrackId>>,
    },
}

/// A smart cut of one sequence.
#[derive(Debug, Clone, PartialEq)]
pub struct CutRequest {
    /// The sequence cut.
    pub sequence_id: SequenceId,
    /// What to cut.
    pub target: CutTarget,
    /// Cut every unlocked track instead of the speech's tracks and their linked partners.
    pub all_tracks: bool,
    /// Cuts never leave this range (an AI task's scope); `None`: the whole sequence.
    pub limit: Option<TimeRange>,
}

/// A cut resolved against a document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedCuts {
    /// The ranges to remove, sorted and apart.
    pub cuts: Vec<TimeRange>,
    /// The tracks whose content decided the cuts (the speech's tracks).
    pub seeds: BTreeSet<TrackId>,
    /// Words removed (word and filler cuts).
    pub word_count: u32,
    /// Media of the range without a transcript.
    pub untranscribed: BTreeSet<AssetId>,
    /// Media of the range without levels.
    pub unanalysed: BTreeSet<AssetId>,
}

impl ResolvedCuts {
    /// The summary of these cuts made on `tracks`.
    #[must_use]
    pub fn summary(&self, tracks: &BTreeSet<TrackId>) -> CutSummary {
        let removed_duration = self
            .cuts
            .iter()
            .try_fold(Flicks::ZERO, |total, cut| total.checked_add(cut.duration()))
            .unwrap_or(Flicks::MAX);
        CutSummary {
            cuts: self.cuts.clone(),
            removed_duration,
            word_count: self.word_count,
            track_ids: tracks.iter().copied().collect(),
            untranscribed_assets: self.untranscribed.iter().copied().collect(),
            unanalysed_assets: self.unanalysed.iter().copied().collect(),
        }
    }
}
