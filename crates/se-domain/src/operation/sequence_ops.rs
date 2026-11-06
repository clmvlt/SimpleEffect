//! Sequence operations: [`AddSequence`], [`RemoveSequence`], [`UpdateSequence`].

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    Edit, Effect, TouchedEntities,
    checks::{ensure_name, ensure_unlocked, ensure_unused, insertion_index, swap_field},
    lookup::find_sequence,
};
use crate::{DomainError, Format, Project, Sequence, SequenceId};

/// Adds a sequence (with its tracks, items and markers, if any). Inverse: [`RemoveSequence`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AddSequence {
    /// Position in the project's sequences (`null` appends).
    pub index: Option<usize>,
    /// The sequence, with unused ids and a non-empty name.
    pub sequence: Sequence,
}

impl Edit for AddSequence {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let id = self.sequence.id;
        ensure_unused(find_sequence(project, id).is_ok(), "sequence", &id)?;
        ensure_name("sequence", &self.sequence.name)?;
        let index = insertion_index("sequence", self.index, project.sequences.len())?;
        project.sequences.insert(index, self.sequence.clone());
        effect.touched.merge(contents(&self.sequence));
        effect.undo_with(RemoveSequence { sequence: id });
        Ok(())
    }
}

/// Removes a sequence and everything in it; refused if one of its tracks is locked.
/// Inverse: [`AddSequence`] at the same position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RemoveSequence {
    /// The sequence removed.
    pub sequence: SequenceId,
}

impl Edit for RemoveSequence {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let index = find_sequence(project, self.sequence)?;
        for track in &project.sequences[index].tracks {
            ensure_unlocked(track)?;
        }
        let sequence = project.sequences.remove(index);
        effect.touched.merge(contents(&sequence));
        effect.undo_with(AddSequence {
            index: Some(index),
            sequence,
        });
        Ok(())
    }
}

/// Changes of a sequence; `null` (or absent) fields are unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct SequencePatch {
    /// Another name (not empty).
    pub name: Option<String>,
    /// Another format (frame size and rates). Times are flicks, so items keep their positions.
    pub format: Option<Format>,
}

/// Renames a sequence or changes its format.
/// Inverse: [`UpdateSequence`] with the previous values of the changed fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSequence {
    /// The sequence changed.
    pub sequence: SequenceId,
    /// What changes.
    pub patch: SequencePatch,
}

impl Edit for UpdateSequence {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let index = find_sequence(project, self.sequence)?;
        if let Some(name) = &self.patch.name {
            ensure_name("sequence", name)?;
        }
        let sequence = &mut project.sequences[index];
        let previous = SequencePatch {
            name: swap_field(&mut sequence.name, self.patch.name.as_ref()),
            format: swap_field(&mut sequence.format, self.patch.format.as_ref()),
        };
        effect.touched.sequences.insert(self.sequence);
        effect.undo_with(Self {
            sequence: self.sequence,
            patch: previous,
        });
        Ok(())
    }
}

/// Everything a sequence holds, as touched entities.
fn contents(sequence: &Sequence) -> TouchedEntities {
    let mut touched = TouchedEntities::default();
    touched.sequences.insert(sequence.id);
    touched
        .markers
        .extend(sequence.markers.iter().map(|marker| marker.id));
    for track in &sequence.tracks {
        touched.tracks.insert(track.id);
        touched.items.extend(track.items.iter().map(|item| item.id));
    }
    touched
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FormatPreset, FrameRate, SequenceId,
        operation::testing::{SEQUENCE, V1, apply_err, apply_ok, example, lock},
    };

    fn empty_sequence(index: u64) -> Sequence {
        Sequence {
            id: SequenceId::from_test_index(index),
            name: "Vertical".to_owned(),
            format: Format::from_preset(FormatPreset::Vertical9x16, FrameRate::FPS_30).unwrap(),
            tracks: Vec::new(),
            markers: Vec::new(),
        }
    }

    #[test]
    fn adds_and_removes_sequences() {
        let add = AddSequence {
            index: Some(0),
            sequence: empty_sequence(2),
        };
        let applied = apply_ok(&example(), add.clone());
        assert_eq!(
            applied.project.sequences[0].id,
            SequenceId::from_test_index(2)
        );
        assert!(matches!(
            apply_err(&applied.project, add),
            DomainError::IdAlreadyUsed {
                kind: "sequence",
                ..
            }
        ));
        let applied = apply_ok(&example(), RemoveSequence { sequence: SEQUENCE });
        assert_eq!(applied.project.sequences, []);
        assert_eq!(applied.touched.items.len(), 9);
        assert_eq!(applied.touched.markers.len(), 1);
    }

    #[test]
    fn a_sequence_with_a_locked_track_stays() {
        assert!(matches!(
            apply_err(&lock(example(), V1), RemoveSequence { sequence: SEQUENCE }),
            DomainError::TrackLocked { .. }
        ));
    }

    #[test]
    fn renames_and_reformats() {
        let vertical = Format::from_preset(FormatPreset::Vertical9x16, FrameRate::FPS_25).unwrap();
        let patch = SequencePatch {
            name: Some("Short".to_owned()),
            format: Some(vertical),
        };
        let applied = apply_ok(
            &example(),
            UpdateSequence {
                sequence: SEQUENCE,
                patch,
            },
        );
        assert_eq!(applied.project.sequences[0].format, vertical);
        let blank = SequencePatch {
            name: Some(String::new()),
            format: None,
        };
        assert!(matches!(
            apply_err(
                &example(),
                UpdateSequence {
                    sequence: SEQUENCE,
                    patch: blank
                }
            ),
            DomainError::EmptyName { entity: "sequence" }
        ));
    }
}
