//! Marker operations: [`AddMarker`], [`RemoveMarker`], [`UpdateMarker`].

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    Edit, Effect,
    checks::{ensure_unused, insertion_index, swap_field},
    lookup::{find_marker, find_sequence},
};
use crate::{DomainError, Flicks, Marker, MarkerColor, MarkerId, Project, SequenceId};

/// Adds a marker to a sequence. Inverse: [`RemoveMarker`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AddMarker {
    /// The sequence receiving the marker.
    pub sequence: SequenceId,
    /// Position in the sequence's markers (`null` appends).
    pub index: Option<usize>,
    /// The marker, with an unused id.
    pub marker: Marker,
}

impl Edit for AddMarker {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let sequence_index = find_sequence(project, self.sequence)?;
        let id = self.marker.id;
        ensure_unused(find_marker(project, id).is_ok(), "marker", &id)?;
        let markers = &mut project.sequences[sequence_index].markers;
        let index = insertion_index("marker", self.index, markers.len())?;
        markers.insert(index, self.marker.clone());
        effect.touched.markers.insert(id);
        effect.undo_with(RemoveMarker { marker: id });
        Ok(())
    }
}

/// Removes a marker. Inverse: [`AddMarker`] at the same position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RemoveMarker {
    /// The marker removed.
    pub marker: MarkerId,
}

impl Edit for RemoveMarker {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let (sequence_index, index) = find_marker(project, self.marker)?;
        let sequence = &mut project.sequences[sequence_index];
        let marker = sequence.markers.remove(index);
        effect.touched.markers.insert(self.marker);
        effect.undo_with(AddMarker {
            sequence: sequence.id,
            index: Some(index),
            marker,
        });
        Ok(())
    }
}

/// Changes of a marker; `null` (or absent) fields are unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct MarkerPatch {
    /// Another position (≥ 0).
    pub time: Option<Flicks>,
    /// Another label (may be empty).
    pub label: Option<String>,
    /// Another colour.
    pub color: Option<MarkerColor>,
}

/// Moves, relabels or recolours a marker.
/// Inverse: [`UpdateMarker`] with the previous values of the changed fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMarker {
    /// The marker changed.
    pub marker: MarkerId,
    /// What changes.
    pub patch: MarkerPatch,
}

impl Edit for UpdateMarker {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let (sequence_index, index) = find_marker(project, self.marker)?;
        let marker = &mut project.sequences[sequence_index].markers[index];
        let previous = MarkerPatch {
            time: swap_field(&mut marker.time, self.patch.time.as_ref()),
            label: swap_field(&mut marker.label, self.patch.label.as_ref()),
            color: swap_field(&mut marker.color, self.patch.color.as_ref()),
        };
        effect.touched.markers.insert(self.marker);
        effect.undo_with(Self {
            marker: self.marker,
            patch: previous,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        operation::testing::{SEQUENCE, apply_err, apply_ok, example},
        test_support::millis,
    };

    fn marker(index: u64, time_ms: i64) -> Marker {
        Marker {
            id: MarkerId::from_test_index(index),
            time: millis(time_ms),
            label: String::new(),
            color: MarkerColor::Red,
        }
    }

    #[test]
    fn adds_updates_and_removes_markers() {
        let add = AddMarker {
            sequence: SEQUENCE,
            index: None,
            marker: marker(2, 9000),
        };
        let applied = apply_ok(&example(), add.clone());
        assert_eq!(applied.project.sequences[0].markers.len(), 2);
        assert!(matches!(
            apply_err(&applied.project, add),
            DomainError::IdAlreadyUsed { kind: "marker", .. }
        ));
        let patch = MarkerPatch {
            label: Some("Drop".to_owned()),
            ..MarkerPatch::default()
        };
        let update = UpdateMarker {
            marker: MarkerId::from_test_index(1),
            patch,
        };
        let applied = apply_ok(&example(), update);
        assert_eq!(applied.project.sequences[0].markers[0].label, "Drop");
        let remove = RemoveMarker {
            marker: MarkerId::from_test_index(1),
        };
        assert_eq!(
            apply_ok(&example(), remove).project.sequences[0].markers,
            []
        );
    }

    #[test]
    fn markers_stay_at_or_after_zero() {
        let patch = MarkerPatch {
            time: Some(millis(-1)),
            ..MarkerPatch::default()
        };
        let update = UpdateMarker {
            marker: MarkerId::from_test_index(1),
            patch,
        };
        assert!(matches!(
            apply_err(&example(), update),
            DomainError::InvalidDocument { .. }
        ));
    }
}
