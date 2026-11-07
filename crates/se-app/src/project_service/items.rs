//! Item edits of open projects (prompt 11, ADR-0021): moves, copies, pastes, deletes (with or
//! without ripple), links; dropped media in `placed_media`, trims and gaps in `trims`. Each is one user transaction planned with
//! [`EditPlan`] against the current document, so one undo step.

use se_domain::{
    Item, ItemId, LinkGroupId, LinkItems, ProjectId, RemoveItem, SequenceId, UnlinkItems,
};
use se_protocol::{ItemPlacement, PastedItem, PlacementMode};

use super::ProjectService;
use crate::{
    AppError,
    edit_plan::{
        EditPlan, copies_of, distinct, find_item, lift, place, ripple_delete, sequence_of,
        starting_at,
    },
};

/// Labels of the item transactions (undo menu). Stable identifiers: the renderer translates them
/// (`apps/desktop/src/app/editor/history-label.ts`).
pub mod labels {
    /// Media was dropped on a sequence.
    pub const ADD_CLIPS: &str = "Add clips";
    /// Items were moved.
    pub const MOVE: &str = "Move items";
    /// Items were duplicated.
    pub const DUPLICATE: &str = "Duplicate items";
    /// Clipboard items were pasted.
    pub const PASTE: &str = "Paste items";
    /// Items were deleted, leaving gaps.
    pub const DELETE: &str = "Delete items";
    /// Items were deleted and their time closed.
    pub const RIPPLE_DELETE: &str = "Ripple delete";
    /// Items were linked.
    pub const LINK: &str = "Link items";
    /// Items were unlinked.
    pub const UNLINK: &str = "Unlink items";
    /// Item edges were moved.
    pub const TRIM: &str = "Trim items";
    /// Item edges were moved, shifting what follows.
    pub const RIPPLE_TRIM: &str = "Ripple trim";
    /// Items were cut in two.
    pub const SPLIT: &str = "Split items";
    /// A gap was closed.
    pub const CLOSE_GAP: &str = "Close gap";
    /// A gap was lengthened or shortened.
    pub const RESIZE_GAP: &str = "Resize gap";
    /// Fades or their shapes changed.
    pub const CHANGE_FADES: &str = "Change fades";
    /// Other content of items changed (speed, text…).
    pub const EDIT_ITEMS: &str = "Edit items";
    /// A clip's volume (static or keyframed) changed.
    pub const CHANGE_VOLUME: &str = "Change volume";
    /// The keyframes of another animatable property changed.
    pub const CHANGE_KEYFRAMES: &str = "Change keyframes";
    /// Items were moved, scaled, rotated or re-anchored in the frame (gizmo, inspector).
    pub const TRANSFORM: &str = "Transform items";
    /// A text was added.
    pub const ADD_TEXT: &str = "Add text";
    /// A shape was added.
    pub const ADD_SHAPE: &str = "Add shape";
    /// An image was added at the playhead.
    pub const ADD_IMAGE: &str = "Add image";
    /// An animation preset was applied.
    pub const APPLY_ANIMATION: &str = "Apply animation";
    /// The reframe mode or focus of clips changed (inspector « Cadrage », viewer reframing).
    pub const REFRAME: &str = "Reframe items";
}

impl ProjectService {
    /// Moves items to new tracks and starts in one sequence, with `mode` for what they land on.
    ///
    /// # Errors
    /// Not open, nothing listed or an item twice, unknown items or tracks, several sequences,
    /// refusals of the document, storage errors.
    pub async fn move_items(
        &self,
        project_id: ProjectId,
        moves: &[ItemPlacement],
        mode: PlacementMode,
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let mut plan = EditPlan::new(&view.project, self.new_ids());
        distinct(moves.iter().map(|placement| placement.item_id))?;
        let sequence = placements_sequence(&mut plan, moves)?;
        let item_ids: Vec<ItemId> = moves.iter().map(|placement| placement.item_id).collect();
        let lifted = lift(&mut plan, &item_ids)?;
        let placed = lifted
            .into_iter()
            .zip(moves)
            .map(|((_, item), placement)| {
                Ok((placement.track_id, starting_at(item, placement.start)?))
            })
            .collect::<Result<Vec<_>, AppError>>()?;
        place(&mut plan, sequence, placed, mode)?;
        self.commit_plan(&handle, labels::MOVE, plan).await
    }

