//! Track edits of open projects: add, remove, change the header, reorder. Each is one user
//! transaction, so one undo step.

use se_domain::{
    AddTrack, ProjectId, RemoveTrack, ReorderTracks, Sequence, SequenceId, TrackId, TrackKind,
    TrackPatch, UpdateTrack,
};

use super::ProjectService;
use crate::{AppError, new_ids::NewIds, new_project::empty_track};

/// Labels of the track transactions (undo menu). Stable identifiers: the renderer translates them
/// (`apps/desktop/src/app/editor/history-label.ts`).
pub mod labels {
    /// A track was added.
    pub const ADD: &str = "Add track";
    /// A track was removed.
    pub const REMOVE: &str = "Remove track";
    /// The tracks of a sequence were reordered.
    pub const REORDER: &str = "Reorder tracks";
    /// A track was renamed.
    pub const RENAME: &str = "Rename track";
    /// A track was muted.
    pub const MUTE: &str = "Mute track";
    /// A track was unmuted.
    pub const UNMUTE: &str = "Unmute track";
    /// A track was put in solo.
    pub const SOLO: &str = "Solo track";
    /// A track left solo.
    pub const UNSOLO: &str = "Unsolo track";
    /// A track was locked.
    pub const LOCK: &str = "Lock track";
    /// A track was unlocked.
    pub const UNLOCK: &str = "Unlock track";
    /// A track was hidden.
    pub const HIDE: &str = "Hide track";
    /// A track was shown again.
    pub const SHOW: &str = "Show track";
    /// The gain of a track changed.
    pub const VOLUME: &str = "Change track volume";
    /// The stereo balance of a track changed.
    pub const PAN: &str = "Change track pan";
    /// Several header fields changed at once.
    pub const UPDATE: &str = "Update track";
}

impl ProjectService {
    /// Adds an empty track of `kind` at `index` of the sequence's track list (`None` appends),
    /// named after its kind and the first free number (`V3`, `A3`).
    ///
    /// # Errors
    /// Not open, unknown sequence, index past the end, storage errors.
    pub async fn add_track(
        &self,
        project_id: ProjectId,
        sequence_id: SequenceId,
        kind: TrackKind,
        index: Option<usize>,
    ) -> Result<TrackId, AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let name = view
            .project
            .sequences
            .iter()
            .find(|sequence| sequence.id == sequence_id)
            .map_or_else(
                || track_name(kind, 1),
                |sequence| next_track_name(sequence, kind),
            );
        let track = empty_track(NewIds(self.deps.ids.as_ref()), &name, kind)?;
        let track_id = track.id;
        let operation = AddTrack {
            sequence: sequence_id,
            index,
            track,
        };
        self.commit_user_edit(&handle, labels::ADD, vec![operation.into()])
            .await?;
        Ok(track_id)
    }

    /// Removes an unlocked track and its items.
    ///
    /// # Errors
    /// Not open, unknown or locked track, storage errors.
    pub async fn remove_track(
        &self,
        project_id: ProjectId,
        track_id: TrackId,
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id).await?;
        let operation = RemoveTrack { track: track_id };
        self.commit_user_edit(&handle, labels::REMOVE, vec![operation.into()])
            .await
    }

    /// Changes a track header (locked or not: this is how a track gets unlocked).
    ///
    /// # Errors
    /// Not open, unknown track, empty name, storage errors.
    pub async fn update_track(
        &self,
        project_id: ProjectId,
        track_id: TrackId,
        patch: TrackPatch,
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id).await?;
        let label = track_patch_label(&patch);
        let operation = UpdateTrack {
            track: track_id,
            patch,
        };
        self.commit_user_edit(&handle, label, vec![operation.into()])
            .await
    }

    /// Puts the tracks of a sequence in `order` (every track once).
    ///
    /// # Errors
    /// Not open, unknown sequence, an order that is not a permutation of its tracks, storage
    /// errors.
    pub async fn reorder_tracks(
        &self,
        project_id: ProjectId,
        sequence_id: SequenceId,
        order: Vec<TrackId>,
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id).await?;
        let operation = ReorderTracks {
            sequence: sequence_id,
            order,
        };
        self.commit_user_edit(&handle, labels::REORDER, vec![operation.into()])
            .await
    }
}

