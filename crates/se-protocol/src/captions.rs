//! Caption DTOs (prompt 23): generation from the transcript, edits of captions and of their
//! shared styles, SRT import and export. Every edit is one undoable user transaction of
//! ordinary items (`ItemKind::Caption`) and caption styles.

use se_domain::{
    AssetId, CaptionStyleId, CaptionStylePatch, Flicks, ItemId, ProjectId, SequenceId, TimeRange,
    TrackId,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Which style new captions get.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CaptionStyleChoice {
    /// A new style made from a predefined one (`tiktokBold`, `minimal`, `karaoke`, `box`).
    Preset {
        /// The preset's id.
        preset: String,
    },
    /// A style of the project.
    Existing {
        /// Its id.
        style_id: CaptionStyleId,
    },
    /// A new style made from the caption look of the project's brand kit (prompt 25).
    BrandKit,
}

/// Overrides of the segmentation limits (`null` fields keep the style's line limits and the
/// defaults: 0.7 s to 6 s, a new caption after a 0.6 s pause).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[ts(optional_fields)]
pub struct CaptionSegmentation {
    /// Longest line in characters (4–80).
    pub max_chars_per_line: Option<u32>,
    /// Lines per caption (1–4).
    pub max_lines: Option<u32>,
    /// Shortest caption.
    pub min_duration: Option<Flicks>,
    /// Longest caption.
    pub max_duration: Option<Flicks>,
    /// A pause at least this long starts a new caption.
    pub pause: Option<Flicks>,
}

/// Captions to make from the transcript of a sequence range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(optional_fields)]
pub struct GenerateCaptionsRequest {
    /// The open project.
    pub project_id: ProjectId,
    /// The sequence.
    pub sequence_id: SequenceId,
    /// Where (`null`: the whole sequence). Captions of the caption track crossing it are replaced
    /// (the range grows to cover them whole).
    #[serde(default)]
    pub range: Option<TimeRange>,
    /// The style (`null`: the style of the captions replaced, else the `minimal` preset).
    #[serde(default)]
    pub style: Option<CaptionStyleChoice>,
    /// Segmentation overrides.
    #[serde(default)]
    pub segmentation: Option<CaptionSegmentation>,
    /// Name of the caption track when one is created (`null`: « Sous-titres »).
    #[serde(default)]
    pub track_name: Option<String>,
}

/// What a caption generation or import did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CaptionsReport {
    /// The captions created, in time order (empty when no word was found: nothing changed).
    pub item_ids: Vec<ItemId>,
    /// The caption track (created when the sequence had none).
    pub track_id: Option<TrackId>,
    /// The style of the new captions.
    pub style_id: Option<CaptionStyleId>,
    /// Captions replaced.
    pub replaced: u32,
    /// Media of the range not transcribed yet (their speech got no caption).
    pub untranscribed_assets: Vec<AssetId>,
}

/// An SRT file to turn into captions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(optional_fields)]
pub struct ImportSubtitlesRequest {
    /// The open project.
    pub project_id: ProjectId,
    /// The sequence (cue times are sequence times).
    pub sequence_id: SequenceId,
    /// Absolute path of the `.srt` file.
    pub path: String,
    /// The style (`null`: as for generation).
    #[serde(default)]
    pub style: Option<CaptionStyleChoice>,
    /// Name of the caption track when one is created.
    #[serde(default)]
    pub track_name: Option<String>,
}

/// New words of one caption.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CaptionTextEdit {
    /// The open project.
    pub project_id: ProjectId,
    /// The caption.
    pub item_id: ItemId,
    /// Its new text: unchanged words keep their timing, changed runs are redistributed.
    pub text: String,
}

/// Emphasis of some words of a caption.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CaptionEmphasisEdit {
    /// The open project.
    pub project_id: ProjectId,
    /// The caption.
    pub item_id: ItemId,
    /// Indices of its words (0-based).
    pub word_indices: Vec<u32>,
    /// Put forward (`true`) or back to normal.
    pub emphasis: bool,
}

/// A change of the style of captions, made only for them (copy on write).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CaptionStyleEdit {
    /// The open project.
    pub project_id: ProjectId,
    /// The captions (all those of a track to change the track's look). A style also used by
    /// captions not listed is copied for the listed ones.
    pub item_ids: Vec<ItemId>,
    /// What changes.
    pub patch: CaptionStylePatch,
}
