//! The words of a caption whose text was edited.
//!
//! Rule (prompt 23, documented in `docs/glossary.md` « redistribution des timings »): the new text
//! is split on whitespace and compared word by word with the old words (longest common
//! subsequence of identical texts). Words found unchanged keep their timing and emphasis. Between
//! two of them, a run of changed words with as many new words as old ones keeps the old timings
//! one for one (a corrected typo never moves the highlight); otherwise the new words share the
//! run's time — from the first replaced word's start to the last one's end, or for pure insertions
//! the pause between the neighbours — in proportion to their characters. A run with no time to
//! share takes in its neighbouring words until it has some (the whole caption at worst).

use std::ops::Range;

use super::words::{Spread, distribute_words, tidy_words};
use crate::{CaptionWord, Flicks};

/// Above this many word pairs the comparison is skipped and the whole text is redistributed.
const MAX_COMPARED_PAIRS: usize = 250_000;

/// A run of old words replaced by a run of new words (equal texts for single matched words).
#[derive(Debug, Clone)]
struct Group {
    old: Range<usize>,
    new: Range<usize>,
}

/// The words of a caption lasting `duration` whose words were `words` once its text becomes
/// `text` (see the module's rule). An empty text gives no words.
#[must_use]
pub fn retimed_words(words: &[CaptionWord], text: &str, duration: Flicks) -> Vec<CaptionWord> {
    let tokens: Vec<&str> = text.split_whitespace().collect();
    if tokens.is_empty() {
        return Vec::new();
    }
    let groups = if words.is_empty() || words.len() * tokens.len() > MAX_COMPARED_PAIRS {
        vec![Group {
            old: 0..words.len(),
            new: 0..tokens.len(),
        }]
    } else {
        widened(groups_of(words, &tokens), words, duration)
    };
    let mut result = Vec::with_capacity(tokens.len());
    for group in &groups {
        let new = &tokens[group.new.clone()];
        if group.old.len() == group.new.len() {
            result.extend(
                words[group.old.clone()]
                    .iter()
                    .zip(new)
                    .map(|(word, text)| CaptionWord {
                        text: (*text).to_owned(),
                        ..word.clone()
                    }),
            );
        } else if !new.is_empty() {
            let (start, end) = match span(group, words, duration) {
                (start, end) if end <= start && groups.len() == 1 => (Flicks::ZERO, duration),
                extent => extent,
            };
            result.extend(distribute_words(new, start, end, Spread::ByLength));
        }
    }
    tidy_words(result, duration)
}

/// The time a group's new words may share: its old words' extent, or the pause around an
/// insertion.
fn span(group: &Group, words: &[CaptionWord], duration: Flicks) -> (Flicks, Flicks) {
    if group.old.is_empty() {
        let before = group
            .old
            .start
            .checked_sub(1)
            .and_then(|index| words.get(index));
        let after = words.get(group.old.end);
        (
            before.map_or(Flicks::ZERO, |word| word.end),
            after.map_or(duration, |word| word.start),
        )
    } else {
        (words[group.old.start].start, words[group.old.end - 1].end)
    }
}

/// Merges groups that need time but have none with their neighbours until they have some.
fn widened(mut groups: Vec<Group>, words: &[CaptionWord], duration: Flicks) -> Vec<Group> {
    let mut index = 0;
    while index < groups.len() {
        let group = &groups[index];
        let needs_time = !group.new.is_empty() && group.old.len() != group.new.len();
        let (start, end) = span(group, words, duration);
        if !needs_time || end > start || groups.len() == 1 {
            index += 1;
            continue;
        }
        let (left, right) = if index > 0 {
            (index - 1, index)
        } else {
            (index, index + 1)
        };
        let merged = Group {
            old: groups[left].old.start..groups[right].old.end,
            new: groups[left].new.start..groups[right].new.end,
        };
        groups.splice(left..=right, [merged]);
        index = left;
    }
    groups
}

/// Matched words and the runs between them, in order.
fn groups_of(words: &[CaptionWord], tokens: &[&str]) -> Vec<Group> {
    let mut groups = Vec::new();
    let (mut old, mut new) = (0, 0);
    let push_run = |groups: &mut Vec<Group>, old_end: usize, new_end: usize, old, new| {
        if old < old_end || new < new_end {
            groups.push(Group {
                old: old..old_end,
                new: new..new_end,
            });
        }
    };
    for (matched_old, matched_new) in common_words(words, tokens) {
        push_run(&mut groups, matched_old, matched_new, old, new);
        groups.push(Group {
            old: matched_old..matched_old + 1,
            new: matched_new..matched_new + 1,
        });
        old = matched_old + 1;
        new = matched_new + 1;
    }
    push_run(&mut groups, words.len(), tokens.len(), old, new);
    groups
}

