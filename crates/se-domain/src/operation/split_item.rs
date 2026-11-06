//! [`SplitItem`]: cuts an item in two.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    AddItem, Edit, Effect, RestoreItem,
    checks::{check_revision, ensure_item_id_unused},
    effect::run_steps,
    lookup::find_item,
    reshape::cut_pieces,
};
use crate::{DomainError, Flicks, ItemId, Project};

/// Cuts an item at `at`: the item keeps `[start, at)` and its fade-in; a new item `new_item_id`
/// takes `[at, end)` and the fade-out, with the same content, origin and source continuity
/// (keyframes stay at their instants). The new item is in no link group: to split a linked pair,
/// split each item and link the new parts. Inverse: remove the new part, restore the item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SplitItem {
    /// The item cut.
    pub item: ItemId,
    /// The cut instant, strictly between the item's start and end.
    pub at: Flicks,
    /// Id of the new right part (unused in the project).
    pub new_item_id: ItemId,
    /// Revision the caller read, if it wants the split refused when the item changed since.
    pub expected_revision: Option<u64>,
}

impl Edit for SplitItem {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let location = find_item(project, self.item)?;
        let item = location.get(project);
        check_revision(item, self.expected_revision)?;
        let range = item.timeline_range;
        if self.at <= range.start() || self.at >= range.end() {
            return Err(DomainError::SplitOutsideItem {
                item: self.item,
                at: self.at,
                start: range.start(),
                end: range.end(),
            });
        }
        ensure_item_id_unused(project, self.new_item_id)?;
        let (left, right) = cut_pieces(item, self.at, self.at, self.at, self.new_item_id)?;
        let track = location.track.get(project).id;
        let steps = vec![
            RestoreItem { track, item: left }.into(),
            AddItem { track, item: right }.into(),
        ];
        run_steps(project, &steps, effect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Animatable, DomainError, ItemKind,
        operation::testing::{V1, V2, apply_err, apply_ok, clip, example, find, item_ids, lock},
        test_support::{millis, range_ms},
    };

    fn split(item: u64, at_ms: i64) -> SplitItem {
        SplitItem {
            item: ItemId::from_test_index(item),
            at: millis(at_ms),
            new_item_id: ItemId::from_test_index(100),
            expected_revision: None,
        }
    }

    #[test]
    fn the_right_part_continues_the_media() {
        // Clip 2: `[10 s, 14 s)` at 2× from 20 s.
        let applied = apply_ok(&example(), split(2, 11_000));
        let (left, right) = (find(&applied.project, 2), find(&applied.project, 100));
        assert_eq!(left.timeline_range, range_ms(10_000, 1000));
        assert_eq!(right.timeline_range, range_ms(11_000, 3000));
        assert_eq!(clip(right).source_start, millis(22_000));
        assert_eq!(right.origin, left.origin);
        assert_eq!(right.revision, 0, "the commit gives new items revision 1");
        assert_eq!(
            item_ids(&applied.project, V1),
            [1, 2, 100].map(ItemId::from_test_index)
        );
    }

    #[test]
    fn fades_stay_on_the_outer_edges_and_links_on_the_left_part() {
        // Clip 1 `[0 s, 10 s)` fades in for 0.5 s and is linked to clip 8.
        let applied = apply_ok(&example(), split(1, 300));
        let (left, right) = (find(&applied.project, 1), find(&applied.project, 100));
        assert_eq!(clip(left).fade_in, millis(300));
        assert_eq!(clip(right).fade_in, Flicks::ZERO);
        assert!(left.link_group.is_some());
        assert_eq!(right.link_group, None);
    }

    #[test]
    fn keyframes_are_shared_out_between_the_parts() {
        // Text 3 `[1 s, 4 s)`: opacity keyframes at 0 and 0.5 s.
        let applied = apply_ok(&example(), split(3, 1250));
        let opacity = |index| match &find(&applied.project, index).kind {
            ItemKind::Text(text) => text.transform.opacity.clone(),
            _ => unreachable!(),
        };
        assert!(matches!(opacity(3), Animatable::Keyframed { keyframes } if keyframes.len() == 1));
        assert!(
            matches!(opacity(100), Animatable::Keyframed { keyframes } if keyframes.len() == 1)
        );
    }

    #[test]
    fn the_cut_must_be_strictly_inside() {
        for at_ms in [1000, 4000, 500, 5000] {
            assert!(matches!(
                apply_err(&example(), split(3, at_ms)),
                DomainError::SplitOutsideItem { .. }
            ));
        }
    }

    #[test]
    fn refuses_a_used_id_and_a_locked_track() {
        let mut reused = split(3, 2000);
        reused.new_item_id = ItemId::from_test_index(4);
        assert!(matches!(
            apply_err(&example(), reused),
            DomainError::IdAlreadyUsed { .. }
        ));
        assert!(matches!(
            apply_err(&lock(example(), V2), split(3, 2000)),
            DomainError::TrackLocked { .. }
        ));
    }
}
