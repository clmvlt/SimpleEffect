//! Segmentation: words spoken in a sequence → caption drafts.
//!
//! Words are grouped in speaking order. A new caption starts before a word when the pause before
//! it reaches `pause`, when it would not fit in `max_lines` lines of `max_chars_per_line`
//! characters (the group is then cut after its last clause punctuation if that keeps a third of
//! the room filled), when the caption would last longer than `max_duration`, after a sentence end
//! (`.`, `!`, `?`, `…`), or after a clause end (`,`, `;`, `:`) once half the room is filled. A
//! word made only of punctuation never starts a caption.
//!
//! Each caption runs from its first word's start to its last word's end, lengthened to
//! `min_duration` and over pauses shorter than `pause` (never into the next caption nor past the
//! bounds, never beyond `max_duration` for closed pauses), then snapped outwards to frames when a
//! frame rate is given.

use super::{
    lines::{caption_lines, line_chars},
    words::tidy_words,
};
use crate::{CaptionStyle, CaptionWord, Flicks, FrameRate, RoundingMode, TimeRange};

/// A word spoken in a sequence, in sequence time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimedWord {
    /// The word (punctuation attached).
    pub text: String,
    /// When it starts.
    pub start: Flicks,
    /// When it ends.
    pub end: Flicks,
}

/// Limits of [`segment_words`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentationParams {
    /// Longest line in characters.
    pub max_chars_per_line: u32,
    /// Lines per caption.
    pub max_lines: u32,
    /// Shortest caption (lengthened up to it when room allows).
    pub min_duration: Flicks,
    /// Longest caption.
    pub max_duration: Flicks,
    /// A pause at least this long starts a new caption; shorter gaps between captions are closed.
    pub pause: Flicks,
    /// Frames to snap caption edges to.
    pub frame_rate: Option<FrameRate>,
}

impl SegmentationParams {
    /// Shortest caption by default (0.7 s).
    pub const DEFAULT_MIN_DURATION: Flicks = Flicks::from_millis(700);
    /// Longest caption by default (6 s).
    pub const DEFAULT_MAX_DURATION: Flicks = Flicks::from_millis(6000);
    /// Pause starting a new caption by default (0.6 s).
    pub const DEFAULT_PAUSE: Flicks = Flicks::from_millis(600);

    /// The defaults with the line limits of `style`.
    #[must_use]
    pub const fn for_style(style: &CaptionStyle, frame_rate: Option<FrameRate>) -> Self {
        Self {
            max_chars_per_line: style.max_chars_per_line,
            max_lines: style.max_lines,
            min_duration: Self::DEFAULT_MIN_DURATION,
            max_duration: Self::DEFAULT_MAX_DURATION,
            pause: Self::DEFAULT_PAUSE,
            frame_rate,
        }
    }
}

/// A caption to create: where it sits in the sequence and its item-relative words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptionDraft {
    /// Its timeline range.
    pub range: TimeRange,
    /// Its words, relative to `range.start()`.
    pub words: Vec<CaptionWord>,
}

/// The captions of the words whose middle lies in `bounds` (see the module's rules). Drafts are
/// sorted, apart and inside `bounds`; every such word is in exactly one draft, in order.
#[must_use]
pub fn segment_words(
    words: &[TimedWord],
    params: &SegmentationParams,
    bounds: TimeRange,
) -> Vec<CaptionDraft> {
    let inside: Vec<TimedWord> = words
        .iter()
        .filter(|word| !word.text.trim().is_empty() && bounds.contains(middle(word)))
        .map(|word| TimedWord {
            text: word.text.trim().replace(['\r', '\n'], " "),
            ..word.clone()
        })
        .collect();
    let groups = group_words(&inside, params);
    let spans = caption_spans(&groups, params, bounds);
    groups
        .iter()
        .zip(spans)
        .filter_map(|(group, (start, end))| draft(group, start, end))
        .collect()
}

fn middle(word: &TimedWord) -> Flicks {
    CaptionWord::new("", word.start, word.end).middle()
}

/// The words cut into caption groups. A group never starts at the same instant as the previous
/// one (zero-length words), so every caption gets time.
fn group_words<'a>(words: &'a [TimedWord], params: &SegmentationParams) -> Vec<Vec<&'a TimedWord>> {
    let capacity = room(params);
    let mut groups = Vec::new();
    let mut current: Vec<&TimedWord> = Vec::new();
    for word in words {
        if punctuation_only(&word.text) && !current.is_empty() && !fits(&current, word, params) {
            // Punctuation never starts a caption: it takes the previous word along instead.
            let last_start = current.last().map_or(Flicks::ZERO, |last| last.start);
            if current.len() >= 2 && last_start > current[0].start {
                let rest = current.split_off(current.len() - 1);
                groups.push(std::mem::replace(&mut current, rest));
            }
        } else if let Some(&last) = current.last()
            && !punctuation_only(&word.text)
            && word.start > current[0].start
        {
            let pause = word.start.checked_sub(last.end).unwrap_or(Flicks::ZERO);
            let too_long = word
                .end
                .checked_sub(current[0].start)
                .unwrap_or(Flicks::ZERO)
                > params.max_duration;
            let filled = chars_of(&current) * 2 >= capacity;
            if pause >= params.pause
                || too_long
                || ends_sentence(&last.text)
                || (filled && ends_clause(&last.text))
            {
                groups.push(std::mem::take(&mut current));
            } else if !fits(&current, word, params) {
                let rest = current.split_off(clause_cut(&current, capacity));
                groups.push(std::mem::replace(&mut current, rest));
                if !current.is_empty()
                    && !fits(&current, word, params)
                    && word.start > current[0].start
                {
                    groups.push(std::mem::take(&mut current));
                }
            }
        }
        current.push(word);
    }
    if !current.is_empty() {
        groups.push(current);
    }
    groups
}

