//! Transcripts of media (prompt 21): the words spoken in an asset, timed in **source** time
//! ([`Transcript`]), and the same words brought into a sequence's time through its clips
//! ([`sequence_transcript`]).
//!
//! Full transcripts live in the project database (written by se-store, like analyses): they are
//! facts about a file's content, not edits. The asset keeps a small [`TranscriptSummary`]
//! (`Asset.analysis.transcript`) so the media bin and agents know what is transcribed.
//!
//! [`sequence_transcript`] is the one source ↔ sequence correspondence shared by the transcript
//! panel (its TypeScript mirror is tested on the same vectors, `docs/test-vectors/
//! transcript-mapping.json`), the MCP tools and the text-based editing and captions of later
//! prompts.

mod clip_window;
mod sequence_transcript;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use self::{
    clip_window::ClipWindow,
    sequence_transcript::{
        SequenceSegment, SequenceTranscript, SequenceWord, sequence_transcript, words_in,
    },
};
use crate::Flicks;

/// The words spoken in an asset, in source time.
///
/// Invariants (guaranteed by the transcriber adapter, checked by [`Transcript::problems`]): words
/// are sorted by start, never overlap (`end ≤` the next word's `start`), `start ≤ end`,
/// confidences lie in `[0, 1]` and segment indices point into `segments`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Transcript {
    /// The spoken language (ISO 639 code), forced or detected.
    pub language: String,
    /// The model that transcribed it (`large-v3-turbo`, or a custom model's file name).
    pub model: String,
    /// Every word, in order.
    pub words: Vec<TranscriptWord>,
    /// Sentences or phrases as the model cut them.
    pub segments: Vec<TranscriptSegment>,
}

/// One spoken word in source time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptWord {
    /// The word as written, with the punctuation attached to it (`va ?`, `l'homme,`).
    pub text: String,
    /// When it starts in the asset.
    pub start: Flicks,
    /// When it ends in the asset.
    pub end: Flicks,
    /// How sure the model is, from 0 to 1 (mean probability of its tokens).
    pub confidence: f64,
    /// Index of its segment in [`Transcript::segments`].
    pub segment: u32,
}

/// A sentence or phrase of a transcript.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSegment {
    /// When it starts in the asset.
    pub start: Flicks,
    /// When it ends in the asset.
    pub end: Flicks,
}

/// What an asset keeps of its transcript (in the document): enough to show a badge and to know
/// which transcript version the renderer holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSummary {
    /// The spoken language.
    pub language: String,
    /// The model that transcribed it.
    pub model: String,
    /// How many words were recognised.
    pub word_count: u32,
    /// When it was transcribed (Unix milliseconds): a new transcription replaces the old one and
    /// changes this value.
    pub transcribed_at: i64,
}

impl Transcript {
    /// Confidence under which a word is shown as unsure.
    pub const LOW_CONFIDENCE: f64 = 0.5;

    /// The summary an asset keeps.
    #[must_use]
    pub fn summary(&self, transcribed_at: i64) -> TranscriptSummary {
        TranscriptSummary {
            language: self.language.clone(),
            model: self.model.clone(),
            word_count: u32::try_from(self.words.len()).unwrap_or(u32::MAX),
            transcribed_at,
        }
    }

    /// The text of segment `index`: its words joined by spaces (empty for an unknown index).
    #[must_use]
    pub fn segment_text(&self, index: u32) -> String {
        join_words(self.words.iter().filter(|word| word.segment == index))
    }

    /// The broken invariants, described; empty for a well-formed transcript.
    #[must_use]
    pub fn problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        let segment_count = self.segments.len();
        for (index, word) in self.words.iter().enumerate() {
            if word.start > word.end {
                problems.push(format!("word {index} ends before it starts"));
            }
            if !(0.0..=1.0).contains(&word.confidence) {
                problems.push(format!("word {index} has a confidence outside [0, 1]"));
            }
            if usize::try_from(word.segment).map_or(true, |segment| segment >= segment_count) {
                problems.push(format!("word {index} points to a missing segment"));
            }
            if let Some(next) = self.words.get(index + 1)
                && word.end > next.start
            {
                problems.push(format!("word {index} overlaps the next one"));
            }
        }
        problems
    }
}

/// Words joined by single spaces (the punctuation is part of each word).
pub fn join_words<'a>(words: impl IntoIterator<Item = &'a TranscriptWord>) -> String {
    let mut text = String::new();
    for word in words {
        if !text.is_empty() {
            text.push(' ');
        }
        text.push_str(&word.text);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::millis;

    fn word(text: &str, start_ms: i64, end_ms: i64, segment: u32) -> TranscriptWord {
        TranscriptWord {
            text: text.to_owned(),
            start: millis(start_ms),
            end: millis(end_ms),
            confidence: 0.9,
            segment,
        }
    }

    fn transcript() -> Transcript {
        Transcript {
            language: "fr".to_owned(),
            model: "base".to_owned(),
            words: vec![
                word("Bonjour", 0, 400, 0),
                word("à", 400, 500, 0),
                word("tous.", 500, 900, 0),
                word("Ça", 1500, 1700, 1),
                word("va ?", 1700, 2000, 1),
            ],
            segments: vec![
                TranscriptSegment {
                    start: millis(0),
                    end: millis(900),
                },
                TranscriptSegment {
                    start: millis(1500),
                    end: millis(2000),
                },
            ],
        }
    }

    #[test]
    fn summarises_and_joins_segments() {
        let transcript = transcript();
        let summary = transcript.summary(42);
        assert_eq!(summary.word_count, 5);
        assert_eq!(summary.transcribed_at, 42);
        assert_eq!(transcript.segment_text(0), "Bonjour à tous.");
        assert_eq!(transcript.segment_text(1), "Ça va ?");
        assert_eq!(transcript.segment_text(7), "");
        assert_eq!(transcript.problems(), Vec::<String>::new());
    }

    #[test]
    fn reports_broken_invariants() {
        let mut transcript = transcript();
        transcript.words[1].start = millis(300);
        transcript.words[2].confidence = 1.5;
        transcript.words[3].segment = 9;
        transcript.words[4].end = millis(1600);
        let problems = transcript.problems();
        assert_eq!(problems.len(), 4, "{problems:?}");
    }
}
