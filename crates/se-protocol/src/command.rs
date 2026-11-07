//! Commands a client asks `se-core` to execute.

use se_domain::{
    AssetId, Flicks, Format, ItemId, JobId, MarkerColor, MarkerId, MarkerPatch, ProjectId,
    PropertyAnimation, SequenceId, TrackId, TrackKind, TrackPatch, TransitionId, TransitionPatch,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    CreateProjectRequest, DecodabilityReport, DecoderCapabilities, GenerateDerivedRequest,
    ImportMediaRequest, ItemPlacement, ItemTrim, ItemUpdate, MediaSettings, NewTransition,
    PastedItem, PlaceMediaRequest, PlacementMode, RelinkAssetRequest,
};

/// A command sent inside a [`crate::RequestEnvelope`], tagged by `kind` (camelCase).
///
/// Every variant has a [`crate::CommandResult`] variant **with the same name**: that pairing is
/// how the TypeScript client derives the result type of each command without a hand-written table
/// (ADR-0012). Adding a command: see the recipe in `crates/se-protocol/CLAUDE.md`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Command {
    /// Liveness check. Answered by [`crate::CommandResult::Ping`].
    Ping,
    /// Describes the running core (version, platform, folders). Answered by
    /// [`crate::CommandResult::GetCoreInfo`].
    GetCoreInfo,
    /// Choices for the « new project » dialog (folder, formats, frame rates). Answered by
    /// [`crate::CommandResult::GetNewProjectDefaults`].
    GetNewProjectDefaults,
    /// Creates a project folder with a default sequence (tracks V1, V2, A1, A2) and opens it.
    /// Answered by [`crate::CommandResult::CreateProject`].
    CreateProject(CreateProjectRequest),
    /// Opens the project folder at `path` (replaying its journal after a crash). Opening a project
    /// this core already has open returns its current session. Answered by
    /// [`crate::CommandResult::OpenProject`].
    OpenProject {
        /// Absolute path of a `.seproj` folder.
        path: String,
    },
    /// Saves and closes an open project. Answered by [`crate::CommandResult::CloseProject`].
    CloseProject {
        /// The open project.
        project_id: ProjectId,
    },
    /// The recent projects, most recently opened first. Answered by
    /// [`crate::CommandResult::ListRecentProjects`].
    ListRecentProjects,
    /// Renames a project (open or not; its folder keeps its name). Answered by
    /// [`crate::CommandResult::RenameProject`].
    RenameProject {
        /// Absolute path of the project folder.
        path: String,
        /// The new name (not empty).
        name: String,
    },
    /// Copies a project into a new folder next to it, with a new id and `name`. Answered by
    /// [`crate::CommandResult::DuplicateProject`].
    DuplicateProject {
        /// Absolute path of the project folder to copy.
        path: String,
        /// Name of the copy (not empty).
        name: String,
    },
    /// Closes the project if it is open, moves its folder to the OS trash (never deletes it) and
    /// forgets it in the recent list. Answered by [`crate::CommandResult::TrashProject`].
    TrashProject {
        /// Absolute path of the project folder.
        path: String,
    },
    /// The current session of an open project, e.g. after a missed patch or a reconnection.
    /// Answered by [`crate::CommandResult::GetProjectSnapshot`].
    GetProjectSnapshot {
        /// The open project.
        project_id: ProjectId,
    },
    /// Adds a sequence with tracks V1, V2, A1, A2 to an open project (undoable). Answered by
    /// [`crate::CommandResult::CreateSequence`].
    CreateSequence {
        /// The open project.
        project_id: ProjectId,
        /// Name of the sequence (not empty).
        name: String,
        /// Its format; `null` uses the project's default format.
        format: Option<Format>,
    },
    /// Undoes the latest user transaction of an open project. Answered by
    /// [`crate::CommandResult::Undo`].
    Undo {
        /// The open project.
        project_id: ProjectId,
    },
    /// Redoes the latest undone transaction of an open project. Answered by
    /// [`crate::CommandResult::Redo`].
    Redo {
        /// The open project.
        project_id: ProjectId,
    },
    /// Adds an empty track named after its kind and number (`V3`, `A3`) to a sequence (one undo
    /// step). Answered by [`crate::CommandResult::AddTrack`].
    AddTrack {
        /// The open project.
        project_id: ProjectId,
        /// The sequence receiving the track.
        sequence_id: SequenceId,
        /// Video or audio (not `kind`: that is the command's tag).
        track_kind: TrackKind,
        /// Position in the sequence's track list (video tracks stack in list order, first =
        /// bottom); `null` appends.
        index: Option<usize>,
    },
    /// Removes an unlocked track and its items (one undo step). Answered by
    /// [`crate::CommandResult::RemoveTrack`].
    RemoveTrack {
        /// The open project.
        project_id: ProjectId,
        /// The track.
        track_id: TrackId,
    },
    /// Changes a track header: name, mute, solo, lock, visibility, gain, balance (one undo step).
    /// Answered by [`crate::CommandResult::UpdateTrack`].
    UpdateTrack {
        /// The open project.
        project_id: ProjectId,
        /// The track.
        track_id: TrackId,
        /// The fields to change (absent or `null` fields stay).
        patch: TrackPatch,
    },
    /// Puts the tracks of a sequence in a new order (one undo step). Answered by
    /// [`crate::CommandResult::ReorderTracks`].
    ReorderTracks {
        /// The open project.
        project_id: ProjectId,
        /// The sequence.
        sequence_id: SequenceId,
        /// Every track of the sequence, once each, in the new order.
        order: Vec<TrackId>,
    },
    /// Adds a marker to a sequence (one undo step). Answered by
    /// [`crate::CommandResult::AddMarker`].
    AddMarker {
        /// The open project.
        project_id: ProjectId,
        /// The sequence receiving the marker.
        sequence_id: SequenceId,
        /// Its position (≥ 0).
        time: Flicks,
        /// Its text (may be empty).
        label: String,
        /// Its colour; `null` uses the default (green).
        color: Option<MarkerColor>,
    },
    /// Moves, renames or recolours a marker (one undo step). Answered by
    /// [`crate::CommandResult::UpdateMarker`].
    UpdateMarker {
        /// The open project.
        project_id: ProjectId,
        /// The marker.
        marker_id: MarkerId,
        /// The fields to change (absent or `null` fields stay).
        patch: MarkerPatch,
    },
    /// Removes a marker (one undo step). Answered by [`crate::CommandResult::RemoveMarker`].
    RemoveMarker {
        /// The open project.
        project_id: ProjectId,
        /// The marker.
        marker_id: MarkerId,
    },
    /// Adds dropped media to a sequence as clips placed one after the other (one undo step): a
    /// video with sound becomes two linked items. Answered by
    /// [`crate::CommandResult::PlaceMedia`].
    PlaceMedia(PlaceMediaRequest),
    /// Moves items to new tracks and starts (one undo step); linked items move only when listed.
    /// Answered by [`crate::CommandResult::MoveItems`].
    MoveItems {
        /// The open project.
        project_id: ProjectId,
        /// Every item moved (one sequence), each once.
        moves: Vec<ItemPlacement>,
        /// What happens to items already where they land.
        mode: PlacementMode,
    },
    /// Copies items to new places (one undo step). Answered by
    /// [`crate::CommandResult::DuplicateItems`].
    DuplicateItems {
        /// The open project.
        project_id: ProjectId,
        /// The items copied (one sequence), each once, and where each copy goes.
        copies: Vec<ItemPlacement>,
        /// What happens to items already where the copies land.
        mode: PlacementMode,
    },
    /// Adds copies of clipboard items (one undo step). Answered by
    /// [`crate::CommandResult::PasteItems`].
    PasteItems {
        /// The open project.
        project_id: ProjectId,
        /// The copies and their tracks (one sequence).
        items: Vec<PastedItem>,
        /// What happens to items already where the copies land.
        mode: PlacementMode,
    },
    /// Moves item edges (one undo step). Answered by [`crate::CommandResult::TrimItems`].
    TrimItems {
        /// The open project.
        project_id: ProjectId,
        /// The edges moved, each item once.
        trims: Vec<ItemTrim>,
        /// Shift what follows by the change instead of leaving or filling a gap. A ripple trim
        /// moves edges sitting at one instant (an item and its linked partners) to one place.
        ripple: bool,
    },
    /// Cuts items at an instant (one undo step); right parts of linked items stay linked.
    /// Answered by [`crate::CommandResult::SplitItems`].
    SplitItems {
        /// The open project.
        project_id: ProjectId,
        /// Items crossing `at`.
        item_ids: Vec<ItemId>,
        /// The cut instant.
        at: Flicks,
    },
    /// Removes items (one undo step). Answered by [`crate::CommandResult::DeleteItems`].
    DeleteItems {
        /// The open project.
        project_id: ProjectId,
        /// The items removed (one sequence), each once.
        item_ids: Vec<ItemId>,
        /// Close the time they took on their tracks (and on the tracks of items linked to what
        /// follows) instead of leaving gaps.
        ripple: bool,
    },
    /// Puts items in a new link group (one undo step). Answered by
    /// [`crate::CommandResult::LinkItems`].
    LinkItems {
        /// The open project.
        project_id: ProjectId,
        /// At least two items of one sequence.
        item_ids: Vec<ItemId>,
    },
    /// Takes items out of their link groups (one undo step). Answered by
    /// [`crate::CommandResult::UnlinkItems`].
    UnlinkItems {
        /// The open project.
        project_id: ProjectId,
        /// The items unlinked.
        item_ids: Vec<ItemId>,
    },
    /// Lengthens, shortens or closes the empty time before an item (one undo step): what follows
    /// moves on the track and on the tracks of items linked to it. Answered by
    /// [`crate::CommandResult::ResizeGap`].
    ResizeGap {
        /// The open project.
        project_id: ProjectId,
        /// The track of the gap.
        track_id: TrackId,
        /// Where the gap starts (end of the previous item, or 0).
        start: Flicks,
        /// Where it ends (start of the next item).
        end: Flicks,
        /// Its new length (0 closes it).
        duration: Flicks,
    },
    /// Changes the content of items (one undo step): fades and their shapes, speed, reframing…
    /// Answered by [`crate::CommandResult::UpdateItems`].
    UpdateItems {
        /// The open project.
        project_id: ProjectId,
        /// One typed patch per item (its kind must match the item's), each item once.
        updates: Vec<ItemUpdate>,
    },
    /// Sets the keyframes (or the static value) of one animatable property of an item, e.g. a
    /// clip's volume curve (one undo step). Answered by
    /// [`crate::CommandResult::SetItemAnimation`].
    SetItemAnimation {
        /// The open project.
        project_id: ProjectId,
        /// The item animated.
        item_id: ItemId,
        /// The property and its new keyframes or value.
        animation: PropertyAnimation,
    },
    /// Adds cross-fades on cuts (one undo step). Each is shortened to what its cut allows; cuts
    /// without room are skipped, and the command fails with `TRANSITION_NO_ROOM` when none has
    /// room. Answered by [`crate::CommandResult::AddTransitions`].
    AddTransitions {
        /// The open project.
        project_id: ProjectId,
        /// The cuts and the wanted transitions (kind from the track).
        transitions: Vec<NewTransition>,
    },
    /// Resizes or realigns a transition (one undo step); the duration is shortened to what its
    /// cut allows. Answered by [`crate::CommandResult::UpdateTransition`].
    UpdateTransition {
        /// The open project.
        project_id: ProjectId,
        /// The transition changed.
        transition_id: TransitionId,
        /// What changes.
        patch: TransitionPatch,
    },
    /// Removes transitions (one undo step). Answered by
    /// [`crate::CommandResult::RemoveTransitions`].
    RemoveTransitions {
        /// The open project.
        project_id: ProjectId,
        /// The transitions removed, each once.
        transition_ids: Vec<TransitionId>,
    },
    /// Imports media files into an open project (one undoable transaction), skipping content it
    /// already has, then queues their thumbnails and waveform peaks. Answered by
    /// [`crate::CommandResult::ImportMedia`].
    ImportMedia(ImportMediaRequest),
    /// Points an asset at another file (a moved or renamed original). Answered by
    /// [`crate::CommandResult::RelinkAsset`].
    RelinkAsset(RelinkAssetRequest),
    /// Removes an asset no item uses (undoable). Answered by
    /// [`crate::CommandResult::RemoveAsset`].
    RemoveAsset {
        /// The open project.
        project_id: ProjectId,
        /// The asset.
        asset_id: AssetId,
    },
    /// The background jobs of an open project (queued, running and finished in this session).
    /// Answered by [`crate::CommandResult::ListJobs`].
    ListJobs {
        /// The open project.
        project_id: ProjectId,
    },
    /// Cancels a queued or running job. Answered by [`crate::CommandResult::CancelJob`].
    CancelJob {
        /// The job.
        job_id: JobId,
    },
    /// Records whether this machine's WebCodecs decoders accept an asset's streams (sent by the
    /// renderer after `isConfigSupported`). Answered by
    /// [`crate::CommandResult::ReportDecodability`].
    ReportDecodability(DecodabilityReport),
    /// Records what this machine's WebCodecs decodes (sent by the renderer once per connection,
    /// before decodability reports): decides mezzanine and proxy codecs. Answered by
    /// [`crate::CommandResult::ReportDecoderCapabilities`].
    ReportDecoderCapabilities(DecoderCapabilities),
    /// Queues a mezzanine or proxy of an asset (a proxy on request, or a new attempt after a
    /// failure). Answered by [`crate::CommandResult::GenerateDerivedMedia`].
    GenerateDerivedMedia(GenerateDerivedRequest),
    /// The media settings. Answered by [`crate::CommandResult::GetMediaSettings`].
    GetMediaSettings,
    /// Replaces the media settings (new mezzanines and proxies use them). Answered by
    /// [`crate::CommandResult::UpdateMediaSettings`].
    UpdateMediaSettings(MediaSettings),
    /// Which ffmpeg/ffprobe are used and which hardware encoders work. Answered by
    /// [`crate::CommandResult::GetMediaToolsStatus`].
    GetMediaToolsStatus,
    /// The silent stretches of an asset's sound, from its audio analysis (shown on its clips).
    /// Answered by [`crate::CommandResult::GetAudioSilences`].
    GetAudioSilences {
        /// The open project.
        project_id: ProjectId,
        /// The asset.
        asset_id: AssetId,
    },
    /// The cache folder of an open project and its size. Answered by
    /// [`crate::CommandResult::GetProjectCache`].
    GetProjectCache {
        /// The open project.
        project_id: ProjectId,
    },
    /// Stops the project's jobs, deletes its cache (thumbnails, peaks, mezzanines, proxies) and
    /// queues what is needed again. Analyses are kept. Answered by
    /// [`crate::CommandResult::ClearProjectCache`].
    ClearProjectCache {
        /// The open project.
        project_id: ProjectId,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_are_tagged_by_camel_case_kind() {
        assert_eq!(
            serde_json::to_string(&Command::GetCoreInfo).unwrap(),
            r#"{"kind":"getCoreInfo"}"#
        );
        let parsed: Command = serde_json::from_str(r#"{"kind":"ping"}"#).unwrap();
        assert_eq!(parsed, Command::Ping);
    }

    #[test]
    fn struct_command_fields_are_camel_case() {
        let undo = Command::Undo {
            project_id: ProjectId::from_test_index(1),
        };
        assert_eq!(
            serde_json::to_string(&undo).unwrap(),
            r#"{"kind":"undo","projectId":"00000000-0001-7000-8000-000000000000"}"#
        );
    }

    #[test]
    fn track_and_marker_patches_may_omit_unchanged_fields() {
        let json = r#"{"kind":"updateTrack","projectId":"00000000-0001-7000-8000-000000000000","trackId":"00000000-0002-7000-8000-000000000000","patch":{"muted":true}}"#;
        let Command::UpdateTrack { patch, .. } = serde_json::from_str(json).unwrap() else {
            panic!("updateTrack must parse as Command::UpdateTrack");
        };
        assert_eq!(
            patch,
            TrackPatch {
                muted: Some(true),
                ..TrackPatch::default()
            }
        );
        let json = r#"{"kind":"updateMarker","projectId":"00000000-0001-7000-8000-000000000000","markerId":"00000000-0003-7000-8000-000000000000","patch":{"label":"Intro"}}"#;
        let Command::UpdateMarker { patch, .. } = serde_json::from_str(json).unwrap() else {
            panic!("updateMarker must parse as Command::UpdateMarker");
        };
        assert_eq!(patch.label.as_deref(), Some("Intro"));
        assert_eq!(patch.time, None);
    }

    #[test]
    fn create_project_carries_its_request_next_to_the_kind() {
        let json = r#"{"kind":"createProject","name":"Film","parentDir":"/p","format":{"width":1920,"height":1080,"frameRate":{"numerator":25,"denominator":1},"sampleRate":48000}}"#;
        let Command::CreateProject(request) = serde_json::from_str(json).unwrap() else {
            panic!("createProject must parse as Command::CreateProject");
        };
        assert_eq!(request.name, "Film");
        assert_eq!(request.parent_dir, "/p");
        assert_eq!(request.format.width, 1920);
    }
}
