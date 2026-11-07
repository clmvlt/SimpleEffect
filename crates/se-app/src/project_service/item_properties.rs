//! Content edits of items (prompt 14): typed patches (fades and their shapes, speed, reframe
//! modes…) and the keyframes of animatable properties (a clip's volume curve). Each command is
//! one user transaction, so one undo step.

use se_domain::{
    ItemId, ItemPatch, MediaClipPatch, ProjectId, PropertyAnimation, SetKeyframes, UpdateItem,
};
use se_protocol::{ItemAnimation, ItemUpdate};

use super::{ProjectService, items::labels};
use crate::{
    AppError,
    edit_plan::{EditPlan, distinct},
};

impl ProjectService {
    /// Applies one typed patch per item, then sets `animations` (after the patches, so a clip
    /// switched to a fill mode can get its focus, or a scale that keeps its size, in the same
    /// step). Planned like the item edits, so a speed or slip change that breaks a transition
    /// removes it, and transitions that still fit are kept.
    ///
    /// # Errors
    /// Not open, nothing listed or an item twice, unknown items, a patch of another kind than its
    /// item, a property an item does not have, values the document refuses (fades longer than
    /// the item…), storage errors.
    pub async fn update_items(
        &self,
        project_id: ProjectId,
        updates: &[ItemUpdate],
        animations: &[ItemAnimation],
    ) -> Result<(), AppError> {
        distinct(updates.iter().map(|update| update.item_id))?;
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let mut plan = EditPlan::new(&view.project, self.new_ids());
        for update in updates {
            plan.push(UpdateItem {
                item: update.item_id,
                patch: update.patch.clone(),
                expected_revision: None,
            });
        }
        for change in animations {
            plan.push(SetKeyframes {
                item: change.item_id,
                animation: change.animation.clone(),
                expected_revision: None,
            });
        }
        plan.project()?;
        self.commit_plan(&handle, update_label(updates), plan).await
    }

    /// Sets the keyframes (or the static value) of one animatable property of an item.
    ///
    /// # Errors
    /// Not open, unknown item, a property the item does not have, keyframes the document refuses
    /// (outside the item, unsorted, a volume out of range…), storage errors.
    pub async fn set_item_animation(
        &self,
        project_id: ProjectId,
        item_id: ItemId,
        animation: PropertyAnimation,
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id).await?;
        let label = match animation {
            PropertyAnimation::VolumeDb { .. } => labels::CHANGE_VOLUME,
            PropertyAnimation::ReframeFocus { .. } => labels::REFRAME,
            _ => labels::CHANGE_KEYFRAMES,
        };
        let operation = SetKeyframes {
            item: item_id,
            animation,
            expected_revision: None,
        };
        self.commit_user_edit(&handle, label, vec![operation.into()])
            .await
    }
}

/// « Change fades » when every patch only touches fades of media clips, « Reframe items » when
/// every patch only changes the reframe mode of clips, else « Edit items ».
fn update_label(updates: &[ItemUpdate]) -> &'static str {
    let only_reframes = !updates.is_empty()
        && updates.iter().all(|update| match &update.patch {
            ItemPatch::MediaClip(patch) => {
                patch.reframe.is_some()
                    && *patch
                        == MediaClipPatch {
                            reframe: patch.reframe.clone(),
                            ..MediaClipPatch::default()
                        }
            }
            _ => false,
        });
    if only_reframes {
        return labels::REFRAME;
    }
    let only_fades = updates.iter().all(|update| match &update.patch {
        ItemPatch::MediaClip(patch) => {
            let fades = MediaClipPatch {
                fade_in: patch.fade_in,
                fade_out: patch.fade_out,
                fade_in_shape: patch.fade_in_shape,
                fade_out_shape: patch.fade_out_shape,
                ..MediaClipPatch::default()
            };
            *patch == fades
        }
        _ => false,
    });
    if only_fades {
        labels::CHANGE_FADES
    } else {
        labels::EDIT_ITEMS
    }
}

#[cfg(test)]
mod tests {
    use se_domain::{FadeShape, ShapePatch};

    use super::*;

    fn update(patch: ItemPatch) -> ItemUpdate {
        ItemUpdate {
            item_id: ItemId::from_test_index(1),
            patch,
        }
    }

    #[test]
    fn fade_patches_are_labelled_as_fades() {
        let fades = update(ItemPatch::MediaClip(MediaClipPatch {
            fade_out_shape: Some(FadeShape::ConstantPower),
            ..MediaClipPatch::default()
        }));
        assert_eq!(
            update_label(std::slice::from_ref(&fades)),
            labels::CHANGE_FADES
        );
        let speed = update(ItemPatch::MediaClip(MediaClipPatch {
            speed: Some(2.0),
            ..MediaClipPatch::default()
        }));
        assert_eq!(update_label(&[fades, speed]), labels::EDIT_ITEMS);
        let shape = update(ItemPatch::Shape(ShapePatch::default()));
        assert_eq!(update_label(&[shape]), labels::EDIT_ITEMS);
        let reframe = update(ItemPatch::MediaClip(MediaClipPatch {
            reframe: Some(se_domain::Reframe::Custom),
            ..MediaClipPatch::default()
        }));
        assert_eq!(update_label(&[reframe]), labels::REFRAME);
    }
}