/// `V<n>` or `A<n>`.
fn track_name(kind: TrackKind, number: usize) -> String {
    let letter = match kind {
        TrackKind::Video => 'V',
        TrackKind::Audio => 'A',
    };
    format!("{letter}{number}")
}

/// The first `V<n>`/`A<n>` name not used by a track of `sequence`, counting from the number of
/// tracks of that kind plus one (so a third video track is `V3` unless that name is taken).
fn next_track_name(sequence: &Sequence, kind: TrackKind) -> String {
    let same_kind = sequence
        .tracks
        .iter()
        .filter(|track| track.kind == kind)
        .count();
    // One more candidate than there are tracks: at least one of them is free.
    let first = same_kind + 1;
    (first..=first + sequence.tracks.len())
        .map(|number| track_name(kind, number))
        .find(|name| sequence.tracks.iter().all(|track| &track.name != name))
        .unwrap_or_else(|| track_name(kind, first))
}

/// The undo label of a header change: specific when one field changes.
fn track_patch_label(patch: &TrackPatch) -> &'static str {
    let toggle = |value: bool, on: &'static str, off: &'static str| if value { on } else { off };
    let labels: Vec<&'static str> = [
        patch.name.as_ref().map(|_| labels::RENAME),
        patch
            .muted
            .map(|muted| toggle(muted, labels::MUTE, labels::UNMUTE)),
        patch
            .solo
            .map(|solo| toggle(solo, labels::SOLO, labels::UNSOLO)),
        patch
            .locked
            .map(|locked| toggle(locked, labels::LOCK, labels::UNLOCK)),
        patch
            .hidden
            .map(|hidden| toggle(hidden, labels::HIDE, labels::SHOW)),
        patch.volume_db.map(|_| labels::VOLUME),
        patch.pan.map(|_| labels::PAN),
    ]
    .into_iter()
    .flatten()
    .collect();
    match labels.as_slice() {
        [single] => single,
        _ => labels::UPDATE,
    }
}

#[cfg(test)]
mod tests {
    use se_domain::test_support::track;

    use super::*;

    fn sequence_with(names: &[(&str, TrackKind)]) -> Sequence {
        let mut sequence = se_domain::test_support::example_project().sequences[0].clone();
        sequence.tracks = names
            .iter()
            .zip(100..)
            .map(|((name, kind), index)| track(index, *kind, name, Vec::new()))
            .collect();
        sequence
    }

    #[test]
    fn new_tracks_take_the_next_free_number_of_their_kind() {
        let sequence = sequence_with(&[
            ("V1", TrackKind::Video),
            ("V2", TrackKind::Video),
            ("A1", TrackKind::Audio),
        ]);
        assert_eq!(next_track_name(&sequence, TrackKind::Video), "V3");
        assert_eq!(next_track_name(&sequence, TrackKind::Audio), "A2");
        let renamed = sequence_with(&[("V2", TrackKind::Video), ("Titres", TrackKind::Video)]);
        assert_eq!(next_track_name(&renamed, TrackKind::Video), "V3");
        let taken = sequence_with(&[("V2", TrackKind::Video)]);
        assert_eq!(next_track_name(&taken, TrackKind::Video), "V3");
    }

    #[test]
    fn a_single_field_change_gets_its_own_label() {
        let mute = TrackPatch {
            muted: Some(true),
            ..TrackPatch::default()
        };
        assert_eq!(track_patch_label(&mute), labels::MUTE);
        let unlock = TrackPatch {
            locked: Some(false),
            ..TrackPatch::default()
        };
        assert_eq!(track_patch_label(&unlock), labels::UNLOCK);
        let several = TrackPatch {
            name: Some("Voix".to_owned()),
            hidden: Some(true),
            ..TrackPatch::default()
        };
        assert_eq!(track_patch_label(&several), labels::UPDATE);
        assert_eq!(track_patch_label(&TrackPatch::default()), labels::UPDATE);
    }
}
