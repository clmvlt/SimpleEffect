//! Caption rules (prompt 23): the shared line-breaking vectors (`docs/test-vectors/
//! caption-lines.json`, also run by the engine) and properties of segmentation, text edits and
//! caption splits.

#![allow(clippy::expect_used, clippy::panic)]

use proptest::prelude::*;
use se_domain::{
    CaptionWord, Flicks, FrameRate, TimeRange,
    captions::{
        SegmentationParams, TimedWord, caption_lines, merged_words, retimed_words, segment_words,
        split_point,
    },
};
use serde::Deserialize;

const LINE_VECTORS: &str = include_str!("../../../docs/test-vectors/caption-lines.json");

#[derive(Deserialize)]
struct LineVectors {
    cases: Vec<LineCase>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LineCase {
    name: String,
    words: Vec<String>,
    max_chars: usize,
    lines: Vec<[usize; 2]>,
}

#[test]
fn shared_line_vectors_break_identically() {
    let vectors: LineVectors = serde_json::from_str(LINE_VECTORS).expect("valid vectors");
    for case in vectors.cases {
        let lines: Vec<[usize; 2]> = caption_lines(&case.words, case.max_chars)
            .into_iter()
            .map(|line| [line.start, line.end])
            .collect();
        assert_eq!(lines, case.lines, "{}", case.name);
    }
}

fn millis(value: i64) -> Flicks {
    Flicks::new(value * Flicks::PER_SECOND / 1000).expect("small times")
}

/// Sorted, apart words with random texts (some punctuation-only), lengths and pauses.
fn spoken_words() -> impl Strategy<Value = Vec<TimedWord>> {
    let texts = prop::sample::select(vec![
        "Bonjour",
        "à",
        "tous,",
        "on",
        "commence.",
        "?",
        "anticonstitutionnellement",
        "été",
        "là",
        "Allons-y",
        "!",
        "vraiment",
        "oui",
    ]);
    prop::collection::vec((texts, 0..900_i64, 0..1500_i64), 0..40).prop_map(|specs| {
        let mut time = 0;
        specs
            .into_iter()
            .map(|(text, length, pause)| {
                let start = time + pause;
                time = start + length;
                TimedWord {
                    text: text.to_owned(),
                    start: millis(start),
                    end: millis(time),
                }
            })
            .collect()
    })
}

fn assert_valid_words(words: &[CaptionWord], duration: Flicks) {
    let mut previous_end = Flicks::ZERO;
    for word in words {
        assert_ne!(word.text.trim(), "");
        assert!(word.start >= previous_end, "{words:?}");
        assert!(word.start <= word.end && word.end <= duration, "{words:?}");
        previous_end = word.end;
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn segmentation_keeps_every_word_once_inside_valid_captions(
        words in spoken_words(),
        max_chars in 8_u32..40,
        max_lines in 1_u32..3,
        bounds_start in 0_i64..5000,
        bounds_length in 1_i64..60_000,
        snap in any::<bool>(),
    ) {
        let params = SegmentationParams {
            max_chars_per_line: max_chars,
            max_lines,
            min_duration: millis(700),
            max_duration: millis(6000),
            pause: millis(600),
            frame_rate: snap.then_some(FrameRate::FPS_29_97),
        };
        let bounds = TimeRange::new(millis(bounds_start), millis(bounds_length)).expect("range");
        let drafts = segment_words(&words, &params, bounds);
        let mut previous_end = bounds.start();
        let mut texts = Vec::new();
        for draft in &drafts {
            prop_assert!(draft.range.start() >= previous_end);
            prop_assert!(draft.range.end() <= bounds.end());
            prop_assert!(draft.range.duration() > Flicks::ZERO);
            assert_valid_words(&draft.words, draft.range.duration());
            let line_count = caption_lines(
                &draft.words.iter().map(|word| word.text.as_str()).collect::<Vec<_>>(),
                usize::try_from(max_chars).expect("small"),
            )
            .len();
            let long_word = draft.words.iter().any(|word| word.text.chars().count() > 20);
            // Only a word longer than a line, or punctuation pulled along, overflows.
            prop_assert!(line_count <= max_lines as usize || long_word || draft.words.len() <= 2);
            previous_end = draft.range.end();
            texts.extend(draft.words.iter().map(|word| word.text.clone()));
        }
        let expected: Vec<String> = words
            .iter()
            .filter(|word| {
                bounds.contains(CaptionWord::new("", word.start, word.end).middle())
            })
            .map(|word| word.text.clone())
            .collect();
        prop_assert_eq!(texts, expected);
    }

    #[test]
    fn edited_texts_give_valid_words_one_per_token(
        words in spoken_words(),
        edit in prop::collection::vec(prop::sample::select(vec!["Bonjour", "à", "toi", "et", "là", "!"]), 0..12),
        keep_some in any::<bool>(),
    ) {
        let duration = words.last().map_or(millis(1000), |word| word.end).max(millis(1));
        let caption: Vec<CaptionWord> = words
            .iter()
            .map(|word| CaptionWord::new(word.text.clone(), word.start, word.end))
            .collect();
        let mut tokens: Vec<String> = edit.iter().map(|token| (*token).to_owned()).collect();
        if keep_some {
            tokens.extend(caption.iter().take(3).map(|word| word.text.clone()));
        }
        let text = tokens.join(" ");
        let retimed = retimed_words(&caption, &text, duration);
        assert_valid_words(&retimed, duration);
        let texts: Vec<&str> = retimed.iter().map(|word| word.text.as_str()).collect();
        prop_assert_eq!(texts, text.split_whitespace().collect::<Vec<_>>());
    }

    #[test]
    fn substitutions_keep_every_timing(words in spoken_words(), index in any::<prop::sample::Index>()) {
        prop_assume!(!words.is_empty());
        let duration = words.last().map_or(millis(1), |word| word.end).max(millis(1));
        let caption: Vec<CaptionWord> = words
            .iter()
            .map(|word| CaptionWord::new(word.text.clone(), word.start, word.end))
            .collect();
        let changed = index.index(caption.len());
        let text = caption
            .iter()
            .enumerate()
            .map(|(position, word)| if position == changed { "corrigé" } else { word.text.as_str() })
            .collect::<Vec<_>>()
            .join(" ");
        let retimed = retimed_words(&caption, &text, duration);
        prop_assert_eq!(retimed.len(), caption.len());
        for (before, after) in caption.iter().zip(&retimed) {
            prop_assert_eq!((before.start, before.end), (after.start, after.end));
        }
    }

    #[test]
    fn splitting_between_words_then_merging_gives_the_words_back(
        words in spoken_words(),
        index in any::<prop::sample::Index>(),
    ) {
        prop_assume!(words.len() >= 2);
        let caption: Vec<CaptionWord> = words
            .iter()
            .map(|word| CaptionWord::new(word.text.clone(), word.start, word.end))
            .collect();
        let at = 1 + index.index(caption.len() - 1);
        let cut = split_point(&caption, at).expect("a word on each side");
        let (left, right): (Vec<_>, Vec<_>) =
            caption.iter().cloned().partition(|word| word.middle() < cut);
        let right: Vec<CaptionWord> = right
            .into_iter()
            .map(|word| CaptionWord {
                start: word.start.checked_sub(cut).expect("after the cut"),
                end: word.end.checked_sub(cut).expect("after the cut"),
                ..word
            })
            .collect();
        prop_assert_eq!(left.len(), at);
        prop_assert_eq!(merged_words(&left, &right, cut), caption);
    }
}