/// Characters a caption may hold in all.
fn room(params: &SegmentationParams) -> usize {
    usize::try_from(params.max_chars_per_line.saturating_mul(params.max_lines))
        .unwrap_or(usize::MAX)
}

fn chars_of(group: &[&TimedWord]) -> usize {
    group
        .iter()
        .map(|word| line_chars(&word.text) + 1)
        .sum::<usize>()
        .saturating_sub(1)
}

/// Whether `group` followed by `word` fits in the caption's lines.
fn fits(group: &[&TimedWord], word: &TimedWord, params: &SegmentationParams) -> bool {
    let texts: Vec<&str> = group
        .iter()
        .map(|word| word.text.as_str())
        .chain([word.text.as_str()])
        .collect();
    let max_chars = usize::try_from(params.max_chars_per_line).unwrap_or(usize::MAX);
    let max_lines = usize::try_from(params.max_lines).unwrap_or(usize::MAX);
    caption_lines(&texts, max_chars).len() <= max_lines
}

/// Where to cut a full group: after its last clause end leaving at least a third of the room
/// filled, else at its end (everything stays).
fn clause_cut(group: &[&TimedWord], capacity: usize) -> usize {
    (1..group.len())
        .rev()
        .find(|&end| {
            ends_clause(&group[end - 1].text)
                && chars_of(&group[..end]) * 3 >= capacity
                && group[end].start > group[0].start
        })
        .unwrap_or(group.len())
}

/// The last character of `text` that is not a closing quote or bracket.
fn last_mark(text: &str) -> Option<char> {
    text.chars().rev().find(|character| {
        !matches!(
            character,
            '»' | '"' | '\'' | ')' | ']' | '”' | '’' | '\u{a0}' | ' '
        )
    })
}

fn ends_sentence(text: &str) -> bool {
    matches!(last_mark(text), Some('.' | '!' | '?' | '…'))
}

fn ends_clause(text: &str) -> bool {
    matches!(last_mark(text), Some(',' | ';' | ':'))
}

fn punctuation_only(text: &str) -> bool {
    text.chars().all(|character| !character.is_alphanumeric())
}

/// Each group's caption extent (module rules), before frame snapping.
fn caption_spans(
    groups: &[Vec<&TimedWord>],
    params: &SegmentationParams,
    bounds: TimeRange,
) -> Vec<(Flicks, Flicks)> {
    let starts: Vec<Flicks> = groups
        .iter()
        .map(|group| {
            group
                .first()
                .map_or(bounds.start(), |word| word.start.max(bounds.start()))
        })
        .collect();
    let spans: Vec<(Flicks, Flicks)> = groups
        .iter()
        .enumerate()
        .map(|(index, group)| {
            let start = starts[index];
            let limit = starts.get(index + 1).copied().unwrap_or(bounds.end());
            let last_end = group
                .last()
                .map_or(start, |word| word.end)
                .clamp(start, limit);
            let lengthened = start
                .checked_add(params.min_duration)
                .map_or(limit, |minimum| minimum.min(limit))
                .max(last_end);
            let closes = limit
                .checked_sub(lengthened)
                .is_some_and(|gap| gap < params.pause)
                && limit
                    .checked_sub(start)
                    .is_some_and(|length| length <= params.max_duration);
            (start, if closes { limit } else { lengthened })
        })
        .collect();
    match params.frame_rate {
        Some(rate) => snapped(&spans, rate, bounds),
        None => spans,
    }
}

/// Spans snapped outwards to frames where room allows: starts floor to a frame (never before the
/// previous caption's end nor the bounds), ends ceil to a frame (never past the next caption's
/// floored start, its exact start, nor the bounds). Spans only grow and stay apart.
fn snapped(
    spans: &[(Flicks, Flicks)],
    rate: FrameRate,
    bounds: TimeRange,
) -> Vec<(Flicks, Flicks)> {
    let to_frame = |time: Flicks, rounding| {
        Flicks::from_frames(time.to_frames(rate, rounding), rate).unwrap_or(time)
    };
    let mut previous_end = bounds.start();
    spans
        .iter()
        .enumerate()
        .map(|(index, &(start, end))| {
            let next_start = spans.get(index + 1).map_or(bounds.end(), |&(next, _)| next);
            let cap = to_frame(next_start, RoundingMode::Floor)
                .max(end)
                .min(next_start);
            let snapped_start = to_frame(start, RoundingMode::Floor)
                .max(previous_end)
                .min(start);
            let snapped_end = to_frame(end, RoundingMode::Ceil).min(cap).max(end);
            previous_end = snapped_end;
            (snapped_start, snapped_end)
        })
        .collect()
}

