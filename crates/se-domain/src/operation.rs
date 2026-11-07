//! Operations: every change of a [`Project`] is one of the intent-level edits of [`Operation`]
//! (`docs/architecture.md` §5.4, ADR-0005).
//!
//! [`apply`] is pure and deterministic. Ids it needs (the new part of a split item…) are given in
//! the operation by the caller. It returns the edited project, the operations that undo the
//! edit and the entities it touched, or a [`DomainError`] and no change. Every result satisfies
//! [`validate_project`].
//!
//! Revisions are not bumped here: [`crate::commit`] bumps them once per transaction. An operation
//! keeps the revision of every item it modifies, so `apply(op)` followed by its inverse gives back
//! exactly the original project.
//!
//! One file per operation (or small coherent group); shared plumbing in `effect`, `lookup`,
//! `checks`, `placement` and `reshape`. Recipe to add one: `crates/se-domain/CLAUDE.md`.

mod add_remove_item;
mod animation;
mod apply_animation_preset;
mod asset_ops;
mod checks;
mod effect;
mod insert_gap;
mod item_patch;
mod link_items;
pub(crate) mod lookup;
mod marker_ops;
mod move_item;
mod placement;
mod project_ops;
mod reshape;
mod ripple_delete_range;
mod sequence_ops;
mod split_item;
#[cfg(test)]
pub(crate) mod testing;
mod touched;
mod track_ops;
mod transition_ops;
mod transition_upkeep;
mod trim_item;
mod update_item;

pub use add_remove_item::{AddItem, RemoveItem, RestoreItem};
pub use animation::{PropertyAnimation, SetKeyframes};
pub use apply_animation_preset::ApplyAnimationPreset;
pub use asset_ops::{AddAsset, AssetPatch, Assign, RemoveAsset, UpdateAsset};
pub use insert_gap::InsertGap;
pub use item_patch::{
    CaptionPatch, ImagePatch, ItemPatch, MediaClipPatch, ShapePatch, TemplatePatch, TextPatch,
};
pub use link_items::{LinkItems, UnlinkItems};
pub use marker_ops::{AddMarker, MarkerPatch, RemoveMarker, UpdateMarker};
pub use move_item::MoveItem;
pub use project_ops::RenameProject;
pub use ripple_delete_range::RippleDeleteRange;
pub use sequence_ops::{AddSequence, RemoveSequence, SequencePatch, UpdateSequence};
pub use split_item::SplitItem;
pub use touched::TouchedEntities;
pub use track_ops::{AddTrack, RemoveTrack, ReorderTracks, TrackPatch, UpdateTrack};
pub use transition_ops::{AddTransition, RemoveTransition, TransitionPatch, UpdateTransition};
pub use transition_upkeep::TransitionSnapshot;
pub use trim_item::{ItemEdge, TrimItem};
pub use update_item::UpdateItem;

pub(crate) use effect::{Edit, Effect};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{DomainError, Project, validate_project};

/// One intent-level edit of a project. Serialized with its fields next to `"kind"`
/// (`{"kind": "moveItem", "item": "…", "track": "…", "start": 0, "expectedRevision": null}`),
/// the format of the operation journal (prompt 07).
///
/// Operations that change one item accept an `expected_revision`: when set and different from
/// the item's revision, the edit fails with [`DomainError::RevisionConflict`].
///
/// TODO(prompt-24): operations on motion templates. TODO(prompt-25): on the brand kit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Operation {
    /// Puts a new item on a track.
    AddItem(AddItem),
    /// Removes an item.
    RemoveItem(RemoveItem),
    /// Puts an item back to a recorded state, possibly on another track (inverse of item edits).
    RestoreItem(RestoreItem),
    /// Moves an item in time and/or to another track.
    MoveItem(MoveItem),
    /// Moves one edge of an item, adjusting the media played.
    TrimItem(TrimItem),
    /// Cuts an item in two at an instant.
    SplitItem(SplitItem),
    /// Changes the content of an item (typed by item kind).
    UpdateItem(UpdateItem),
    /// Sets the keyframes (or static value) of one animatable property.
    SetKeyframes(SetKeyframes),
    /// Expands an animation preset into keyframes of an item.
    ApplyAnimationPreset(ApplyAnimationPreset),
    /// Puts items in a link group.
    LinkItems(LinkItems),
    /// Takes items out of their link group.
    UnlinkItems(UnlinkItems),
    /// Deletes a time range on tracks and closes the gap.
    RippleDeleteRange(RippleDeleteRange),
    /// Opens empty time on tracks, pushing later items right.
    InsertGap(InsertGap),
    /// Adds a track to a sequence.
    AddTrack(AddTrack),
    /// Removes a track and its items.
    RemoveTrack(RemoveTrack),
    /// Changes the header of a track (name, mute, lock…).
    UpdateTrack(UpdateTrack),
    /// Changes the order of the tracks of a sequence.
    ReorderTracks(ReorderTracks),
    /// Adds a sequence.
    AddSequence(AddSequence),
    /// Removes a sequence and everything in it.
    RemoveSequence(RemoveSequence),
    /// Renames a sequence or changes its format.
    UpdateSequence(UpdateSequence),
    /// Adds a marker to a sequence.
    AddMarker(AddMarker),
    /// Removes a marker.
    RemoveMarker(RemoveMarker),
    /// Moves, relabels or recolours a marker.
    UpdateMarker(UpdateMarker),
    /// Adds an asset.
    AddAsset(AddAsset),
    /// Changes the facts known about an asset (path, probe, derived files…).
    UpdateAsset(UpdateAsset),
    /// Removes an asset that no item uses.
    RemoveAsset(RemoveAsset),
    /// Renames the project.
    RenameProject(RenameProject),
    /// Adds a cross-fade on the cut between two adjacent items of a track.
    AddTransition(AddTransition),
    /// Resizes or realigns a transition.
    UpdateTransition(UpdateTransition),
    /// Removes a transition.
    RemoveTransition(RemoveTransition),
}

