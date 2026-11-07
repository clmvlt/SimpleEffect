//! [`format_change`] and [`sequence_version`]: a sequence in another format, in place or as a
//! copy.

use super::{FormatAdaptation, SequenceIds, adapted_patch, copy_sequence};
use crate::{
    DomainError, Format, Operation, Project, Sequence, SequenceId, SequencePatch, TrackKind,
    UpdateItem, UpdateSequence,
};

/// The operations that give `sequence` the format `format` and adapt its items: one
/// [`UpdateSequence`], then one [`UpdateItem`] per item that changes (see
/// [`super::adapted_patch`]). Items on locked tracks stay as they are. Commit them as one
/// transaction: their inverses undo the change exactly.
///
/// # Errors
/// [`DomainError::SequenceNotFound`]; [`DomainError::FrameRateLocked`] when the frame rate
/// changes while the sequence holds items.
pub fn format_change(
    project: &Project,
    sequence: SequenceId,
    format: Format,
    adaptation: FormatAdaptation,
) -> Result<Vec<Operation>, DomainError> {
    let source = project
        .sequences
        .iter()
        .find(|candidate| candidate.id == sequence)
        .ok_or(DomainError::SequenceNotFound { sequence })?;
    ensure_frame_rate_kept(source, &format)?;
    let mut operations: Vec<Operation> = vec![
        UpdateSequence {
            sequence,
            patch: SequencePatch {
                name: None,
                format: Some(format),
            },
        }
        .into(),
    ];
    for track in source.tracks.iter().filter(|track| !track.locked) {
        let on_video_track = track.kind == TrackKind::Video;
        for item in &track.items {
            if let Some(patch) = adapted_patch(&item.kind, on_video_track, &format, adaptation) {
                operations.push(
                    UpdateItem {
                        item: item.id,
                        patch,
                        expected_revision: None,
                    }
                    .into(),
                );
            }
        }
    }
    Ok(operations)
}

/// A copy of `source` named `name` in the format `format`, its items adapted (locked tracks
/// included: the copy is new content), with fresh ids (see [`copy_sequence`]). Add it with
/// [`crate::AddSequence`]; it is independent of `source` from then on.
///
/// # Errors
/// [`DomainError::FrameRateLocked`] (converted) when the frame rate differs while `source` holds
/// items; the id generator failed.
pub fn sequence_version<I: SequenceIds>(
    source: &Sequence,
    name: String,
    format: Format,
    adaptation: FormatAdaptation,
    ids: &mut I,
) -> Result<Sequence, I::Error> {
    ensure_frame_rate_kept(source, &format)?;
    let mut copy = copy_sequence(source, name, ids)?;
    copy.format = format;
    for track in &mut copy.tracks {
        let on_video_track = track.kind == TrackKind::Video;
        for item in &mut track.items {
            if let Some(patch) = adapted_patch(&item.kind, on_video_track, &format, adaptation) {
                patch.apply(&mut item.kind);
            }
        }
    }
    Ok(copy)
}

/// Items are placed on frames of their sequence's rate: once there are some, the rate stays.
fn ensure_frame_rate_kept(sequence: &Sequence, format: &Format) -> Result<(), DomainError> {
    let holds_items = sequence.tracks.iter().any(|track| !track.items.is_empty());
    if holds_items && format.frame_rate != sequence.format.frame_rate {
        return Err(DomainError::FrameRateLocked {
            sequence: sequence.id,
            current: sequence.format.frame_rate,
            requested: format.frame_rate,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AddSequence, ClipReframePolicy, FormatPreset, FrameRate, ItemKind, Reframe, apply_all,
        reformat::copy::CountingIds, test_support::example_project, validate_project,
    };

    fn vertical(rate: FrameRate) -> Format {
        Format::from_preset(FormatPreset::Vertical9x16, rate).unwrap()
    }

    fn fill_and_fit_texts() -> FormatAdaptation {
        FormatAdaptation {
            clip_reframe: ClipReframePolicy::Fill,
            fit_text_widths: true,
        }
    }

    fn video_clips(sequence: &Sequence) -> Vec<&Reframe> {
        sequence
            .tracks
            .iter()
            .filter(|track| track.kind == TrackKind::Video)
            .flat_map(|track| &track.items)
            .filter_map(|item| match &item.kind {
                ItemKind::MediaClip(clip) => Some(&clip.reframe),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_format_change_reframes_clips_and_undoes_exactly() {
        let project = example_project();
        let sequence = project.sequences[0].id;
        let rate = project.sequences[0].format.frame_rate;
        let operations =
            format_change(&project, sequence, vertical(rate), fill_and_fit_texts()).unwrap();
        let applied = apply_all(&project, &operations).unwrap();
        let changed = applied.project;
        assert_eq!(validate_project(&changed), []);
        assert_eq!(changed.sequences[0].format, vertical(rate));
        assert!(
            video_clips(&changed.sequences[0])
                .iter()
                .all(|reframe| matches!(reframe, Reframe::Fill { .. }))
        );
        let restored = apply_all(&changed, &applied.inverse).unwrap().project;
        assert_eq!(restored, project);
    }

    #[test]
    fn the_frame_rate_stays_once_a_sequence_holds_items() {
        let project = example_project();
        let sequence = &project.sequences[0];
        let other_rate = if sequence.format.frame_rate == FrameRate::FPS_25 {
            FrameRate::FPS_30
        } else {
            FrameRate::FPS_25
        };
        let refused = format_change(
            &project,
            sequence.id,
            vertical(other_rate),
            FormatAdaptation::default(),
        );
        assert!(matches!(refused, Err(DomainError::FrameRateLocked { .. })));
        let mut empty = sequence.clone();
        empty.tracks.iter_mut().for_each(|track| {
            track.items.clear();
            track.transitions.clear();
        });
        let copy = sequence_version(
            &empty,
            "Vide".to_owned(),
            vertical(other_rate),
            FormatAdaptation::default(),
            &mut CountingIds::after(5000),
        )
        .unwrap();
        assert_eq!(copy.format.frame_rate, other_rate);
    }

    #[test]
    fn a_version_is_an_independent_valid_sequence_in_the_new_format() {
        let project = example_project();
        let source = &project.sequences[0];
        let version = sequence_version(
            source,
            "Short".to_owned(),
            vertical(source.format.frame_rate),
            fill_and_fit_texts(),
            &mut CountingIds::after(5000),
        )
        .unwrap();
        assert!(
            video_clips(&version)
                .iter()
                .all(|reframe| matches!(reframe, Reframe::Fill { .. }))
        );
        assert_eq!(video_clips(source).len(), video_clips(&version).len());
        let added = apply_all(
            &project,
            &[AddSequence {
                index: None,
                sequence: version,
            }
            .into()],
        )
        .unwrap()
        .project;
        assert_eq!(validate_project(&added), []);
        assert_eq!(added.sequences[0], project.sequences[0]);
    }
}
