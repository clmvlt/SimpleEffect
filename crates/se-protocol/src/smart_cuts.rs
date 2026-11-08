//! Smart-cut DTOs (prompt 22, ADR-0032): text-based editing, filler words and silence removal.
//! Every cut is one ripple transaction of ordinary items, adjustable by hand afterwards.

use se_domain::{AssetId, Flicks, ItemId, TimeRange, TrackId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// How silences are found and cut.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SilenceCutParams {
    /// Level (dBFS, RMS of 20 ms windows) under which sound counts as silence (−80 to −10).
    pub threshold_dbfs: f64,
    /// Shortest silence cut (0.1 s to 10 s).
    pub min_silence: Flicks,
    /// Silence kept before and after each cut so speech breathes (0 to 1 s).
    pub padding: Flicks,
}

impl Default for SilenceCutParams {
    fn default() -> Self {
        Self {
            threshold_dbfs: -40.0,
            min_silence: Flicks::from_millis(500),
            padding: Flicks::from_millis(120),
        }
    }
}

/// The filler words of one language.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FillerList {
    /// ISO 639-1 code of the language (`fr`, `en`).
    pub language: String,
    /// Words or short phrases (`euh`, `you know`), matched case-, accent- and
    /// punctuation-insensitively; repeated letters match once (`euuuh` is `euh`).
    pub words: Vec<String>,
}

/// The app's smart-cut settings (stored in `library.db`, edited in the transcript panel).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SmartCutSettings {
    /// Filler words per language; a transcript in a language without a list uses every list.
    pub fillers: Vec<FillerList>,
    /// Longest filler occurrence detected (longer ones are probably mistimed words).
    pub filler_max_duration: Flicks,
    /// Defaults of the silence removal dialog and of agents' `remove_silences`.
    pub silence: SilenceCutParams,
}

/// What a word cut removes, for its undo label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum WordCutPurpose {
    /// Words chosen in the transcript (« Remove words »).
    Words,
    /// Filler words (« Remove fillers »).
    Fillers,
}

/// A filler word (or phrase) found in a sequence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FillerOccurrence {
    /// Its words as written.
    pub text: String,
    /// Where it starts in the sequence.
    pub start: Flicks,
    /// Where it ends in the sequence.
    pub end: Flicks,
    /// The clip playing it.
    pub item_id: ItemId,
    /// The asset it was said in.
    pub asset_id: AssetId,
}

/// What a cut did (or, for a preview, would do).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CutSummary {
    /// The ranges removed, in sequence time **before** the cut, sorted.
    pub cuts: Vec<TimeRange>,
    /// Their total duration (the time gained).
    pub removed_duration: Flicks,
    /// Words removed (word and filler cuts; 0 for silences).
    pub word_count: u32,
    /// The tracks cut (the speech's tracks and their linked partners, or every unlocked track).
    pub track_ids: Vec<TrackId>,
    /// Media of the range whose words are unknown (not transcribed yet): nothing was cut there.
    pub untranscribed_assets: Vec<AssetId>,
    /// Media of the range whose sound is not analysed yet: no silence was found there.
    pub unanalysed_assets: Vec<AssetId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_params_default_to_speech_pauses() {
        assert_eq!(
            serde_json::to_value(SilenceCutParams::default()).unwrap(),
            serde_json::json!({
                "thresholdDbfs": -40.0,
                "minSilence": 352_800_000,
                "padding": 84_672_000
            })
        );
    }

    #[test]
    fn summaries_are_camel_case() {
        let summary = CutSummary {
            cuts: Vec::new(),
            removed_duration: Flicks::ZERO,
            word_count: 2,
            track_ids: Vec::new(),
            untranscribed_assets: Vec::new(),
            unanalysed_assets: Vec::new(),
        };
        let json = serde_json::to_value(summary).unwrap();
        assert_eq!(json["wordCount"], 2);
        assert!(json["unanalysedAssets"].is_array());
    }
}
