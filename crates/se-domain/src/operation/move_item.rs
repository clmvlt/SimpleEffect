//! [`MoveItem`]: moves an item in time and/or to another track.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    Edit, Effect,
    checks::check_revision,
    lookup::{find_item, find_track},
    placement::place_item,
};
use crate::{DomainError, Flicks, Item, ItemId, Project, TimeRange, TrackId};

/// Moves an item so that it starts at `start` on `track` (its own track or another one, of any
/// sequence), keeping its duration and content. Refused if it would overlap another item.
/// Linked items are not moved along: move each of them in the same transaction.
/// Inverse: [`super::RestoreItem`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MoveItem {
    /// The item moved.
    pub item: ItemId,
    /// The track it ends up on.
    pub track: TrackId,
    /// Its new start in the sequence (≥ 0).
    pub start: Flicks,
    /// Revision the caller read, if it wants the move refused when the item changed since.
    pub expected_revision: Option<u64>,
}

impl Edit for MoveItem {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let location = find_item(project, self.item)?;
        let item = location.get(project);
        check_revision(item, self.expected_revision)?;
        let target = find_track(project, self.track)?;
        let timeline_range = TimeRange::new(self.start, item.timeline_range.duration())
            .map_err(|_| DomainError::TimeOutOfRange { item: self.item })?;
        let moved = Item {
            timeline_range,
            ..item.clone()
        };
        place_item(project, location, target, moved, effect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ViolationCode,
        operation::testing::{A1, V1, V2, V3, apply_err, apply_ok, example, find, item_ids, lock},
        test_support::{millis, range_ms},
    };

    fn move_to(item: u64, track: TrackId, start_ms: i64) -> MoveItem {
        MoveItem {
            item: ItemId::from_test_index(item),
            track,
            start: millis(start_ms),
            expected_revision: None,
        }
    }

    #[test]
    fn moves_within_a_track_and_keeps_the_items_sorted() {
        let applied = apply_ok(&example(), move_to(3, V2, 8000));
        assert_eq!(
            find(&applied.project, 3).timeline_range,
            range_ms(8000, 3000)
        );
        assert_eq!(
            item_ids(&applied.project, V2),
            [4, 3].map(ItemId::from_test_index)
        );
    }

    #[test]
    fn moves_to_another_track() {
        let applied = apply_ok(&example(), move_to(4, V1, 20_000));
        assert_eq!(
            item_ids(&applied.project, V1),
            [1, 2, 4].map(ItemId::from_test_index)
        );
        assert_eq!(item_ids(&applied.project, V2), [ItemId::from_test_index(3)]);
    }

    #[test]
    fn touching_the_neighbour_is_fine_overlapping_is_not() {
        apply_ok(&example(), move_to(4, V2, 4000));
        let error = apply_err(&example(), move_to(4, V2, 3999));
        let DomainError::Overlap {
            other, start, end, ..
        } = error
        else {
            panic!("expected an overlap, got {error}");
        };
        assert_eq!(other, ItemId::from_test_index(3));
        assert_eq!((start, end), (millis(3999), millis(4000)));
    }

    #[test]
    fn locked_tracks_neither_give_nor_receive_items() {
        let error = apply_err(&lock(example(), V2), move_to(4, V3, 20_000));
        assert!(matches!(error, DomainError::TrackLocked { .. }));
        let error = apply_err(&lock(example(), V3), move_to(4, V3, 20_000));
        assert!(matches!(error, DomainError::TrackLocked { .. }));
    }

    #[test]
    fn invalid_destinations_are_reported_by_validation() {
        let error = apply_err(&example(), move_to(3, A1, 20_000));
        let DomainError::InvalidDocument { violations } = error else {
            panic!("expected a validation error, got {error}");
        };
        assert_eq!(violations[0].code, ViolationCode::ItemNotAllowedOnTrack);
        let error = apply_err(&example(), move_to(3, V2, -500));
        assert!(matches!(error, DomainError::InvalidDocument { .. }));
    }

    #[test]
    fn checks_the_expected_revision() {
        let mut stale = move_to(3, V2, 8000);
        stale.expected_revision = Some(0);
        assert!(matches!(
            apply_err(&example(), stale),
            DomainError::RevisionConflict { .. }
        ));
    }
}
