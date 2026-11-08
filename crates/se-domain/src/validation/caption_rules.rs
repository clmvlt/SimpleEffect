//! Rules of captions (prompt 23): `VALUE_OUT_OF_RANGE` for caption style values,
//! `UNKNOWN_REFERENCE` for a caption's style, `INVALID_CAPTION_WORDS` for its words (non-empty,
//! without line breaks, in speaking order, apart, inside the item).

use super::{Path, Report, ViolationCode, item_rules::ItemContext, report::seconds};
use crate::{CaptionItem, CaptionWord, Flicks, Project};

/// Checks the value ranges of every caption style of the project.
pub(super) fn check_styles(project: &Project, report: &mut Report) {
    let styles = Path::default().field("captionStyles");
    for style in &project.caption_styles {
        let path = styles.key(style.id);
        for (field, rule) in style.problems() {
            report.push(
                ViolationCode::ValueOutOfRange,
                &path.field(field),
                format!("caption style {}: {field} {rule}", style.id),
            );
        }
    }
}

/// Checks a caption's style reference and words (paths relative to `kind_path`).
pub(super) fn check_caption(
    caption: &CaptionItem,
    context: &ItemContext<'_>,
    kind_path: &Path,
    report: &mut Report,
) {
    if !context.index.has_caption_style(caption.style_id) {
        report.push(
            ViolationCode::UnknownReference,
            &kind_path.field("styleId"),
            format!(
                "caption style {} does not exist; use a style of the project's captionStyles",
                caption.style_id
            ),
        );
    }
    let duration = context.item.timeline_range.duration();
    let words = kind_path.field("words");
    let mut previous_end = Flicks::ZERO;
    for (index, word) in caption.words.iter().enumerate() {
        if let Some(problem) = word_problem(word, previous_end, duration) {
            report.push(
                ViolationCode::InvalidCaptionWords,
                &words.key(index),
                format!("caption word {index} ('{}') {problem}", word.text),
            );
        }
        previous_end = previous_end.max(word.end);
    }
}

/// What is wrong with `word`, given where the previous word ended and the item's duration.
fn word_problem(word: &CaptionWord, previous_end: Flicks, duration: Flicks) -> Option<String> {
    if word.text.trim().is_empty() {
        return Some("is empty; remove the word or give it text".to_owned());
    }
    if word.text.contains(['\n', '\r']) {
        return Some("contains a line break; split it into words".to_owned());
    }
    if word.start < previous_end {
        return Some(format!(
            "starts at {} before the previous word ends ({}); words are in speaking order and \
             never overlap",
            seconds(word.start),
            seconds(previous_end)
        ));
    }
    if word.end < word.start {
        return Some(format!(
            "ends at {} before it starts ({})",
            seconds(word.end),
            seconds(word.start)
        ));
    }
    if word.end > duration {
        return Some(format!(
            "ends at {} after its caption ({}); word times are relative to the caption's start",
            seconds(word.end),
            seconds(duration)
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::{CaptionStyleId, ItemKind, ViolationCode, test_support::millis, validate_project};

    fn codes_after(change: impl FnOnce(&mut crate::Project)) -> Vec<ViolationCode> {
        let mut project = crate::test_support::example_project();
        project.sequences[0].tracks[0].transitions.clear();
        change(&mut project);
        validate_project(&project)
            .into_iter()
            .map(|violation| violation.code)
            .collect()
    }

    fn with_caption(project: &mut crate::Project, change: impl FnOnce(&mut crate::CaptionItem)) {
        let caption = project.sequences[0]
            .tracks
            .iter_mut()
            .flat_map(|track| &mut track.items)
            .find_map(|item| match &mut item.kind {
                ItemKind::Caption(caption) => Some(caption),
                _ => None,
            })
            .unwrap();
        change(caption);
    }

    #[test]
    fn the_example_captions_are_valid() {
        assert_eq!(codes_after(|_| {}), []);
    }

    #[test]
    fn reports_unknown_caption_styles() {
        let codes = codes_after(|project| {
            with_caption(project, |caption| {
                caption.style_id = CaptionStyleId::from_test_index(99);
            });
        });
        assert_eq!(codes, [ViolationCode::UnknownReference]);
    }

    #[test]
    fn reports_words_out_of_order_outside_or_empty() {
        let overlapping = codes_after(|project| {
            with_caption(project, |caption| caption.words[1].start = millis(100));
        });
        assert_eq!(overlapping, [ViolationCode::InvalidCaptionWords]);
        let outside = codes_after(|project| {
            with_caption(project, |caption| {
                let last = caption.words.len() - 1;
                caption.words[last].end = millis(60_000);
            });
        });
        assert_eq!(outside, [ViolationCode::InvalidCaptionWords]);
        let empty = codes_after(|project| {
            with_caption(project, |caption| caption.words[0].text = " ".to_owned());
        });
        assert_eq!(empty, [ViolationCode::InvalidCaptionWords]);
    }

    #[test]
    fn reports_caption_style_values_out_of_range() {
        let codes = codes_after(|project| project.caption_styles[0].max_chars_per_line = 2);
        assert_eq!(codes, [ViolationCode::ValueOutOfRange]);
    }
}
