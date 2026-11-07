//! [`LinkItems`] and [`UnlinkItems`]: link groups of items.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Edit, Effect, lookup::find_item, placement::place_item};
use crate::{DomainError, Item, ItemId, LinkGroupId, Project};

/// Puts items in link group `group` (a new group, or an existing one to add items to it). Items
/// of a link group must be in the same sequence. Inverse: one [`super::RestoreItem`] per item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LinkItems {
    /// The items linked (at least one, each once).
    pub items: Vec<ItemId>,
    /// The group they join.
    pub group: LinkGroupId,
}

impl Edit for LinkItems {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        set_link_group(project, &self.items, Some(self.group), "linkItems", effect)
    }
}

/// Takes items out of their link group. Inverse: one [`super::RestoreItem`] per item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct UnlinkItems {
    /// The items unlinked (at least one, each once).
    pub items: Vec<ItemId>,
}

impl Edit for UnlinkItems {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        set_link_group(project, &self.items, None, "unlinkItems", effect)
    }
}

fn set_link_group(
    project: &mut Project,
    items: &[ItemId],
    group: Option<LinkGroupId>,
    operation: &'static str,
    effect: &mut Effect,
) -> Result<(), DomainError> {
    if items.is_empty() {
        return Err(DomainError::EmptyItemList { operation });
    }
    let mut seen = BTreeSet::new();
    for &id in items {
        if !seen.insert(id) {
            return Err(DomainError::DuplicateItem { item: id });
        }
        let location = find_item(project, id)?;
        let relinked = Item {
            link_group: group,
            ..location.get(project).clone()
        };
        place_item(project, location, location.track, relinked, effect)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ViolationCode,
        operation::testing::{A1, apply_err, apply_ok, example, find, lock},
    };

    fn ids(indices: &[u64]) -> Vec<ItemId> {
        indices
            .iter()
            .copied()
            .map(ItemId::from_test_index)
            .collect()
    }

    #[test]
    fn links_and_unlinks_items() {
        let group = LinkGroupId::from_test_index(9);
        let applied = apply_ok(
            &example(),
            LinkItems {
                items: ids(&[2, 9]),
                group,
            },
        );
        assert_eq!(find(&applied.project, 2).link_group, Some(group));
        assert_eq!(find(&applied.project, 9).link_group, Some(group));
        let applied = apply_ok(
            &example(),
            UnlinkItems {
                items: ids(&[1, 8]),
            },
        );
        assert_eq!(find(&applied.project, 1).link_group, None);
        assert_eq!(applied.inverse.len(), 2);
    }

    #[test]
    fn refuses_empty_or_repeated_lists_and_locked_tracks() {
        assert!(matches!(
            apply_err(&example(), UnlinkItems { items: Vec::new() }),
            DomainError::EmptyItemList {
                operation: "unlinkItems"
            }
        ));
        assert!(matches!(
            apply_err(
                &example(),
                UnlinkItems {
                    items: ids(&[1, 1])
                }
            ),
            DomainError::DuplicateItem { .. }
        ));
        assert!(matches!(
            apply_err(&lock(example(), A1), UnlinkItems { items: ids(&[8]) }),
            DomainError::TrackLocked { .. }
        ));
    }

    #[test]
    fn link_groups_stay_within_one_sequence() {
        let mut project = example();
        let mut other = project.sequences[0].clone();
        other.id = crate::SequenceId::from_test_index(2);
        other.markers.clear();
        other.tracks.truncate(1);
        other.tracks[0].transitions.clear();
        other.tracks[0].id = crate::TrackId::from_test_index(20);
        other.tracks[0].items.truncate(1);
        other.tracks[0].items[0].id = ItemId::from_test_index(20);
        other.tracks[0].items[0].link_group = None;
        project.sequences.push(other);
        let link = LinkItems {
            items: ids(&[20]),
            group: LinkGroupId::from_test_index(1),
        };
        let DomainError::InvalidDocument { violations } = apply_err(&project, link) else {
            panic!("expected a validation error");
        };
        assert_eq!(violations[0].code, ViolationCode::LinkGroupAcrossSequences);
    }
}
