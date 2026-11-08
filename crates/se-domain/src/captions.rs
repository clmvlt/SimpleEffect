//! The pure rules of captions (prompt 23), shared by the core's use cases and mirrored where the
//! renderer needs them:
//!
//! - `lines`: [`caption_lines`], the greedy line breaking by characters (also run by the engine
//!   on `docs/test-vectors/caption-lines.json`: change both sides together).
//! - `segmentation`: [`segment_words`], spoken words → caption drafts (line and duration limits,
//!   punctuation and pauses).
//! - `retime`: [`retimed_words`], the words of an edited caption text (unchanged words keep their
//!   timing, changed runs are redistributed).
//! - `words`: [`distribute_words`] (words spread over a span), [`merged_words`] and
//!   [`split_point`] (merging and splitting captions).

mod lines;
mod retime;
mod segmentation;
mod words;

pub use lines::{caption_lines, line_chars};
pub use retime::retimed_words;
pub use segmentation::{CaptionDraft, SegmentationParams, TimedWord, segment_words};
pub use words::{Spread, distribute_words, merged_words, split_point, tidy_words};
