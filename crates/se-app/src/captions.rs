//! Captions (prompt 23): subtitles generated from the transcript, edited as data (text, timing,
//! style) and exchanged as SRT files, for the user (one undoable transaction per action) and
//! for AI agents through the same planning.
//!
//! - `track`: the caption track of a sequence (the highest unlocked video track holding captions
//!   only; a new « Sous-titres » track on top when missing).
//! - `styles`: the style new captions use, copy on write of shared styles, unused styles removed.
//! - `generate`: drafts (from `se_domain::captions::segment_words` or an SRT file) replacing
//!   the captions crossing a range, in one plan.
//! - `edits`: text (unchanged words keep their timing), emphasis, merge, split, shift.
//! - `subtitles`: captions ↔ SRT cues.
//! - `service`: [`CaptionService`], the user's use cases.
//!
//! Captions are ordinary items: smart cuts ripple caption tracks with the speech
//! (`smart_cuts::apply::cut_tracks`), and every edit is undone in one step.

pub(crate) mod edits;
pub(crate) mod generate;
mod service;
pub(crate) mod styles;
pub mod subtitles;
pub mod track;

pub use service::{CaptionService, CaptionServiceDeps};

/// Labels of the caption transactions (translated by the renderer's `history-label.ts`).
pub mod labels {
    /// Captions made from the transcript.
    pub const GENERATE_CAPTIONS: &str = "Generate captions";
    /// Captions made from an SRT file.
    pub const IMPORT_SUBTITLES: &str = "Import subtitles";
    /// A caption's text changed.
    pub const EDIT_CAPTION_TEXT: &str = "Edit caption text";
    /// Captions merged.
    pub const MERGE_CAPTIONS: &str = "Merge captions";
    /// A caption split in two.
    pub const SPLIT_CAPTION: &str = "Split caption";
    /// Captions moved in time.
    pub const SHIFT_CAPTIONS: &str = "Shift captions";
    /// Words put forward or back.
    pub const EMPHASIZE_WORDS: &str = "Emphasize caption words";
    /// The look of captions changed.
    pub const CHANGE_CAPTION_STYLE: &str = "Change caption style";
    /// Captions given a predefined style.
    pub const APPLY_CAPTION_PRESET: &str = "Apply caption style";
}
