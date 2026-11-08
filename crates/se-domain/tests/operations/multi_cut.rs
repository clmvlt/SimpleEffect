//! `RippleDeleteRanges` (prompt 22) against single `RippleDeleteRange`s: after random edits, a
//! multi-range ripple delete gives exactly the project of the same cuts applied one by one, last
//! range first.

use std::collections::BTreeMap;

use proptest::prelude::*;
use se_domain::{
    Item, Operation, Project, RippleDeleteRange, RippleDeleteRanges, TimeRange, apply, apply_all,
    test_support::example_project,
};

use super::{
    CASES,
    generators::{Generator, seed},
};

fn strictly_inside(cut: &TimeRange, range: TimeRange) -> bool {
    cut.start() > range.start() && cut.end() < range.end()
}

/// The single ripple deletes doing what `together` does, each item's k-th new part getting the
/// k-th id of `together.split_ids`.
fn one_by_one(project: &Project, together: &RippleDeleteRanges) -> Vec<Operation> {
    let items: Vec<&Item> = project.items().collect();
    let mut operations = Vec::new();
    for (index, range) in together.ranges.iter().enumerate().rev() {
        let mut split_ids = BTreeMap::new();
        for item in items
            .iter()
            .filter(|item| strictly_inside(range, item.timeline_range))
        {
            let earlier = together.ranges[..index]
                .iter()
                .filter(|cut| strictly_inside(cut, item.timeline_range))
                .count();
            if let Some(new) = together
                .split_ids
                .get(&item.id)
                .and_then(|ids| ids.get(earlier))
            {
                split_ids.insert(item.id, *new);
            }
        }
        operations.push(Operation::from(RippleDeleteRange {
            sequence: together.sequence,
            range: *range,
            tracks: together.tracks.clone(),
            split_ids,
        }));
    }
    operations
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn multi_range_ripple_deletes_equal_single_ones(
        seeds in prop::collection::vec(seed(), 0..10),
        cut_seed in seed(),
    ) {
        let mut generator = Generator::new();
        let mut project = example_project();
        for seed in seeds {
            let Some(operation) = generator.operation(&project, seed) else { continue };
            if let Ok(applied) = apply(&project, &operation) {
                project = applied.project;
            }
        }
        let cut = generator.multi_cut(&project, cut_seed);
        let Some(together) = cut else { return Ok(()) };
        let at_once = apply(&project, &Operation::from(together.clone()));
        let sequential = apply_all(&project, &one_by_one(&project, &together));
        match (at_once, sequential) {
            (Ok(at_once), Ok(sequential)) => prop_assert_eq!(at_once.project, sequential.project),
            (Err(_), Err(_)) => {}
            (at_once, sequential) => prop_assert!(
                false,
                "one succeeded and the other did not: {:?} / {:?}",
                at_once.err(),
                sequential.err()
            ),
        }
    }
}
