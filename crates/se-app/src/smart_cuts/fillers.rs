//! Filler words: the default French and English lists, the matching rules and their detection in
//! a sequence's words.
//!
//! Matching ignores case, accents and punctuation, and repeated letters count once
//! (« Euuuh… » is « euh », « hmmm » is « hm »). A filler may be a phrase (« you know »): its
//! words must follow each other in one clip. Occurrences longer than the maximum duration are
//! skipped (a « ben » lasting two seconds is a mistimed word, not a hesitation).

use se_domain::{AssetId, Flicks, SequenceWord};
use se_protocol::FillerList;

/// The default lists (« Réglages » start from these).
#[must_use]
pub fn default_filler_lists() -> Vec<FillerList> {
    let list = |language: &str, words: &[&str]| FillerList {
        language: language.to_owned(),
        words: words.iter().map(|word| (*word).to_owned()).collect(),
    };
    vec![
        list(
            "fr",
            &[
                "euh", "heu", "hum", "hmm", "mmh", "bah", "ben", "beh", "euhm",
            ],
        ),
        list(
            "en",
            &["um", "uh", "uhm", "erm", "er", "hmm", "mm", "you know"],
        ),
    ]
}

/// The default longest filler occurrence: 1.5 s.
pub const DEFAULT_FILLER_MAX_DURATION: Flicks = Flicks::from_millis(1500);

/// `text` lowercased, without accents or punctuation, repeated letters once.
#[must_use]
pub fn normalized(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars().flat_map(char::to_lowercase) {
        let Some(base) = base_letter(character) else {
            continue;
        };
        if out.ends_with(base) {
            continue;
        }
        out.push(base);
    }
    out
}

/// The letter or digit `character` reads as without its accent; `None` for punctuation and
/// spaces.
fn base_letter(character: char) -> Option<char> {
    let base = match character {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
        'ç' => 'c',
        'è' | 'é' | 'ê' | 'ë' => 'e',
        'ì' | 'í' | 'î' | 'ï' => 'i',
        'ñ' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
        'ù' | 'ú' | 'û' | 'ü' => 'u',
        'ý' | 'ÿ' => 'y',
        other if other.is_alphanumeric() => other,
        _ => return None,
    };
    Some(base)
}

/// Filler phrases, normalised word by word, for one transcript language.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FillerPhrases(Vec<Vec<String>>);

impl FillerPhrases {
    /// The phrases of `words` (empty entries dropped).
    #[must_use]
    pub fn new<'a>(words: impl IntoIterator<Item = &'a String>) -> Self {
        let mut phrases: Vec<Vec<String>> = words
            .into_iter()
            .map(|phrase| {
                phrase
                    .split_whitespace()
                    .map(normalized)
                    .filter(|word| !word.is_empty())
                    .collect::<Vec<_>>()
            })
            .filter(|phrase| !phrase.is_empty())
            .collect();
        // Longest phrases first, so « you know » wins over a « you » list entry.
        phrases.sort_by_key(|phrase| std::cmp::Reverse(phrase.len()));
        phrases.dedup();
        Self(phrases)
    }

    /// The phrases for transcripts in `language`: its list, or every list when it has none.
    #[must_use]
    pub fn for_language(lists: &[FillerList], language: &str) -> Self {
        let own: Vec<&FillerList> = lists
            .iter()
            .filter(|list| list.language.eq_ignore_ascii_case(language))
            .collect();
        let chosen = if own.is_empty() {
            lists.iter().collect()
        } else {
            own
        };
        Self::new(chosen.into_iter().flat_map(|list| &list.words))
    }

    /// How many words of `words` from `start` form a filler phrase (0: none).
    fn match_at(&self, words: &[SequenceWord], start: usize) -> usize {
        let Some(first) = words.get(start) else {
            return 0;
        };
        self.0
            .iter()
            .find(|phrase| {
                phrase.iter().enumerate().all(|(offset, expected)| {
                    words.get(start + offset).is_some_and(|word| {
                        word.item_id == first.item_id && normalized(&word.text) == *expected
                    })
                })
            })
            .map_or(0, Vec::len)
    }
}

