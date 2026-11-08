//! Word cuts: the candidate ranges that remove chosen words of a sequence with natural margins.
//!
//! Removing words keeps the **longer** of the two pauses around them (the pause before a removed
//! sentence end, or the one after a removed sentence start), and cuts the shorter one down to a
//! breath before the next kept word. A removed word is cut a little before its start (word
//! starts are known to ±30 ms; ends from whisper are rather late, so the end itself is safe).
//! Margins never take more than half of a pause, so a cut never reaches a kept word. Clip edges
//! bound everything: words of other clips are not neighbours.

use std::collections::{BTreeMap, BTreeSet};

use se_domain::{Flicks, ItemId, SequenceWord, TimeRange};

/// Cut before a removed word's start (its onset), at most half the pause before it.
pub const ONSET_MARGIN: Flicks = Flicks::from_millis(30);

/// Silence kept next to a kept word when the pause on its side is cut, at most half of it.
pub const BREATH: Flicks = Flicks::from_millis(60);

/// The indices of `words` whose middle lies in one of `ranges`.
#[must_use]
pub fn words_in_ranges(words: &[SequenceWord], ranges: &[TimeRange]) -> BTreeSet<usize> {
    words
        .iter()
        .enumerate()
        .filter(|(_, word)| {
            let middle = midpoint(word.start, word.end);
            ranges.iter().any(|range| range.contains(middle))
        })
        .map(|(index, _)| index)
        .collect()
}

/// The candidate cuts removing the words at `removed` (indices into `words`, sorted by start):
/// one per run of consecutive removed words. `clips` gives the sequence range of each word's
/// clip.
#[must_use]
pub fn word_cut_ranges(
    words: &[SequenceWord],
    removed: &BTreeSet<usize>,
    clips: &BTreeMap<ItemId, TimeRange>,
) -> Vec<TimeRange> {
    runs(removed)
        .into_iter()
        .filter_map(|(first, last)| run_cut(words, first, last, clips))
        .collect()
}

/// Consecutive indices grouped as `(first, last)`.
fn runs(removed: &BTreeSet<usize>) -> Vec<(usize, usize)> {
    let mut runs: Vec<(usize, usize)> = Vec::new();
    for &index in removed {
        match runs.last_mut() {
            Some((_, last)) if index == *last + 1 => *last = index,
            _ => runs.push((index, index)),
        }
    }
    runs
}

/// The cut removing `words[first..=last]`.
fn run_cut(
    words: &[SequenceWord],
    first: usize,
    last: usize,
    clips: &BTreeMap<ItemId, TimeRange>,
) -> Option<TimeRange> {
    let (first_word, last_word) = (words.get(first)?, words.get(last)?);
    let first_clip = clips.get(&first_word.item_id)?;
    let last_clip = clips.get(&last_word.item_id)?;
    let previous_end = first
        .checked_sub(1)
        .and_then(|index| words.get(index))
        .filter(|word| word.item_id == first_word.item_id)
        .map_or(first_clip.start(), |word| word.end.max(first_clip.start()));
    let next_start = words
        .get(last + 1)
        .filter(|word| word.item_id == last_word.item_id)
        .map_or(last_clip.end(), |word| word.start.min(last_clip.end()));
    let gap_before = non_negative(first_word.start.checked_sub(previous_end)?);
    let gap_after = non_negative(next_start.checked_sub(last_word.end)?);
    let (start, end) = if gap_before >= gap_after {
        (
            first_word
                .start
                .checked_sub(ONSET_MARGIN.min(half(gap_before)))?,
            next_start.checked_sub(BREATH.min(half(gap_after)))?,
        )
    } else {
        (
            previous_end.checked_add(BREATH.min(half(gap_before)))?,
            last_word
                .end
                .checked_add(ONSET_MARGIN.min(half(gap_after)))?,
        )
    };
    let start = start.max(first_clip.start());
    let end = end
        .min(last_clip.end())
        .max(last_word.end.min(last_clip.end()));
    TimeRange::from_start_end(start, end)
        .ok()
        .filter(|range| !range.is_empty())
}

fn non_negative(value: Flicks) -> Flicks {
    value.max(Flicks::ZERO)
}