/// Pairs `(old index, new index)` of a longest common subsequence of identical texts.
fn common_words(words: &[CaptionWord], tokens: &[&str]) -> Vec<(usize, usize)> {
    let (rows, columns) = (words.len(), tokens.len());
    // lengths[i][j]: longest common subsequence of words[i..] and tokens[j..].
    let mut lengths = vec![vec![0_usize; columns + 1]; rows + 1];
    for i in (0..rows).rev() {
        for j in (0..columns).rev() {
            lengths[i][j] = if words[i].text == tokens[j] {
                lengths[i + 1][j + 1] + 1
            } else {
                lengths[i + 1][j].max(lengths[i][j + 1])
            };
        }
    }
    let mut pairs = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < rows && j < columns {
        if words[i].text == tokens[j] {
            pairs.push((i, j));
            i += 1;
            j += 1;
        } else if lengths[i + 1][j] >= lengths[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::millis;

    fn word(text: &str, start: i64, end: i64) -> CaptionWord {
        CaptionWord::new(text, millis(start), millis(end))
    }

    fn words() -> Vec<CaptionWord> {
        vec![
            word("Bonjour", 0, 400),
            word("a", 500, 600),
            word("tous", 700, 1000),
        ]
    }

    #[test]
    fn a_corrected_typo_keeps_every_timing() {
        let mut emphasised = words();
        emphasised[1].emphasis = true;
        let retimed = retimed_words(&emphasised, "Bonjour à tous", millis(1200));
        assert_eq!(retimed[1].text, "à");
        assert!(retimed[1].emphasis);
        assert_eq!(
            retimed
                .iter()
                .map(|word| (word.start, word.end))
                .collect::<Vec<_>>(),
            emphasised
                .iter()
                .map(|word| (word.start, word.end))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn a_replaced_run_shares_its_time_by_length() {
        // « a » (500–600 ms) becomes « à vous » : weights 2 and 5 over 100 ms.
        let retimed = retimed_words(&words(), "Bonjour à vous tous", millis(1200));
        assert_eq!(
            retimed,
            [
                word("Bonjour", 0, 400),
                CaptionWord::new(
                    "à",
                    millis(500),
                    Flicks::new(millis(500).get() + millis(100).get() * 2 / 7).unwrap()
                ),
                CaptionWord::new(
                    "vous",
                    Flicks::new(millis(500).get() + millis(100).get() * 2 / 7).unwrap(),
                    millis(600)
                ),
                word("tous", 700, 1000),
            ]
        );
    }

    #[test]
    fn insertions_use_the_pause_or_take_in_a_neighbour() {
        let retimed = retimed_words(&words(), "Bonjour à tous ici", millis(1200));
        assert_eq!(retimed[3], word("ici", 1000, 1200));
        let tight = vec![word("un", 0, 500), word("deux", 500, 1000)];
        let retimed = retimed_words(&tight, "un et deux", millis(1000));
        // No pause between « un » and « et »: « un et » shares « un »'s time (weights 3, 3).
        assert_eq!(
            retimed,
            [
                word("un", 0, 250),
                word("et", 250, 500),
                word("deux", 500, 1000)
            ]
        );
    }

    #[test]
    fn removed_words_leave_silence_and_empty_text_removes_all() {
        let retimed = retimed_words(&words(), "Bonjour tous", millis(1200));
        assert_eq!(retimed, [word("Bonjour", 0, 400), word("tous", 700, 1000)]);
        assert_eq!(retimed_words(&words(), "  ", millis(1200)), []);
    }

    #[test]
    fn a_rewritten_text_spreads_over_the_old_words() {
        let retimed = retimed_words(&words(), "Salut", millis(1200));
        assert_eq!(retimed, [word("Salut", 0, 1000)]);
        let from_nothing = retimed_words(&[], "un deux", millis(1000));
        // Weights 3 and 5.
        assert_eq!(from_nothing, [word("un", 0, 375), word("deux", 375, 1000)]);
    }
}
