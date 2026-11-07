//! Marker edits of open projects: add, change, remove. Each is one user transaction, so one undo
//! step.

use se_domain::{
    AddMarker, Flicks, Marker, MarkerColor, MarkerId, MarkerPatch, ProjectId, RemoveMarker,
    SequenceId, UpdateMarker,
};

use super::ProjectService;
use crate::{AppError, new_ids::NewIds};

/// Labels of the marker transactions (undo menu). Stable identifiers: the renderer translates
/// them (`apps/desktop/src/app/editor/history-label.ts`).
pub mod labels {
    /// A marker was added.
    pub const ADD: &str = "Add marker";
    /// A marker was removed.
    pub const REMOVE: &str = "Remove marker";
    /// A marker was moved.
    pub const MOVE: &str = "Move marker";
    /// The text of a marker changed.
    pub const RENAME: &str = "Rename marker";
    /// The colour of a marker changed.
    pub const RECOLOR: &str = "Change marker color";
    /// Several fields of a marker changed at once.
    pub const UPDATE: &str = "Update marker";
}

impl ProjectService {
    /// Adds a marker at `time` (appended to the sequence's markers); `color` defaults to green.
    ///
    /// # Errors
    /// Not open, unknown sequence, negative time, storage errors.
    pub async fn add_marker(
        &self,
        project_id: ProjectId,
        sequence_id: SequenceId,
        time: Flicks,
        label: String,
        color: Option<MarkerColor>,
    ) -> Result<MarkerId, AppError> {
        let handle = self.handle(project_id).await?;
        let marker = Marker {
            id: NewIds(self.deps.ids.as_ref()).marker()?,
            time,
            label,
            color: color.unwrap_or_default(),
        };
        let marker_id = marker.id;
        let operation = AddMarker {
            sequence: sequence_id,
            index: None,
            marker,
        };
        self.commit_user_edit(&handle, labels::ADD, vec![operation.into()])
            .await?;
        Ok(marker_id)
    }

    /// Moves, renames or recolours a marker.
    ///
    /// # Errors
    /// Not open, unknown marker, negative time, storage errors.
    pub async fn update_marker(
        &self,
        project_id: ProjectId,
        marker_id: MarkerId,
        patch: MarkerPatch,
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id).await?;
        let label = marker_patch_label(&patch);
        let operation = UpdateMarker {
            marker: marker_id,
            patch,
        };
        self.commit_user_edit(&handle, label, vec![operation.into()])
            .await
    }

    /// Removes a marker.
    ///
    /// # Errors
    /// Not open, unknown marker, storage errors.
    pub async fn remove_marker(
        &self,
        project_id: ProjectId,
        marker_id: MarkerId,
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id).await?;
        let operation = RemoveMarker { marker: marker_id };
        self.commit_user_edit(&handle, labels::REMOVE, vec![operation.into()])
            .await
    }
}

/// The undo label of a marker change: specific when one field changes.
fn marker_patch_label(patch: &MarkerPatch) -> &'static str {
    let changed = [
        patch.time.map(|_| labels::MOVE),
        patch.label.as_ref().map(|_| labels::RENAME),
        patch.color.map(|_| labels::RECOLOR),
    ];
    let mut labels = changed.into_iter().flatten();
    match (labels.next(), labels.next()) {
        (Some(single), None) => single,
        _ => labels::UPDATE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_single_field_change_gets_its_own_label() {
        let rename = MarkerPatch {
            label: Some("Intro".to_owned()),
            ..MarkerPatch::default()
        };
        assert_eq!(marker_patch_label(&rename), labels::RENAME);
        let recolor = MarkerPatch {
            color: Some(MarkerColor::Red),
            ..MarkerPatch::default()
        };
        assert_eq!(marker_patch_label(&recolor), labels::RECOLOR);
        let both = MarkerPatch {
            time: Some(Flicks::ZERO),
            color: Some(MarkerColor::Blue),
            ..MarkerPatch::default()
        };
        assert_eq!(marker_patch_label(&both), labels::UPDATE);
    }
}
