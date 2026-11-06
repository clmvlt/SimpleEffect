//! [`AddItem`], [`RemoveItem`] and [`RestoreItem`]: the primitive item operations.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    Edit, Effect,
    checks::check_revision,
    lookup::{find_item, find_track},
    placement::{insert_new_item, place_item, remove_item},
};
use crate::{DomainError, Item, ItemId, Project, TrackId};

/// Puts a new item on a track. Inverse: [`RemoveItem`].
///
/// The item is stored as given, revision included: new items carry revision 0, which the commit
/// bumps to 1 (an item restored by an undo keeps its old revision, bumped by the commit).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AddItem {
    /// The track receiving the item.
    pub track: TrackId,
    /// The item, with an id unused in the project.
    pub item: Item,
}

impl Edit for AddItem {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let target = find_track(project, self.track)?;
        insert_new_item(project, target, self.item.clone(), effect)
    }
}

/// Removes an item. Inverse: [`AddItem`] of the removed item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RemoveItem {
    /// The item removed.
    pub item: ItemId,
    /// Revision the caller read, if it wants the removal refused when the item changed since.
    pub expected_revision: Option<u64>,
}

impl Edit for RemoveItem {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let location = find_item(project, self.item)?;
        check_revision(location.get(project), self.expected_revision)?;
        remove_item(project, location, effect)
    }
}

/// Puts an existing item back to a recorded state, on the given track (moving it there if it is
/// elsewhere). Its current revision is kept. Inverse: [`RestoreItem`] of the state it replaced.
///
/// This is the inverse of every edit of one item's fields (move, trim, update, keyframes,
/// links), so undo and changeset reverts restore exactly what was there.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RestoreItem {
    /// The track the item ends up on.
    pub track: TrackId,
    /// The state to restore; its id names the item.
    pub item: Item,
}

impl Edit for RestoreItem {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let location = find_item(project, self.item.id)?;
        let target = find_track(project, self.track)?;
        place_item(project, location, target, self.item.clone(), effect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DomainError, ItemKind, Operation,
        operation::testing::{
            V1, V2, apply_err, apply_ok, example, item_ids, items_of, lock, shape_item,
        },
        test_support::range_ms,
    };

    #[test]
    fn adds_an_item_in_start_order_and_removes_it_back() {
        let item = shape_item(100, range_ms(14_000, 1000));
        let applied = apply_ok(&example(), AddItem { track: V1, item });
        assert_eq!(
            item_ids(&applied.project, V1),
            [1, 2, 100].map(ItemId::from_test_index)
        );
        assert_eq!(
            applied.inverse,
            [Operation::from(RemoveItem {
                item: ItemId::from_test_index(100),
                expected_revision: None
            })]
        );
        assert!(
            applied
                .touched
                .items
                .contains(&ItemId::from_test_index(100))
        );
    }

    #[test]
    fn refuses_overlaps_reused_ids_and_locked_tracks() {
        let overlapping = shape_item(100, range_ms(9000, 2000));
        let error = apply_err(
            &example(),
            AddItem {
                track: V1,
                item: overlapping,
            },
        );
        assert!(matches!(error, DomainError::Overlap { .. }), "{error}");
        let reused = shape_item(3, range_ms(20_000, 1000));
        let error = apply_err(
            &example(),
            AddItem {
                track: V1,
                item: reused,
            },
        );
        assert!(matches!(
            error,
            DomainError::IdAlreadyUsed { kind: "item", .. }
        ));
        let error = apply_err(
            &lock(example(), V1),
            AddItem {
                track: V1,
                item: shape_item(100, range_ms(20_000, 1000)),
            },
        );
        assert!(matches!(error, DomainError::TrackLocked { .. }));
    }

    #[test]
    fn removal_checks_the_expected_revision() {
        let stale = RemoveItem {
            item: ItemId::from_test_index(3),
            expected_revision: Some(7),
        };
        assert_eq!(
            apply_err(&example(), stale),
            DomainError::RevisionConflict {
                item: ItemId::from_test_index(3),
                expected: 7,
                actual: 1
            }
        );
        let fresh = RemoveItem {
            item: ItemId::from_test_index(3),
            expected_revision: Some(1),
        };
        let applied = apply_ok(&example(), fresh);
        assert_eq!(item_ids(&applied.project, V2), [ItemId::from_test_index(4)]);
    }

    #[test]
    fn restoring_keeps_the_current_revision_and_can_change_track() {
        let mut project = example();
        items_of(&mut project, V2)[0].revision = 5;
        let mut state = items_of(&mut project, V2)[0].clone();
        state.revision = 1;
        state.timeline_range = range_ms(20_000, 3000);
        state.kind = shape_item(0, range_ms(0, 1)).kind;
        let applied = apply_ok(
            &project,
            RestoreItem {
                track: V1,
                item: state,
            },
        );
        let restored = applied.project.sequences[0].tracks[0].items[2].clone();
        assert_eq!(restored.id, ItemId::from_test_index(3));
        assert_eq!(restored.revision, 5);
        assert!(matches!(restored.kind, ItemKind::Shape(_)));
    }
}
