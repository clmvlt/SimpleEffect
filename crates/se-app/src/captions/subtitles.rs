//! Captions ↔ subtitle cues (SRT): the cues a sequence's captions make, the caption drafts a
//! file's cues make.

use se_domain::{
    Flicks, ItemKind, Project, Sequence, TimeRange,
    captions::{CaptionDraft, Spread, caption_lines, distribute_words},
};

use crate::ports::SubtitleCue;

/// One cue per caption of `sequence` (every track), in time order: its time on the timeline and
/// its words as typed (letter case is a look, not text), in lines of its style's length.
#[must_use]
pub fn cues_of(project: &Project, sequence: &Sequence) -> Vec<SubtitleCue> {
    let mut cues: Vec<SubtitleCue> = sequence
        .tracks
        .iter()
        .flat_map(|track| &track.items)
        .filter_map(|item| {
            let ItemKind::Caption(caption) = &item.kind else {
                return None;
            };
            if caption.words.is_empty() {
                return None;
            }
            let max_chars = project
                .caption_styles
                .iter()
                .find(|style| style.id == caption.style_id)
                .map_or(usize::MAX, |style| {
                    usize::try_from(style.max_chars_per_line).unwrap_or(usize::MAX)
                });
            let texts: Vec<&str> = caption
                .words
                .iter()
                .map(|word| word.text.as_str())
                .collect();
            let text = caption_lines(&texts, max_chars)
                .into_iter()
                .map(|line| texts[line].join(" "))
                .collect::<Vec<_>>()
                .join("\n");
            Some(SubtitleCue {
                start: item.timeline_range.start(),
                end: item.timeline_range.end(),
                text,
            })
        })
        .collect();
    cues.sort_by_key(|cue| (cue.start, cue.end));
    cues
}

/// Caption drafts of imported cues: sorted by start, each cut where the next one begins (one
/// track cannot hold overlapping captions), words spread equally over each (no word timing is
/// known), cues without text or time skipped.
#[must_use]
pub fn drafts_of_cues(cues: &[SubtitleCue]) -> Vec<CaptionDraft> {
    let mut sorted: Vec<&SubtitleCue> = cues
        .iter()
        .filter(|cue| !cue.text.trim().is_empty())
        .collect();
    sorted.sort_by_key(|cue| (cue.start, cue.end));
    let mut drafts = Vec::with_capacity(sorted.len());
    let mut previous_end = Flicks::ZERO;
    for (index, cue) in sorted.iter().enumerate() {
        let start = cue.start.max(previous_end);
        let next_start = sorted.get(index + 1).map_or(cue.end, |next| next.start);
        let end = cue.end.min(next_start.max(start));
        let Ok(range) = TimeRange::from_start_end(start, end) else {
            continue;
        };
        if range.duration() <= Flicks::ZERO {
            continue;
        }
        let tokens: Vec<&str> = cue.text.split_whitespace().collect();
        drafts.push(CaptionDraft {
            range,
            words: distribute_words(&tokens, Flicks::ZERO, range.duration(), Spread::Equal),
        });
        previous_end = end;
    }
    drafts
}

/// The extent of `drafts` (from the first start to the last end).
#[must_use]
pub fn drafts_extent(drafts: &[CaptionDraft]) -> Option<TimeRange> {
    let start = drafts.iter().map(|draft| draft.range.start()).min()?;
    let end = drafts.iter().map(|draft| draft.range.end()).max()?;
    TimeRange::from_start_end(start, end).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use se_domain::{
        CaptionWord,
        test_support::{example_project, millis},
    };

    fn cue(start: i64, end: i64, text: &str) -> SubtitleCue {
        SubtitleCue {
            start: millis(start),
            end: millis(end),
            text: text.to_owned(),
        }
    }

    #[test]
    fn the_example_caption_makes_one_cue() {
        let project = example_project();
        let cues = cues_of(&project, &project.sequences[0]);
        assert_eq!(cues, [cue(8000, 10_000, "Merci d'être là")]);
    }

    #[test]
    fn overlapping_cues_are_cut_and_words_spread_equally() {
        let drafts = drafts_of_cues(&[
            cue(2000, 3000, "deux mots"),
            cue(0, 2500, "Bonjour\nà tous"),
            cue(4000, 4000, "vide"),
            cue(5000, 6000, "  "),
        ]);
        assert_eq!(drafts.len(), 2);
        assert_eq!(drafts[0].range, se_domain::test_support::range_ms(0, 2000));
        let texts: Vec<&str> = drafts[0]
            .words
            .iter()
            .map(|word| word.text.as_str())
            .collect();
        assert_eq!(texts, ["Bonjour", "à", "tous"]);
        let lengths: Vec<i64> = drafts[0]
            .words
            .iter()
            .map(|word: &CaptionWord| word.end.get() - word.start.get())
            .collect();
        assert!(
            lengths
                .iter()
                .all(|length| (length - lengths[0]).abs() <= 1)
        );
        assert_eq!(drafts[0].words[2].end, millis(2000));
        assert_eq!(
            drafts[1].range,
            se_domain::test_support::range_ms(2000, 1000)
        );
        assert_eq!(
            drafts_extent(&drafts),
            Some(se_domain::test_support::range_ms(0, 3000))
        );
    }
}
