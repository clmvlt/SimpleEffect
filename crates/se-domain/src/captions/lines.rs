//! Greedy line breaking of caption words by character count.

use std::ops::Range;

/// Characters of `text` as the renderer counts them (Unicode scalar values, like
/// `Array.from(text).length`).
#[must_use]
pub fn line_chars(text: &str) -> usize {
    text.chars().count()
}

/// The lines of `words` (indices into it) when each line holds at most `max_chars` characters,
/// spaces between words included: words are added to the current line while they fit; a word
/// longer than `max_chars` gets a line of its own. No line is empty; no words, no lines.
#[must_use]
pub fn caption_lines<S: AsRef<str>>(words: &[S], max_chars: usize) -> Vec<Range<usize>> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut length = 0;
    for (index, word) in words.iter().enumerate() {
        let chars = line_chars(word.as_ref());
        if index > start && length + 1 + chars > max_chars {
            lines.push(start..index);
            start = index;
            length = chars;
        } else if index == start {
            length = chars;
        } else {
            length += 1 + chars;
        }
    }
    if start < words.len() {
        lines.push(start..words.len());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_fill_lines_greedily() {
        let words = ["Bonjour", "à", "tous", "et", "bienvenue"];
        assert_eq!(caption_lines(&words, 12), [0..2, 2..4, 4..5]);
        assert_eq!(caption_lines(&words, 14), [0..3, 3..5]);
        assert_eq!(caption_lines(&words, 80), vec![0..5]);
        assert_eq!(
            caption_lines(&["anticonstitutionnellement", "oui"], 10),
            [0..1, 1..2]
        );
        assert_eq!(caption_lines::<&str>(&[], 10), []);
    }

    #[test]
    fn characters_are_unicode_scalars() {
        assert_eq!(line_chars("été"), 3);
        assert_eq!(caption_lines(&["été", "là"], 6), vec![0..2]);
    }
}
