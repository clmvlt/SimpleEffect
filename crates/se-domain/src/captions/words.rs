//! Caption words spread over a span, cleaned up, merged and split.

use super::lines::line_chars;
use crate::{CaptionWord, Flicks};

/// How [`distribute_words`] shares a span between words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spread {
    /// Every word lasts as long (imported subtitles: no timing is known).
    Equal,
    /// Each word lasts in proportion to its characters plus one (edited text: longer words take
    /// longer to say; the extra one gives punctuation a share).
    ByLength,
}

/// `tokens` as contiguous words covering `[start, end)` (item-relative), shared as `spread` says.
/// Boundaries are computed exactly in flicks; an empty span gives zero-length words at `start`.
#[must_use]
pub fn distribute_words<S: AsRef<str>>(
    tokens: &[S],
    start: Flicks,
    end: Flicks,
    spread: Spread,
) -> Vec<CaptionWord> {
    let weights: Vec<i128> = tokens
        .iter()
        .map(|token| match spread {
            Spread::Equal => 1,
            Spread::ByLength => i128::try_from(line_chars(token.as_ref())).unwrap_or(0) + 1,
        })
        .collect();
    let total: i128 = weights.iter().sum::<i128>().max(1);
    let span = i128::from(end.get().saturating_sub(start.get()).max(0));
    let boundary = |weight_before: i128| {
        let offset = i64::try_from(span * weight_before / total).unwrap_or(0);
        Flicks::new(start.get().saturating_add(offset)).unwrap_or(start)
    };
    let mut before = 0;
    tokens
        .iter()
        .zip(&weights)
        .map(|(token, weight)| {
            let word_start = boundary(before);
            before += weight;
            CaptionWord::new(token.as_ref(), word_start, boundary(before))
        })
        .collect()
}

/// `words` made valid for a caption lasting `duration`: texts trimmed with line breaks turned into
/// spaces, empty words dropped, times clamped into `[0, duration]`, each start at or after the
/// previous end, each end between its start and the next word's start.
#[must_use]
pub fn tidy_words(words: Vec<CaptionWord>, duration: Flicks) -> Vec<CaptionWord> {
    let clamp = |time: Flicks| time.clamp(Flicks::ZERO, duration);
    let kept: Vec<CaptionWord> = words
        .into_iter()
        .filter_map(|word| {
            let text = word.text.replace(['\r', '\n'], " ").trim().to_owned();
            (!text.is_empty()).then(|| CaptionWord {
                text,
                start: clamp(word.start),
                end: clamp(word.end),
                ..word
            })
        })
        .collect();
    let mut tidy: Vec<CaptionWord> = Vec::with_capacity(kept.len());
    for (index, word) in kept.iter().enumerate() {
        let previous_end = tidy.last().map_or(Flicks::ZERO, |previous| previous.end);
        let start = word.start.max(previous_end);
        let next_start = kept.get(index + 1).map(|next| next.start);
        let end = match next_start {
            Some(next) if next >= start => word.end.min(next),
            _ => word.end,
        }
        .max(start);
        tidy.push(CaptionWord {
            start,
            end,
            ..word.clone()
        });
    }
    tidy
}

/// The words of a caption made of `first` followed by `second`, whose item starts `offset`
/// after the first's.
#[must_use]
pub fn merged_words(
    first: &[CaptionWord],
    second: &[CaptionWord],
    offset: Flicks,
) -> Vec<CaptionWord> {
    let shift = |time: Flicks| time.checked_add(offset).unwrap_or(time);
    first
        .iter()
        .cloned()
        .chain(second.iter().map(|word| CaptionWord {
            start: shift(word.start),
            end: shift(word.end),
            ..word.clone()
        }))
        .collect()
}

/// Where to cut a caption so that word `index` starts the right part: halfway through the pause
/// before it (item-relative). `None` when `index` does not leave a word on each side.
#[must_use]
pub fn split_point(words: &[CaptionWord], index: usize) -> Option<Flicks> {
    let previous = words.get(index.checked_sub(1)?)?;
    let next = words.get(index)?;
    let gap = next.start.get().saturating_sub(previous.end.get()).max(0);
    Flicks::new(previous.end.get().saturating_add(gap / 2)).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::millis;

    fn word(text: &str, start: i64, end: i64) -> CaptionWord {
        CaptionWord::new(text, millis(start), millis(end))
    }

    #[test]
    fn spreads_words_equally_or_by_length() {
        assert_eq!(
            distribute_words(&["a", "bb", "c"], millis(0), millis(900), Spread::Equal),
            [word("a", 0, 300), word("bb", 300, 600), word("c", 600, 900)]
        );
        // Weights 2, 3, 2 over 700 ms.
        assert_eq!(
            distribute_words(
                &["a", "bb", "c"],
                millis(100),
                millis(800),
                Spread::ByLength
            ),
            [
                word("a", 100, 300),
                word("bb", 300, 600),
                word("c", 600, 800)
            ]
        );
    }

    #[test]
    fn tidying_orders_clamps_and_drops_empty_words() {
        let words = vec![
            word(" Salut ", -50, 400),
            word("", 300, 350),
            word("toi\n", 350, 900),
            word("là", 800, 2500),
        ];
        assert_eq!(
            tidy_words(words, millis(2000)),
            [
                word("Salut", 0, 350),
                word("toi", 350, 800),
                word("là", 800, 2000)
            ]
        );
    }

    #[test]
    fn merges_and_splits_between_words() {
        let merged = merged_words(
            &[word("un", 0, 400)],
            &[word("deux", 100, 500)],
            millis(1000),
        );
        assert_eq!(merged, [word("un", 0, 400), word("deux", 1100, 1500)]);
        assert_eq!(split_point(&merged, 1), Some(millis(750)));
        assert_eq!(split_point(&merged, 0), None);
        assert_eq!(split_point(&merged, 2), None);
    }
}
