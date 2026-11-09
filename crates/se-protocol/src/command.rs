//! Commands a client asks `se-core` to execute.

use se_domain::{
    AnimationPhase, AssetId, Flicks, Format, FormatAdaptation, ItemId, JobId, MarkerColor,
    MarkerId, MarkerPatch, ProjectId, PropertyAnimation, SequenceId, TaskId, TemplateId, TimeRange,
    TrackId, TrackKind, TrackPatch, TranscriptionSettings, TransitionId, TransitionPatch,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    AddGraphicRequest, AiSettings, CaptionEmphasisEdit, CaptionStyleEdit, CaptionTextEdit,
    CreateProjectRequest, DecodabilityReport, DecoderCapabilities, DownloadTarget,
    GenerateCaptionsRequest, GenerateDerivedRequest, ImportMediaRequest, ImportSubtitlesRequest,
    ItemAnimation, ItemPlacement, ItemTrim, ItemUpdate, MediaSettings, NewTransition, PastedItem,
    PlaceMediaRequest, PlacementMode, RelinkAssetRequest, SilenceCutParams, SmartCutSettings,
    SubmitTaskRequest, TaskScope, TemplateCheckOutcome, TranscriptionPreferences, WordCutPurpose,
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
    /// Renames a sequence (one undo step). Answered by [`crate::CommandResult::RenameSequence`].
    RenameSequence {
        /// The open project.
        project_id: ProjectId,
        /// The sequence.
        sequence_id: SequenceId,
        /// The new name (not empty).
        name: String,
    },
    /// Gives a sequence another format and adapts its items (one undo step): positions are
    /// normalised and sizes design pixels, so texts and graphics keep their relative places;
    /// `adaptation` sets the reframe mode of video clips and may narrow texts. Items on locked
    /// tracks stay. The frame rate cannot change once the sequence holds items
    /// (`FRAME_RATE_LOCKED`). Answered by [`crate::CommandResult::SetSequenceFormat`].
    SetSequenceFormat {
        /// The open project.
        project_id: ProjectId,
        /// The sequence.
        sequence_id: SequenceId,
        /// Its new format.
        format: Format,
        /// How its items adapt.
        adaptation: FormatAdaptation,
    },
    /// Copies a sequence into a new, independent sequence placed after it (one undo step): in
    /// the same format (`format: null`) or as a version in another format, its items adapted as
    /// `adaptation` says. Later edits of one never reach the other. Answered by
    /// [`crate::CommandResult::DuplicateSequence`].
    DuplicateSequence {
        /// The open project.
        project_id: ProjectId,
        /// The sequence copied.
        sequence_id: SequenceId,
        /// Name of the copy (not empty).
        name: String,
        /// Format of the copy; `null` keeps the source's (same frame rate once it holds items).
        format: Option<Format>,
        /// How the copied items adapt to the copy's format.
        adaptation: FormatAdaptation,
    },
    /// Removes a sequence and everything in it (one undo step); refused for the project's only
    /// sequence (`LAST_SEQUENCE`) or with a locked track. Answered by
    /// [`crate::CommandResult::RemoveSequence`].
    RemoveSequence {
        /// The open project.
        project_id: ProjectId,
        /// The sequence.
        sequence_id: SequenceId,
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
    /// Changes the content of items (one undo step): fades and their shapes, speed, reframe
    /// modes… then, in the same step, the animatable properties listed in `animations` (a clip
    /// switched to a fill mode with its focus, a scale keeping its size). Answered by
    /// [`crate::CommandResult::UpdateItems`].
    UpdateItems {
        /// The open project.
        project_id: ProjectId,
        /// One typed patch per item (its kind must match the item's), each item once.
        updates: Vec<ItemUpdate>,
        /// Properties set after the patches (may be omitted).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        animations: Option<Vec<ItemAnimation>>,
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
    /// Sets animatable properties of items: several properties of several items in one undo
    /// step (a gizmo gesture, a multiple selection edited in the inspector, keyframes dragged in
    /// the timeline). Answered by [`crate::CommandResult::SetItemAnimations`].
    SetItemAnimations {
        /// The open project.
        project_id: ProjectId,
        /// The new values, applied in order.
        animations: Vec<ItemAnimation>,
    },
    /// Adds a text, a shape or an image at a time (one undo step), on the highest free video
    /// track (a new one on top when none is free). Answered by
    /// [`crate::CommandResult::AddGraphicItem`].
    AddGraphicItem(AddGraphicRequest),
    /// Expands an animation preset into keyframes of items (one undo step for all of them).
    /// Answered by [`crate::CommandResult::ApplyAnimationPreset`].
    ApplyAnimationPreset {
        /// The open project.
        project_id: ProjectId,
        /// The items animated, each once; each must have every property of the preset.
        item_ids: Vec<ItemId>,
        /// Id of the preset (`listAnimationPresets`).
        preset: String,
        /// Entrance, exit or emphasis.
        phase: AnimationPhase,
        /// How long it lasts (shortened to each item).
        duration: Flicks,
        /// Start relative to each item; `null` for the phase's place.
        start: Option<Flicks>,
    },
    /// The animation preset library. Answered by
    /// [`crate::CommandResult::ListAnimationPresets`].
    ListAnimationPresets,
    /// The fonts texts may use (bundled and installed). Answered by
    /// [`crate::CommandResult::ListFonts`].
    ListFonts,
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
    /// Submits an AI task: it runs in the background once a slot is free. Answered by
    /// [`crate::CommandResult::SubmitTask`].
    SubmitTask(SubmitTaskRequest),
    /// Cancels a queued or running AI task (its agent process is stopped). Answered by
    /// [`crate::CommandResult::CancelTask`].
    CancelTask {
        /// The open project.
        project_id: ProjectId,
        /// The task.
        task_id: TaskId,
    },
    /// Runs a failed or cancelled task's last prompt again. Answered by
    /// [`crate::CommandResult::RetryTask`].
    RetryTask {
        /// The open project.
        project_id: ProjectId,
        /// The task.
        task_id: TaskId,
    },
    /// Sends a new prompt to a finished task's agent, resuming its session. Answered by
    /// [`crate::CommandResult::FollowUpTask`].
    FollowUpTask {
        /// The open project.
        project_id: ProjectId,
        /// The task.
        task_id: TaskId,
        /// What the agent must do next.
        prompt: String,
        /// A new scope for this run and the next ones, on the task's sequence (« Refaire avec
        /// l'IA » widens it to an item moved since); omitted to keep the task's scope.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        scope: Option<TaskScope>,
    },
    /// The AI tasks of an open project, oldest first. Answered by
    /// [`crate::CommandResult::ListTasks`].
    ListTasks {
        /// The open project.
        project_id: ProjectId,
    },
    /// Undoes every edit of a finished AI task that nobody changed since (its changeset), as one
    /// user transaction. Answered by [`crate::CommandResult::RevertTask`] with what was undone
    /// and what was left (conflicts).
    RevertTask {
        /// The open project.
        project_id: ProjectId,
        /// The task.
        task_id: TaskId,
    },
    /// What [`Command::RevertTask`] would do now, without doing it (the UI lists the conflicts
    /// before the user confirms). Answered by [`crate::CommandResult::PreviewTaskRevert`].
    PreviewTaskRevert {
        /// The open project.
        project_id: ProjectId,
        /// The task.
        task_id: TaskId,
    },
    /// The items a task added or changed that still exist (its open changeset). Answered by
    /// [`crate::CommandResult::GetTaskChanges`].
    GetTaskChanges {
        /// The open project.
        project_id: ProjectId,
        /// The task.
        task_id: TaskId,
    },
    /// The event log of a task. Answered by [`crate::CommandResult::GetTaskEvents`].
    GetTaskEvents {
        /// The open project.
        project_id: ProjectId,
        /// The task.
        task_id: TaskId,
    },
    /// The AI provider in use: installed, logged in, capabilities (checked once, then kept).
    /// Answered by [`crate::CommandResult::GetAiStatus`].
    GetAiStatus,
    /// Checks the AI provider again (after the user installed it or logged in). Answered by
    /// [`crate::CommandResult::RefreshAiStatus`].
    RefreshAiStatus,
    /// The AI settings. Answered by [`crate::CommandResult::GetAiSettings`].
    GetAiSettings,
    /// Replaces the AI settings. Answered by [`crate::CommandResult::UpdateAiSettings`].
    UpdateAiSettings(AiSettings),
    /// The renderer cannot draw the frames of a `renderFramesRequested` event (another project
    /// is open, the sequence is gone, decoding failed…): the agent gets `message`. Answered by
    /// [`crate::CommandResult::ReportFrameRenderFailure`].
    ReportFrameRenderFailure {
        /// The request.
        request_id: String,
        /// Why, for the agent (English, actionable).
        message: String,
    },
    /// The renderer's answer to a `templateCheckRequested` event (prompt 24). Answered by
    /// [`crate::CommandResult::ReportTemplateCheck`].
    ReportTemplateCheck {
        /// The request.
        request_id: String,
        /// What the check found.
        outcome: TemplateCheckOutcome,
    },
    /// Removes a motion template no item uses (one undoable step). Answered by
    /// [`crate::CommandResult::RemoveTemplate`].
    RemoveTemplate {
        /// The open project.
        project_id: ProjectId,
        /// The template.
        template_id: TemplateId,
    },
    /// Transcription on this machine: whisper.cpp, GPU, models, downloads, preferences (prompt
    /// 21). Answered by [`crate::CommandResult::GetTranscriptionStatus`].
    GetTranscriptionStatus,
    /// Replaces the transcription preferences. Answered by
    /// [`crate::CommandResult::UpdateTranscriptionPreferences`].
    UpdateTranscriptionPreferences(TranscriptionPreferences),
    /// Starts downloading a model, the voice model or the GPU pack (after the user agreed to its
    /// size); progress comes as `transcriptionStatusChanged` events. Answered by
    /// [`crate::CommandResult::StartTranscriptionDownload`].
    StartTranscriptionDownload {
        /// What to download.
        target: DownloadTarget,
    },
    /// Stops a running download (nothing is installed). Answered by
    /// [`crate::CommandResult::CancelTranscriptionDownload`].
    CancelTranscriptionDownload {
        /// The download.
        target: DownloadTarget,
    },
    /// Copies a ggml model file into the models folder (a catalogue model when its SHA-256 says
    /// so, else a custom one). Answered by [`crate::CommandResult::ImportTranscriptionModel`].
    ImportTranscriptionModel {
        /// Absolute path of the `.bin` file.
        path: String,
    },
    /// Deletes an installed model, the voice model or the GPU pack. Answered by
    /// [`crate::CommandResult::RemoveTranscriptionFile`].
    RemoveTranscriptionFile {
        /// What to delete.
        target: DownloadTarget,
    },
    /// Queues the transcription of assets (again for those already transcribed: the new
    /// transcript replaces the old one). Answered by
    /// [`crate::CommandResult::TranscribeAssets`].
    TranscribeAssets {
        /// The open project.
        project_id: ProjectId,
        /// Assets with sound.
        asset_ids: Vec<AssetId>,
    },
    /// The transcript of an asset, in source time. Answered by
    /// [`crate::CommandResult::GetTranscript`].
    GetTranscript {
        /// The open project.
        project_id: ProjectId,
        /// The asset.
        asset_id: AssetId,
    },
    /// Transcript segments of the project's assets holding every word of `query` (accents and
    /// case ignored). Answered by [`crate::CommandResult::SearchTranscripts`].
    SearchTranscripts {
        /// The open project.
        project_id: ProjectId,
        /// Words to find.
        query: String,
    },
    /// Sets the project's transcription language and vocabulary (one undo step). Answered by
    /// [`crate::CommandResult::SetTranscriptionSettings`].
    SetTranscriptionSettings {
        /// The open project.
        project_id: ProjectId,
        /// The new settings.
        settings: TranscriptionSettings,
    },
    /// The smart-cut settings (filler lists, silence defaults). Answered by
    /// [`crate::CommandResult::GetSmartCutSettings`].
    GetSmartCutSettings,
    /// Saves the smart-cut settings. Answered by
    /// [`crate::CommandResult::UpdateSmartCutSettings`] (the settings as stored).
    UpdateSmartCutSettings(SmartCutSettings),
    /// Cuts the words whose middle lies in `ranges` (sequence time), with natural margins, as one
    /// ripple transaction on their tracks and linked partners. Answered by
    /// [`crate::CommandResult::RemoveWords`].
    RemoveWords {
        /// The open project.
        project_id: ProjectId,
        /// The sequence edited.
        sequence_id: SequenceId,
        /// Ranges holding the words to remove (usually from the start of the first word to the
        /// end of the last one).
        ranges: Vec<TimeRange>,
        /// Words or fillers (the undo label).
        purpose: WordCutPurpose,
        /// Cut every unlocked track (graphics included) instead of the speech's tracks and their
        /// linked partners.
        all_tracks: bool,
    },
    /// The filler words of a sequence range. Answered by [`crate::CommandResult::FindFillers`].
    FindFillers {
        /// The open project.
        project_id: ProjectId,
        /// The sequence read.
        sequence_id: SequenceId,
        /// Where to look; `null` for the whole sequence.
        range: Option<TimeRange>,
    },
    /// The cuts « remove silences » would make, without changing anything. Answered by
    /// [`crate::CommandResult::PreviewSilenceCuts`].
    PreviewSilenceCuts {
        /// The open project.
        project_id: ProjectId,
        /// The sequence read.
        sequence_id: SequenceId,
        /// Where to look; `null` for the whole sequence.
        range: Option<TimeRange>,
        /// Threshold, minimum silence and padding.
        params: SilenceCutParams,
        /// Sound tracks whose silences count (all of them must be silent); `null` for every
        /// unmuted sound track holding media.
        track_ids: Option<Vec<TrackId>>,
        /// Cut every unlocked track.
        all_tracks: bool,
    },
    /// Cuts the silences of a range as one ripple transaction. Answered by
    /// [`crate::CommandResult::RemoveSilences`].
    RemoveSilences {
        /// The open project.
        project_id: ProjectId,
        /// The sequence edited.
        sequence_id: SequenceId,
        /// Where to cut; `null` for the whole sequence.
        range: Option<TimeRange>,
        /// Threshold, minimum silence and padding.
        params: SilenceCutParams,
        /// Sound tracks whose silences count; `null` for every unmuted sound track holding
        /// media.
        track_ids: Option<Vec<TrackId>>,
        /// Cut every unlocked track.
        all_tracks: bool,
    },
    /// Makes captions from the transcript of a sequence range on its caption track (created on
    /// top when missing), replacing the captions crossing the range (one undo step). Answered by
    /// [`crate::CommandResult::GenerateCaptions`].
    GenerateCaptions(GenerateCaptionsRequest),
    /// Turns an SRT file into captions (words spread equally over each cue; one undo step).
    /// Answered by [`crate::CommandResult::ImportSubtitles`].
    ImportSubtitles(ImportSubtitlesRequest),
    /// Writes the captions of a sequence to an SRT file. Answered by
    /// [`crate::CommandResult::ExportSubtitles`].
    ExportSubtitles {
        /// The open project.
        project_id: ProjectId,
        /// The sequence read.
        sequence_id: SequenceId,
        /// Absolute path of the `.srt` file (replaced when it exists).
        path: String,
    },
    /// Changes the text of a caption (unchanged words keep their timing). Answered by
    /// [`crate::CommandResult::EditCaptionText`].
    EditCaptionText(CaptionTextEdit),
    /// Merges adjacent captions of one track into the first. Answered by
    /// [`crate::CommandResult::MergeCaptions`].
    MergeCaptions {
        /// The open project.
        project_id: ProjectId,
        /// Two or more captions following each other on one track.
        item_ids: Vec<ItemId>,
    },
    /// Splits a caption so that a word starts the new right part. Answered by
    /// [`crate::CommandResult::SplitCaption`].
    SplitCaption {
        /// The open project.
        project_id: ProjectId,
        /// The caption.
        item_id: ItemId,
        /// The first word of the right part (1 to the number of words − 1).
        word_index: u32,
    },
    /// Moves captions in time together. Answered by [`crate::CommandResult::ShiftCaptions`].
    ShiftCaptions {
        /// The open project.
        project_id: ProjectId,
        /// The captions (every caption of a track for a global shift).
        item_ids: Vec<ItemId>,
        /// How far (negative: earlier).
        delta: Flicks,
    },
    /// Puts words of a caption forward (the style's emphasis colour) or back. Answered by
    /// [`crate::CommandResult::SetCaptionEmphasis`].
    SetCaptionEmphasis(CaptionEmphasisEdit),
    /// Changes the look of captions (copy on write of shared styles). Answered by
    /// [`crate::CommandResult::UpdateCaptionStyle`].
    UpdateCaptionStyle(CaptionStyleEdit),
    /// Gives captions a new style made from a predefined one. Answered by
    /// [`crate::CommandResult::ApplyCaptionPreset`].
    ApplyCaptionPreset {
        /// The open project.
        project_id: ProjectId,
        /// The captions.
        item_ids: Vec<ItemId>,
        /// The preset's id (`tiktokBold`, `minimal`, `karaoke`, `box`).
        preset: String,
    },
}

#[cfg(test)]
mod tests;