fn draft(group: &[&TimedWord], start: Flicks, end: Flicks) -> Option<CaptionDraft> {
    let range = TimeRange::from_start_end(start, end).ok()?;
    if range.duration() <= Flicks::ZERO {
        return None;
    }
    let local = |time: Flicks| time.checked_sub(start).unwrap_or(Flicks::ZERO);
    let words = group
        .iter()
        .map(|word| CaptionWord::new(word.text.clone(), local(word.start), local(word.end)))
        .collect();
    Some(CaptionDraft {
        range,
        words: tidy_words(words, range.duration()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{millis, range_ms};

    fn spoken(words: &[(&str, i64, i64)]) -> Vec<TimedWord> {
        words
            .iter()
            .map(|&(text, start, end)| TimedWord {
                text: text.to_owned(),
                start: millis(start),
                end: millis(end),
            })
            .collect()
    }

    fn params(max_chars: u32, max_lines: u32) -> SegmentationParams {
        SegmentationParams {
            max_chars_per_line: max_chars,
            max_lines,
            min_duration: millis(700),
            max_duration: millis(6000),
            pause: millis(600),
            frame_rate: None,
        }
    }

    fn texts(drafts: &[CaptionDraft]) -> Vec<String> {
        drafts
            .iter()
            .map(|draft| {
                draft
                    .words
                    .iter()
                    .map(|word| word.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect()
    }

    /// Segmentation table: each case is words, limits and the caption texts expected.
    #[test]
    fn segmentation_table() {
        let flowing = [
            ("Bonjour", 0, 300),
            ("à", 300, 400),
            ("tous,", 400, 700),
            ("on", 750, 850),
            ("commence", 850, 1300),
            ("tout", 1300, 1500),
            ("de", 1500, 1600),
            ("suite.", 1600, 2000),
            ("Prêts", 2100, 2400),
            ("?", 2400, 2450),
            ("Allons-y", 3500, 4000),
        ];
        let cases: [(&str, SegmentationParams, &[&str]); 4] = [
            (
                "two lines of 32: sentence end, punctuation-only word, long pause",
                params(32, 2),
                &[
                    "Bonjour à tous, on commence tout de suite.",
                    "Prêts ?",
                    "Allons-y",
                ],
            ),
            (
                "one line of 18: fills lines, cuts after the comma once half full",
                params(18, 1),
                &[
                    "Bonjour à tous,",
                    "on commence tout",
                    "de suite.",
                    "Prêts ?",
                    "Allons-y",
                ],
            ),
            (
                "one line of 12: an overflow cuts after the clause",
                params(12, 1),
                &[
                    "Bonjour à",
                    "tous,",
                    "on commence",
                    "tout de",
                    "suite.",
                    "Prêts ?",
                    "Allons-y",
                ],
            ),
            (
                "max duration of 1 s",
                SegmentationParams {
                    max_duration: millis(1000),
                    ..params(80, 2)
                },
                &[
                    "Bonjour à tous, on",
                    "commence tout de",
                    "suite.",
                    "Prêts ?",
                    "Allons-y",
                ],
            ),
        ];
        for (name, limits, expected) in cases {
            let drafts = segment_words(&spoken(&flowing), &limits, range_ms(0, 10_000));
            assert_eq!(texts(&drafts), expected, "{name}");
        }
    }

    #[test]
    fn captions_are_lengthened_closed_and_relative() {
        let words = spoken(&[
            ("Oui.", 1000, 1200),
            ("Non", 1500, 1800),
            ("merci", 3000, 3400),
        ]);
        let drafts = segment_words(&words, &params(32, 2), range_ms(0, 3600));
        let ranges: Vec<_> = drafts.iter().map(|draft| draft.range).collect();
        // « Oui. » reaches the next caption (a 300 ms pause is closed), « Non » gets 700 ms,
        // « merci » stops at the bounds.
        assert_eq!(
            ranges,
            [
                range_ms(1000, 500),
                range_ms(1500, 700),
                range_ms(3000, 600)
            ]
        );
        assert_eq!(
            drafts[2].words,
            [CaptionWord::new("merci", millis(0), millis(400))]
        );
    }

    #[test]
    fn words_outside_the_bounds_are_left_out_and_edges_snap_to_frames() {
        let words = spoken(&[
            ("avant", 0, 900),
            ("dedans", 1010, 1490),
            ("après", 2600, 3000),
        ]);
        let snapping = SegmentationParams {
            frame_rate: Some(FrameRate::FPS_25),
            ..params(32, 2)
        };
        let drafts = segment_words(&words, &snapping, range_ms(1000, 1500));
        assert_eq!(texts(&drafts), ["dedans"]);
        // 1.010 s floors to frame 25 (1.000 s); 1.710 s ceils to 1.720 s.
        assert_eq!(drafts[0].range, range_ms(1000, 720));
        assert_eq!(drafts[0].words[0].start, millis(10));
    }
}