/// Declares `From<Payload> for Operation` and the dispatch to each payload's [`Edit`] impl.
macro_rules! operation_payloads {
    ($($variant:ident),* $(,)?) => {
        $(
            impl From<$variant> for Operation {
                fn from(operation: $variant) -> Self {
                    Self::$variant(operation)
                }
            }
        )*

        impl Operation {
            /// The edit performed by this operation.
            fn as_edit(&self) -> &dyn Edit {
                match self {
                    $(Self::$variant(operation) => operation,)*
                }
            }
        }
    };
}

operation_payloads!(
    AddItem,
    RemoveItem,
    RestoreItem,
    MoveItem,
    TrimItem,
    SplitItem,
    UpdateItem,
    SetKeyframes,
    ApplyAnimationPreset,
    LinkItems,
    UnlinkItems,
    RippleDeleteRange,
    InsertGap,
    AddTrack,
    RemoveTrack,
    UpdateTrack,
    ReorderTracks,
    AddSequence,
    RemoveSequence,
    UpdateSequence,
    AddMarker,
    RemoveMarker,
    UpdateMarker,
    AddAsset,
    UpdateAsset,
    RemoveAsset,
    RenameProject,
    AddTransition,
    UpdateTransition,
    RemoveTransition,
);

/// The outcome of [`apply`].
#[derive(Debug, Clone, PartialEq)]
pub struct Applied {
    /// The edited project (valid; revisions unchanged).
    pub project: Project,
    /// Operations that undo the edit when applied in order to `project`.
    pub inverse: Vec<Operation>,
    /// Entities added, changed or removed.
    pub touched: TouchedEntities,
}

/// Applies `operation` to `project`.
///
/// # Errors
/// A [`DomainError`] saying why the edit is refused (unknown id, overlap, locked track, stale
/// revision…), or [`DomainError::InvalidDocument`] if the result would break an invariant of
/// [`validate_project`]. `project` must itself be valid.
pub fn apply(project: &Project, operation: &Operation) -> Result<Applied, DomainError> {
    apply_all(project, std::slice::from_ref(operation))
}

/// Applies `operations` in order to one copy of `project` and validates the result once, after
/// the last one: all or nothing. Each operation still checks its own preconditions (unknown ids,
/// overlaps, locked tracks…), but the document-wide [`validate_project`] runs once, so a
/// transaction of many operations costs one validation (it dominates with large sequences). The
/// returned inverse undoes all of them (the inverses of each, last operation first).
///
/// # Errors
/// The first operation refused, or [`DomainError::InvalidDocument`] if the result breaks an
/// invariant. `project` must itself be valid.
pub fn apply_all(project: &Project, operations: &[Operation]) -> Result<Applied, DomainError> {
    let mut draft = project.clone();
    let mut inverses = Vec::with_capacity(operations.len());
    let mut touched = TouchedEntities::default();
    for operation in operations {
        let mut effect = Effect::default();
        operation.as_edit().apply_to(&mut draft, &mut effect)?;
        touched.merge(effect.touched);
        inverses.push(effect.inverse);
    }
    let violations = validate_project(&draft);
    if !violations.is_empty() {
        return Err(DomainError::InvalidDocument { violations });
    }
    Ok(Applied {
        project: draft,
        inverse: inverses.into_iter().rev().flatten().collect(),
        touched,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ItemId, test_support::example_project};

    #[test]
    fn operations_are_tagged_by_kind_with_camel_case_fields() {
        let operation = Operation::from(RemoveItem {
            item: ItemId::from_test_index(1),
            expected_revision: Some(3),
        });
        let json = serde_json::to_string(&operation).unwrap();
        assert_eq!(
            json,
            r#"{"kind":"removeItem","item":"00000000-0001-7000-8000-000000000000","expectedRevision":3}"#
        );
        assert_eq!(serde_json::from_str::<Operation>(&json).unwrap(), operation);
    }

    #[test]
    fn a_failed_operation_changes_nothing() {
        let project = example_project();
        let operation = Operation::from(RemoveItem {
            item: ItemId::from_test_index(999),
            expected_revision: None,
        });
        assert_eq!(
            apply(&project, &operation),
            Err(DomainError::ItemNotFound {
                item: ItemId::from_test_index(999)
            })
        );
        assert_eq!(project, example_project());
    }
}