/// A filler found: the indices of its first and last words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillerSpan {
    /// Index of its first word.
    pub first: usize,
    /// Index of its last word.
    pub last: usize,
}

/// The fillers among `words[candidates]` (indices of the words to examine, sorted), with the
/// phrases of each word's asset (`phrases_of`); occurrences longer than `max_duration` are
/// skipped.
#[must_use]
pub fn find_fillers<'a>(
    words: &[SequenceWord],
    candidates: &[usize],
    phrases_of: impl Fn(AssetId) -> &'a FillerPhrases,
    max_duration: Flicks,
) -> Vec<FillerSpan> {
    let mut found = Vec::new();
    let mut next_free = 0;
    for &index in candidates {
        if index < next_free {
            continue;
        }
        let Some(word) = words.get(index) else {
            continue;
        };
        let length = phrases_of(word.asset_id).match_at(words, index);
        if length == 0 {
            continue;
        }
        let last = index + length - 1;
        let lasts = words
            .get(last)
            .and_then(|end| end.end.checked_sub(word.start))
            .is_some_and(|duration| duration <= max_duration);
        if lasts {
            found.push(FillerSpan { first: index, last });
            next_free = last + 1;
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use se_domain::ItemId;

    use super::*;

    fn ms(value: i64) -> Flicks {
        Flicks::new(value * Flicks::PER_SECOND / 1000).unwrap()
    }

    fn word(text: &str, start: i64, end: i64) -> SequenceWord {
        SequenceWord {
            text: text.to_owned(),
            start: ms(start),
            end: ms(end),
            confidence: 0.9,
            asset_id: AssetId::from_test_index(1),
            item_id: ItemId::from_test_index(1),
            word_index: 0,
            segment: 0,
        }
    }

    #[test]
    fn normalisation_ignores_case_accents_punctuation_and_repeats() {
        assert_eq!(normalized("Euuuh…"), "euh");
        assert_eq!(normalized("HMMM,"), "hm");
        assert_eq!(normalized("Hé !"), "he");
        assert_eq!(normalized("l'homme"), "lhome");
    }

    #[test]
    fn a_language_without_a_list_uses_every_list() {
        let lists = default_filler_lists();
        let french = FillerPhrases::for_language(&lists, "FR");
        let german = FillerPhrases::for_language(&lists, "de");
        assert!(french.0.contains(&vec!["euh".to_owned()]));
        assert!(!french.0.contains(&vec!["um".to_owned()]));
        assert!(german.0.contains(&vec!["um".to_owned()]));
        assert!(german.0.contains(&vec!["euh".to_owned()]));
    }

    #[test]
    fn fillers_phrases_and_durations() {
        let words = vec![
            word("Euh,", 0, 300),
            word("alors", 400, 700),
            word("you", 800, 900),
            word("know", 900, 1100),
            word("benn", 1200, 4000),
            word("Hum.", 4100, 4400),
        ];
        let phrases = FillerPhrases::new(
            ["euh", "you know", "ben", "hum"]
                .iter()
                .map(|word| (*word).to_owned())
                .collect::<Vec<_>>()
                .iter(),
        );
        let all: Vec<usize> = (0..words.len()).collect();
        let found = find_fillers(&words, &all, |_| &phrases, ms(1500));
        assert_eq!(
            found,
            [
                FillerSpan { first: 0, last: 0 },
                FillerSpan { first: 2, last: 3 },
                FillerSpan { first: 5, last: 5 },
            ],
            "the 2.8 s « benn » is skipped"
        );
        let only_late = find_fillers(&words, &[4, 5], |_| &phrases, ms(5000));
        assert_eq!(
            only_late,
            [
                FillerSpan { first: 4, last: 4 },
                FillerSpan { first: 5, last: 5 }
            ]
        );
    }
}
