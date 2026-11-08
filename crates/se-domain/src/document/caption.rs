//! [`CaptionItem`]: a subtitle on the timeline, made of timed words (prompt 23).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{CaptionStyleId, Flicks};

/// A subtitle: timed words drawn with a project caption style.
///
/// Word times are relative to the item's start (like keyframes) and lie inside the item; words
/// are listed in speaking order and never overlap (validation: `INVALID_CAPTION_WORDS`). A caption
/// may hold no word (a trim left none): it then draws nothing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CaptionItem {
    /// The words, in speaking order.
    pub words: Vec<CaptionWord>,
    /// The project caption style drawing it ([`crate::Project::caption_styles`]).
    pub style_id: CaptionStyleId,
}

/// One word of a caption: its text and when it is spoken.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CaptionWord {
    /// The word as shown (punctuation attached), never empty, without line breaks.
    pub text: String,
    /// When it starts being spoken, relative to the item's start.
    pub start: Flicks,
    /// When it stops (≥ `start`, ≤ the next word's start), relative to the item's start.
    pub end: Flicks,
    /// Shown in the style's emphasis colour (a keyword put forward).
    #[serde(default)]
    pub emphasis: bool,
}

impl CaptionWord {
    /// A word spoken over `[start, end)`, not emphasised.
    #[must_use]
    pub fn new(text: impl Into<String>, start: Flicks, end: Flicks) -> Self {
        Self {
            text: text.into(),
            start,
            end,
            emphasis: false,
        }
    }

    /// The instant halfway through the word (which part of a cut item keeps it).
    #[must_use]
    pub fn middle(&self) -> Flicks {
        let half = (self.end.get().saturating_sub(self.start.get()) / 2).max(0);
        Flicks::new(self.start.get().saturating_add(half)).unwrap_or(self.start)
    }
}

impl CaptionItem {
    /// The caption's text: its words joined by spaces.
    #[must_use]
    pub fn text(&self) -> String {
        self.words
            .iter()
            .map(|word| word.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::millis;

    #[test]
    fn text_joins_the_words_and_middles_round_down() {
        let caption = CaptionItem {
            words: vec![
                CaptionWord::new("Bonjour", millis(0), millis(400)),
                CaptionWord::new("à tous", millis(400), millis(901)),
            ],
            style_id: CaptionStyleId::from_test_index(1),
        };
        assert_eq!(caption.text(), "Bonjour à tous");
        assert_eq!(caption.words[0].middle(), millis(200));
        assert_eq!(
            caption.words[1].middle().get(),
            millis(400).get() + millis(501).get() / 2
        );
    }
}
