//! Rules of a track as a whole: `OVERLAPPING_ITEMS`, `ITEMS_NOT_SORTED` and
//! `VALUE_OUT_OF_RANGE` for pan.

use super::{Path, Report, ViolationCode, report::seconds};
use crate::{Item, Track};

pub(super) fn check(track: &Track, path: &Path, report: &mut Report) {
    if !(-1.0..=1.0).contains(&track.pan) {
        report.push(
            ViolationCode::ValueOutOfRange,
            &path.field("pan"),
            format!(
                "pan must be between -1 (left) and 1 (right), got {}",
                track.pan
            ),
        );
    }
    check_overlaps(&track.items, &path.field("items"), report);
    check_order(&track.items, &path.field("items"), report);
}

/// Reports each item that starts before the item listed just before it. Operations keep items
/// sorted by start so that their position in the list is deterministic (exact inverses) and
/// time lookups can use binary search.
fn check_order(items: &[Item], items_path: &Path, report: &mut Report) {
    for pair in items.windows(2) {
        let (previous, item) = (&pair[0], &pair[1]);
        if item.timeline_range.start() < previous.timeline_range.start() {
            report.push(
                ViolationCode::ItemsNotSorted,
                &items_path.key(item.id),
                format!(
                    "item starts at {}, before item {} listed just before it at {}; list the \
                     items of a track by start time",
                    seconds(item.timeline_range.start()),
                    previous.id,
                    seconds(previous.timeline_range.start()),
                ),
            );
        }
    }
}

/// Reports each item that starts before an earlier-starting item of the track has ended.
///
/// Items are visited by start time while remembering the one that ends last, so an item
/// overlapping a long earlier item is found even if shorter items sit in between.
fn check_overlaps(items: &[Item], items_path: &Path, report: &mut Report) {
    let mut by_start: Vec<&Item> = items.iter().collect();
    by_start.sort_by_key(|item| (item.timeline_range.start(), item.timeline_range.end()));
    let mut latest_ending: Option<&Item> = None;
    for item in by_start {
        if let Some(previous) = latest_ending
            && previous.timeline_range.intersects(item.timeline_range)
        {
            report.push(
                ViolationCode::OverlappingItems,
                &items_path.key(item.id),
                format!(
                    "item starts at {} but item {} on the same track runs until {}; move or \
                     trim one of them, or put one on another track",
                    seconds(item.timeline_range.start()),
                    previous.id,
                    seconds(previous.timeline_range.end()),
                ),
            );
        }
        if latest_ending
            .is_none_or(|previous| item.timeline_range.end() > previous.timeline_range.end())
        {
            latest_ending = Some(item);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ViolationCode,
        test_support::{example_project, range_ms},
        validate_project,
    };

    #[test]
    fn reports_overlapping_items_on_one_track() {
        let mut project = example_project();
        let items = &mut project.sequences[0].tracks[0].items;
        items[1].timeline_range = range_ms(9000, 2000);
        let second_id = items[1].id;
        let violations = validate_project(&project);
        assert_eq!(violations.len(), 1, "{violations:#?}");
        assert_eq!(violations[0].code, ViolationCode::OverlappingItems);
        assert!(violations[0].path.ends_with(&format!("items[{second_id}]")));
        assert!(violations[0].message.contains("10.000s"));
    }

    #[test]
    fn finds_overlaps_hidden_behind_shorter_items() {
        let mut project = example_project();
        // V3: template [2 s, 5 s), shape [6 s, 8 s), caption [8 s, 10 s). Stretch the template
        // over both others: each of them overlaps it.
        project.sequences[0].tracks[2].items[0].timeline_range = range_ms(2000, 9000);
        let codes: Vec<_> = validate_project(&project)
            .into_iter()
            .map(|violation| violation.code)
            .collect();
        assert_eq!(codes, [ViolationCode::OverlappingItems; 2]);
    }

    #[test]
    fn touching_items_do_not_overlap() {
        let mut project = example_project();
        project.sequences[0].tracks[2].items[0].timeline_range = range_ms(2000, 4000);
        assert_eq!(validate_project(&project), []);
    }

    #[test]
    fn reports_items_out_of_start_order() {
        let mut project = example_project();
        project.sequences[0].tracks[1].items.swap(0, 1);
        let violations = validate_project(&project);
        assert_eq!(violations.len(), 1, "{violations:#?}");
        assert_eq!(violations[0].code, ViolationCode::ItemsNotSorted);
        assert!(
            violations[0]
                .path
                .ends_with("items[00000000-0003-7000-8000-000000000000]")
        );
    }

    #[test]
    fn reports_pan_out_of_range() {
        let mut project = example_project();
        project.sequences[0].tracks[3].pan = -1.5;
        let violations = validate_project(&project);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].code, ViolationCode::ValueOutOfRange);
        assert_eq!(violations[0].path.rsplit('.').next(), Some("pan"));
    }
}