    /// Copies items of one sequence to new places, with `mode` for what the copies land on.
    /// Returns the copies' ids in the order of `copies`.
    ///
    /// # Errors
    /// Like [`Self::move_items`].
    pub async fn duplicate_items(
        &self,
        project_id: ProjectId,
        copies: &[ItemPlacement],
        mode: PlacementMode,
    ) -> Result<Vec<ItemId>, AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let mut plan = EditPlan::new(&view.project, self.new_ids());
        distinct(copies.iter().map(|placement| placement.item_id))?;
        let sequence = placements_sequence(&mut plan, copies)?;
        let project = plan.project()?;
        let originals = copies
            .iter()
            .map(|placement| {
                let item = find_item(project, placement.item_id)?.item.clone();
                starting_at(item, placement.start)
            })
            .collect::<Result<Vec<_>, AppError>>()?;
        let tracks = copies.iter().map(|placement| placement.track_id);
        let new_items = copies_of(originals, plan.ids())?;
        let new_ids = new_items.iter().map(|item| item.id).collect();
        place(&mut plan, sequence, tracks.zip(new_items).collect(), mode)?;
        self.commit_plan(&handle, labels::DUPLICATE, plan).await?;
        Ok(new_ids)
    }

    /// Adds copies of clipboard items (given by content, already at their new starts) to tracks
    /// of one sequence. Returns the copies' ids in the order of `items`.
    ///
    /// # Errors
    /// Not open, nothing listed, unknown or locked tracks, several sequences, unknown assets,
    /// refusals of the document, storage errors.
    pub async fn paste_items(
        &self,
        project_id: ProjectId,
        items: &[PastedItem],
        mode: PlacementMode,
    ) -> Result<Vec<ItemId>, AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let mut plan = EditPlan::new(&view.project, self.new_ids());
        let tracks = items.iter().map(|pasted| pasted.track_id);
        let sequence = sequence_of(plan.project()?, [], tracks)?;
        let contents: Vec<Item> = items.iter().map(|pasted| pasted.item.clone()).collect();
        let new_items = copies_of(contents, plan.ids())?;
        let new_ids = new_items.iter().map(|item| item.id).collect();
        let tracks = items.iter().map(|pasted| pasted.track_id);
        place(&mut plan, sequence, tracks.zip(new_items).collect(), mode)?;
        self.commit_plan(&handle, labels::PASTE, plan).await?;
        Ok(new_ids)
    }

    /// Deletes items of one sequence; with `ripple`, closes the time they took on their tracks
    /// and on the tracks of items linked to what follows.
    ///
    /// # Errors
    /// Not open, nothing listed or an item twice, unknown items, several sequences,
    /// [`AppError::RippleBlocked`], locked tracks, storage errors.
    pub async fn delete_items(
        &self,
        project_id: ProjectId,
        item_ids: &[ItemId],
        ripple: bool,
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let mut plan = EditPlan::new(&view.project, self.new_ids());
        let items = distinct(item_ids.iter().copied())?;
        let sequence = sequence_of(plan.project()?, items.iter().copied(), [])?;
        if ripple {
            ripple_delete(&mut plan, sequence, &items)?;
        } else {
            for &item in &items {
                plan.push(RemoveItem {
                    item,
                    expected_revision: None,
                });
            }
        }
        let label = if ripple {
            labels::RIPPLE_DELETE
        } else {
            labels::DELETE
        };
        self.commit_plan(&handle, label, plan).await
    }

    /// Puts at least two items of one sequence in a new link group (they then select and move
    /// together). Returns the group.
    ///
    /// # Errors
    /// Not open, fewer than two items, unknown items, several sequences, storage errors.
    pub async fn link_items(
        &self,
        project_id: ProjectId,
        item_ids: &[ItemId],
    ) -> Result<LinkGroupId, AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let mut plan = EditPlan::new(&view.project, self.new_ids());
        let items = distinct(item_ids.iter().copied())?;
        if items.len() < 2 {
            return Err(AppError::InvalidEdit {
                reason: "linking needs at least two items: select the picture and the sound to \
                         link"
                    .to_owned(),
            });
        }
        sequence_of(plan.project()?, items.iter().copied(), [])?;
        let group = plan.ids().link_group()?;
        plan.push(LinkItems {
            items: items.into_iter().collect(),
            group,
        });
        self.commit_plan(&handle, labels::LINK, plan).await?;
        Ok(group)
    }

    /// Takes items out of their link groups.
    ///
    /// # Errors
    /// Not open, nothing listed or an item twice, unknown items, storage errors.
    pub async fn unlink_items(
        &self,
        project_id: ProjectId,
        item_ids: &[ItemId],
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let mut plan = EditPlan::new(&view.project, self.new_ids());
        let items = distinct(item_ids.iter().copied())?;
        plan.push(UnlinkItems {
            items: items.into_iter().collect(),
        });
        self.commit_plan(&handle, labels::UNLINK, plan).await
    }
}

/// The one sequence holding the items and target tracks of `placements`.
fn placements_sequence(
    plan: &mut EditPlan<'_>,
    placements: &[ItemPlacement],
) -> Result<SequenceId, AppError> {
    sequence_of(
        plan.project()?,
        placements.iter().map(|placement| placement.item_id),
        placements.iter().map(|placement| placement.track_id),
    )
}
