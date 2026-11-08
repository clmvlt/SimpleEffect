//! The caption track of a sequence: where generated and imported captions go.

use se_domain::{AddTrack, ItemKind, Sequence, SequenceId, Track, TrackId, TrackKind};

use crate::{
    AppError,
    edit_plan::{EditPlan, find_sequence},
    new_project::empty_track,
};

/// Name of a caption track created without a name.
pub const DEFAULT_CAPTION_TRACK_NAME: &str = "Sous-titres";

/// The caption track of `sequence`: the highest unlocked video track holding captions and
/// nothing else. `None` when the sequence has none (a new one is made on top).
#[must_use]
pub fn caption_track(sequence: &Sequence) -> Option<&Track> {
    sequence
        .tracks
        .iter()
        .rev()
        .filter(|track| track.kind == TrackKind::Video && !track.locked)
        .find(|track| {
            !track.items.is_empty()
                && track
                    .items
                    .iter()
                    .all(|item| matches!(item.kind, ItemKind::Caption(_)))
        })
}

/// Whether `track` holds captions only (at least one): a caption track, which follows the speech
/// when smart cuts remove time.
#[must_use]
pub fn is_caption_track(track: &Track) -> bool {
    track.kind == TrackKind::Video
        && !track.items.is_empty()
        && track
            .items
            .iter()
            .all(|item| matches!(item.kind, ItemKind::Caption(_)))
}

/// The caption track of `sequence` in the plan's draft, created on top of the video tracks
/// (named `name`, else « Sous-titres ») when it has none. Returns the track and whether it was
/// created.
///
/// # Errors
/// Unknown sequence, id generator failures.
pub(crate) fn ensure_caption_track(
    plan: &mut EditPlan<'_>,
    sequence_id: SequenceId,
    name: Option<&str>,
) -> Result<(TrackId, bool), AppError> {
    let ids = plan.ids();
    let sequence = find_sequence(plan.project()?, sequence_id)?;
    if let Some(track) = caption_track(sequence) {
        return Ok((track.id, false));
    }
    let name = name
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or(DEFAULT_CAPTION_TRACK_NAME);
    let track = empty_track(ids, name, TrackKind::Video)?;
    let track_id = track.id;
    let index = top_video_index(sequence);
    plan.push(AddTrack {
        sequence: sequence_id,
        index: Some(index),
        track,
    });
    Ok((track_id, true))
}

/// Where a new top video track goes: after the last video track.
fn top_video_index(sequence: &Sequence) -> usize {
    sequence
        .tracks
        .iter()
        .rposition(|track| track.kind == TrackKind::Video)
        .map_or(0, |index| index + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use se_domain::test_support::example_project;

    #[test]
    fn the_caption_track_holds_only_captions() {
        let project = example_project();
        // V3 holds a template, a shape and the caption: not a caption track.
        assert!(caption_track(&project.sequences[0]).is_none());
        let mut project = project;
        let sequence = &mut project.sequences[0];
        sequence.tracks[2]
            .items
            .retain(|item| matches!(item.kind, ItemKind::Caption(_)));
        assert_eq!(
            caption_track(sequence).map(|track| track.id),
            Some(sequence.tracks[2].id)
        );
        assert!(is_caption_track(&sequence.tracks[2]));
        sequence.tracks[2].locked = true;
        assert!(caption_track(sequence).is_none());
    }
}