fn half(value: Flicks) -> Flicks {
    Flicks::new(value.get() / 2).unwrap_or(Flicks::ZERO)
}

fn midpoint(start: Flicks, end: Flicks) -> Flicks {
    Flicks::new(start.get() + (end.get() - start.get()) / 2).unwrap_or(start)
}

#[cfg(test)]
mod tests {
    use se_domain::AssetId;

    use super::*;

    fn ms(value: i64) -> Flicks {
        Flicks::new(value * Flicks::PER_SECOND / 1000).unwrap()
    }

    fn span(start: i64, end: i64) -> TimeRange {
        TimeRange::from_start_end(ms(start), ms(end)).unwrap()
    }

    fn word(text: &str, start: i64, end: i64, item: u64) -> SequenceWord {
        SequenceWord {
            text: text.to_owned(),
            start: ms(start),
            end: ms(end),
            confidence: 0.9,
            asset_id: AssetId::from_test_index(1),
            item_id: ItemId::from_test_index(item),
            word_index: 0,
            segment: 0,
        }
    }

    /// « Bonjour à tous. ‖ Euh je commence » in one clip [0, 5 s).
    fn sentence() -> (Vec<SequenceWord>, BTreeMap<ItemId, TimeRange>) {
        let words = vec![
            word("Bonjour", 200, 600, 1),
            word("à", 650, 700, 1),
            word("tous.", 750, 1100, 1),
            word("Euh", 2000, 2300, 1),
            word("je", 2400, 2500, 1),
            word("commence", 2550, 3100, 1),
        ];
        let clips = BTreeMap::from([(ItemId::from_test_index(1), span(0, 5000))]);
        (words, clips)
    }

    fn to_ms(ranges: &[TimeRange]) -> Vec<(i64, i64)> {
        let millis = |time: Flicks| time.get() * 1000 / Flicks::PER_SECOND;
        ranges
            .iter()
            .map(|range| (millis(range.start()), millis(range.end())))
            .collect()
    }

    #[test]
    fn words_are_chosen_by_their_middle() {
        let (words, _) = sentence();
        assert_eq!(
            words_in_ranges(&words, &[span(640, 1000)]),
            BTreeSet::from([1, 2])
        );
        assert_eq!(words_in_ranges(&words, &[span(0, 100)]), BTreeSet::new());
    }

    #[test]
    fn a_filler_after_a_long_pause_keeps_that_pause_and_cuts_to_a_breath() {
        let (words, clips) = sentence();
        // « Euh »: 900 ms of pause before, 100 ms after → keep the pause before.
        let cuts = word_cut_ranges(&words, &BTreeSet::from([3]), &clips);
        assert_eq!(to_ms(&cuts), [(1970, 2350)]);
    }

    #[test]
    fn a_sentence_end_keeps_the_pause_after_it() {
        let (words, clips) = sentence();
        // « à tous. »: 50 ms before, 900 ms after → keep the pause after.
        let cuts = word_cut_ranges(&words, &BTreeSet::from([1, 2]), &clips);
        assert_eq!(to_ms(&cuts), [(625, 1130)]);
    }

    #[test]
    fn clip_edges_bound_cuts_and_other_clips_are_not_neighbours() {
        let mut words = sentence().0;
        for word in &mut words[3..] {
            word.item_id = ItemId::from_test_index(2);
        }
        let clips = BTreeMap::from([
            (ItemId::from_test_index(1), span(0, 1500)),
            (ItemId::from_test_index(2), span(1500, 5000)),
        ]);
        // « Euh » opens clip 2: 500 ms from the clip start, 100 ms to « je ».
        let cuts = word_cut_ranges(&words, &BTreeSet::from([3]), &clips);
        assert_eq!(to_ms(&cuts), [(1970, 2350)]);
        // « tous. » ends clip 1: 50 ms before it, 400 ms to the clip end.
        let cuts = word_cut_ranges(&words, &BTreeSet::from([2]), &clips);
        assert_eq!(to_ms(&cuts), [(725, 1130)]);
    }

    #[test]
    fn separate_runs_give_separate_cuts() {
        let (words, clips) = sentence();
        let cuts = word_cut_ranges(&words, &BTreeSet::from([0, 4]), &clips);
        assert_eq!(cuts.len(), 2);
    }
}
