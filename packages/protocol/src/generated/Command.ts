// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AddGraphicRequest } from "./AddGraphicRequest";
import type { AiSettings } from "./AiSettings";
import type { AnimationPhase } from "./AnimationPhase";
import type { AssetId } from "./AssetId";
import type { CaptionEmphasisEdit } from "./CaptionEmphasisEdit";
import type { CaptionStyleEdit } from "./CaptionStyleEdit";
import type { CaptionTextEdit } from "./CaptionTextEdit";
import type { CreateProjectRequest } from "./CreateProjectRequest";
import type { DecodabilityReport } from "./DecodabilityReport";
import type { DecoderCapabilities } from "./DecoderCapabilities";
import type { DownloadTarget } from "./DownloadTarget";
import type { Flicks } from "./Flicks";
import type { Format } from "./Format";
import type { FormatAdaptation } from "./FormatAdaptation";
import type { GenerateCaptionsRequest } from "./GenerateCaptionsRequest";
import type { GenerateDerivedRequest } from "./GenerateDerivedRequest";
import type { ImportMediaRequest } from "./ImportMediaRequest";
import type { ImportSubtitlesRequest } from "./ImportSubtitlesRequest";
import type { ItemAnimation } from "./ItemAnimation";
import type { ItemId } from "./ItemId";
import type { ItemPlacement } from "./ItemPlacement";
import type { ItemTrim } from "./ItemTrim";
import type { ItemUpdate } from "./ItemUpdate";
import type { JobId } from "./JobId";
import type { MarkerColor } from "./MarkerColor";
import type { MarkerId } from "./MarkerId";
import type { MarkerPatch } from "./MarkerPatch";
import type { MediaSettings } from "./MediaSettings";
import type { NewTransition } from "./NewTransition";
import type { PastedItem } from "./PastedItem";
import type { PlaceMediaRequest } from "./PlaceMediaRequest";
import type { PlacementMode } from "./PlacementMode";
import type { ProjectId } from "./ProjectId";
import type { PropertyAnimation } from "./PropertyAnimation";
import type { RelinkAssetRequest } from "./RelinkAssetRequest";
import type { SequenceId } from "./SequenceId";
import type { SilenceCutParams } from "./SilenceCutParams";
import type { SmartCutSettings } from "./SmartCutSettings";
import type { SubmitTaskRequest } from "./SubmitTaskRequest";
import type { TaskId } from "./TaskId";
import type { TaskScope } from "./TaskScope";
import type { TimeRange } from "./TimeRange";
import type { TrackId } from "./TrackId";
import type { TrackKind } from "./TrackKind";
import type { TrackPatch } from "./TrackPatch";
import type { TranscriptionPreferences } from "./TranscriptionPreferences";
import type { TranscriptionSettings } from "./TranscriptionSettings";
import type { TransitionId } from "./TransitionId";
import type { TransitionPatch } from "./TransitionPatch";
import type { WordCutPurpose } from "./WordCutPurpose";

/**
 * A command sent inside a [`crate::RequestEnvelope`], tagged by `kind` (camelCase).
 *
 * Every variant has a [`crate::CommandResult`] variant **with the same name**: that pairing is
 * how the TypeScript client derives the result type of each command without a hand-written table
 * (ADR-0012). Adding a command: see the recipe in `crates/se-protocol/CLAUDE.md`.
 */
export type Command = { "kind": "ping" } | { "kind": "getCoreInfo" } | { "kind": "getNewProjectDefaults" } | { "kind": "createProject" } & CreateProjectRequest | { "kind": "openProject", 
/**
 * Absolute path of a `.seproj` folder.
 */
path: string, } | { "kind": "closeProject", 
/**
 * The open project.
 */
projectId: ProjectId, } | { "kind": "listRecentProjects" } | { "kind": "renameProject", 
/**
 * Absolute path of the project folder.
 */
path: string, 
/**
 * The new name (not empty).
 */
name: string, } | { "kind": "duplicateProject", 
/**
 * Absolute path of the project folder to copy.
 */
path: string, 
/**
 * Name of the copy (not empty).
 */
name: string, } | { "kind": "trashProject", 
/**
 * Absolute path of the project folder.
 */
path: string, } | { "kind": "getProjectSnapshot", 
/**
 * The open project.
 */
projectId: ProjectId, } | { "kind": "createSequence", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * Name of the sequence (not empty).
 */
name: string, 
/**
 * Its format; `null` uses the project's default format.
 */
format: Format | null, } | { "kind": "renameSequence", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The sequence.
 */
sequenceId: SequenceId, 
/**
 * The new name (not empty).
 */
name: string, } | { "kind": "setSequenceFormat", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The sequence.
 */
sequenceId: SequenceId, 
/**
 * Its new format.
 */
format: Format, 
/**
 * How its items adapt.
 */
adaptation: FormatAdaptation, } | { "kind": "duplicateSequence", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The sequence copied.
 */
sequenceId: SequenceId, 
/**
 * Name of the copy (not empty).
 */
name: string, 
/**
 * Format of the copy; `null` keeps the source's (same frame rate once it holds items).
 */
format: Format | null, 
/**
 * How the copied items adapt to the copy's format.
 */
adaptation: FormatAdaptation, } | { "kind": "removeSequence", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The sequence.
 */
sequenceId: SequenceId, } | { "kind": "undo", 
/**
 * The open project.
 */
projectId: ProjectId, } | { "kind": "redo", 
/**
 * The open project.
 */
projectId: ProjectId, } | { "kind": "addTrack", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The sequence receiving the track.
 */
sequenceId: SequenceId, 
/**
 * Video or audio (not `kind`: that is the command's tag).
 */
trackKind: TrackKind, 
/**
 * Position in the sequence's track list (video tracks stack in list order, first =
 * bottom); `null` appends.
 */
index: number | null, } | { "kind": "removeTrack", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The track.
 */
trackId: TrackId, } | { "kind": "updateTrack", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The track.
 */
trackId: TrackId, 
/**
 * The fields to change (absent or `null` fields stay).
 */
patch: TrackPatch, } | { "kind": "reorderTracks", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The sequence.
 */
sequenceId: SequenceId, 
/**
 * Every track of the sequence, once each, in the new order.
 */
order: Array<TrackId>, } | { "kind": "addMarker", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The sequence receiving the marker.
 */
sequenceId: SequenceId, 
/**
 * Its position (≥ 0).
 */
time: Flicks, 
/**
 * Its text (may be empty).
 */
label: string, 
/**
 * Its colour; `null` uses the default (green).
 */
color: MarkerColor | null, } | { "kind": "updateMarker", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The marker.
 */
markerId: MarkerId, 
/**
 * The fields to change (absent or `null` fields stay).
 */
patch: MarkerPatch, } | { "kind": "removeMarker", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The marker.
 */
markerId: MarkerId, } | { "kind": "placeMedia" } & PlaceMediaRequest | { "kind": "moveItems", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * Every item moved (one sequence), each once.
 */
moves: Array<ItemPlacement>, 
/**
 * What happens to items already where they land.
 */
mode: PlacementMode, } | { "kind": "duplicateItems", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The items copied (one sequence), each once, and where each copy goes.
 */
copies: Array<ItemPlacement>, 
/**
 * What happens to items already where the copies land.
 */
mode: PlacementMode, } | { "kind": "pasteItems", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The copies and their tracks (one sequence).
 */
items: Array<PastedItem>, 
/**
 * What happens to items already where the copies land.
 */
mode: PlacementMode, } | { "kind": "trimItems", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The edges moved, each item once.
 */
trims: Array<ItemTrim>, 
/**
 * Shift what follows by the change instead of leaving or filling a gap. A ripple trim
 * moves edges sitting at one instant (an item and its linked partners) to one place.
 */
ripple: boolean, } | { "kind": "splitItems", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * Items crossing `at`.
 */
itemIds: Array<ItemId>, 
/**
 * The cut instant.
 */
at: Flicks, } | { "kind": "deleteItems", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The items removed (one sequence), each once.
 */
itemIds: Array<ItemId>, 
/**
 * Close the time they took on their tracks (and on the tracks of items linked to what
 * follows) instead of leaving gaps.
 */
ripple: boolean, } | { "kind": "linkItems", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * At least two items of one sequence.
 */
itemIds: Array<ItemId>, } | { "kind": "unlinkItems", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The items unlinked.
 */
itemIds: Array<ItemId>, } | { "kind": "resizeGap", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The track of the gap.
 */
trackId: TrackId, 
/**
 * Where the gap starts (end of the previous item, or 0).
 */
start: Flicks, 
/**
 * Where it ends (start of the next item).
 */
end: Flicks, 
/**
 * Its new length (0 closes it).
 */
duration: Flicks, } | { "kind": "updateItems", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * One typed patch per item (its kind must match the item's), each item once.
 */
updates: Array<ItemUpdate>, 
/**
 * Properties set after the patches (may be omitted).
 */
animations?: Array<ItemAnimation>, } | { "kind": "setItemAnimation", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The item animated.
 */
itemId: ItemId, 
/**
 * The property and its new keyframes or value.
 */
animation: PropertyAnimation, } | { "kind": "setItemAnimations", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The new values, applied in order.
 */
animations: Array<ItemAnimation>, } | { "kind": "addGraphicItem" } & AddGraphicRequest | { "kind": "applyAnimationPreset", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The items animated, each once; each must have every property of the preset.
 */
itemIds: Array<ItemId>, 
/**
 * Id of the preset (`listAnimationPresets`).
 */
preset: string, 
/**
 * Entrance, exit or emphasis.
 */
phase: AnimationPhase, 
/**
 * How long it lasts (shortened to each item).
 */
duration: Flicks, 
/**
 * Start relative to each item; `null` for the phase's place.
 */
start: Flicks | null, } | { "kind": "listAnimationPresets" } | { "kind": "listFonts" } | { "kind": "addTransitions", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The cuts and the wanted transitions (kind from the track).
 */
transitions: Array<NewTransition>, } | { "kind": "updateTransition", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The transition changed.
 */
transitionId: TransitionId, 
/**
 * What changes.
 */
patch: TransitionPatch, } | { "kind": "removeTransitions", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The transitions removed, each once.
 */
transitionIds: Array<TransitionId>, } | { "kind": "importMedia" } & ImportMediaRequest | { "kind": "relinkAsset" } & RelinkAssetRequest | { "kind": "removeAsset", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The asset.
 */
assetId: AssetId, } | { "kind": "listJobs", 
/**
 * The open project.
 */
projectId: ProjectId, } | { "kind": "cancelJob", 
/**
 * The job.
 */
jobId: JobId, } | { "kind": "reportDecodability" } & DecodabilityReport | { "kind": "reportDecoderCapabilities" } & DecoderCapabilities | { "kind": "generateDerivedMedia" } & GenerateDerivedRequest | { "kind": "getMediaSettings" } | { "kind": "updateMediaSettings" } & MediaSettings | { "kind": "getMediaToolsStatus" } | { "kind": "getAudioSilences", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The asset.
 */
assetId: AssetId, } | { "kind": "getProjectCache", 
/**
 * The open project.
 */
projectId: ProjectId, } | { "kind": "clearProjectCache", 
/**
 * The open project.
 */
projectId: ProjectId, } | { "kind": "submitTask" } & SubmitTaskRequest | { "kind": "cancelTask", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The task.
 */
taskId: TaskId, } | { "kind": "retryTask", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The task.
 */
taskId: TaskId, } | { "kind": "followUpTask", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The task.
 */
taskId: TaskId, 
/**
 * What the agent must do next.
 */
prompt: string, 
/**
 * A new scope for this run and the next ones, on the task's sequence (« Refaire avec
 * l'IA » widens it to an item moved since); omitted to keep the task's scope.
 */
scope?: TaskScope, } | { "kind": "listTasks", 
/**
 * The open project.
 */
projectId: ProjectId, } | { "kind": "revertTask", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The task.
 */
taskId: TaskId, } | { "kind": "previewTaskRevert", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The task.
 */
taskId: TaskId, } | { "kind": "getTaskChanges", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The task.
 */
taskId: TaskId, } | { "kind": "getTaskEvents", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The task.
 */
taskId: TaskId, } | { "kind": "getAiStatus" } | { "kind": "refreshAiStatus" } | { "kind": "getAiSettings" } | { "kind": "updateAiSettings" } & AiSettings | { "kind": "reportFrameRenderFailure", 
/**
 * The request.
 */
requestId: string, 
/**
 * Why, for the agent (English, actionable).
 */
message: string, } | { "kind": "getTranscriptionStatus" } | { "kind": "updateTranscriptionPreferences" } & TranscriptionPreferences | { "kind": "startTranscriptionDownload", 
/**
 * What to download.
 */
target: DownloadTarget, } | { "kind": "cancelTranscriptionDownload", 
/**
 * The download.
 */
target: DownloadTarget, } | { "kind": "importTranscriptionModel", 
/**
 * Absolute path of the `.bin` file.
 */
path: string, } | { "kind": "removeTranscriptionFile", 
/**
 * What to delete.
 */
target: DownloadTarget, } | { "kind": "transcribeAssets", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * Assets with sound.
 */
assetIds: Array<AssetId>, } | { "kind": "getTranscript", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The asset.
 */
assetId: AssetId, } | { "kind": "searchTranscripts", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * Words to find.
 */
query: string, } | { "kind": "setTranscriptionSettings", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The new settings.
 */
settings: TranscriptionSettings, } | { "kind": "getSmartCutSettings" } | { "kind": "updateSmartCutSettings" } & SmartCutSettings | { "kind": "removeWords", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The sequence edited.
 */
sequenceId: SequenceId, 
/**
 * Ranges holding the words to remove (usually from the start of the first word to the
 * end of the last one).
 */
ranges: Array<TimeRange>, 
/**
 * Words or fillers (the undo label).
 */
purpose: WordCutPurpose, 
/**
 * Cut every unlocked track (graphics included) instead of the speech's tracks and their
 * linked partners.
 */
allTracks: boolean, } | { "kind": "findFillers", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The sequence read.
 */
sequenceId: SequenceId, 
/**
 * Where to look; `null` for the whole sequence.
 */
range: TimeRange | null, } | { "kind": "previewSilenceCuts", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The sequence read.
 */
sequenceId: SequenceId, 
/**
 * Where to look; `null` for the whole sequence.
 */
range: TimeRange | null, 
/**
 * Threshold, minimum silence and padding.
 */
params: SilenceCutParams, 
/**
 * Sound tracks whose silences count (all of them must be silent); `null` for every
 * unmuted sound track holding media.
 */
trackIds: Array<TrackId> | null, 
/**
 * Cut every unlocked track.
 */
allTracks: boolean, } | { "kind": "removeSilences", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The sequence edited.
 */
sequenceId: SequenceId, 
/**
 * Where to cut; `null` for the whole sequence.
 */
range: TimeRange | null, 
/**
 * Threshold, minimum silence and padding.
 */
params: SilenceCutParams, 
/**
 * Sound tracks whose silences count; `null` for every unmuted sound track holding
 * media.
 */
trackIds: Array<TrackId> | null, 
/**
 * Cut every unlocked track.
 */
allTracks: boolean, } | { "kind": "generateCaptions" } & GenerateCaptionsRequest | { "kind": "importSubtitles" } & ImportSubtitlesRequest | { "kind": "exportSubtitles", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The sequence read.
 */
sequenceId: SequenceId, 
/**
 * Absolute path of the `.srt` file (replaced when it exists).
 */
path: string, } | { "kind": "editCaptionText" } & CaptionTextEdit | { "kind": "mergeCaptions", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * Two or more captions following each other on one track.
 */
itemIds: Array<ItemId>, } | { "kind": "splitCaption", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The caption.
 */
itemId: ItemId, 
/**
 * The first word of the right part (1 to the number of words − 1).
 */
wordIndex: number, } | { "kind": "shiftCaptions", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The captions (every caption of a track for a global shift).
 */
itemIds: Array<ItemId>, 
/**
 * How far (negative: earlier).
 */
delta: Flicks, } | { "kind": "setCaptionEmphasis" } & CaptionEmphasisEdit | { "kind": "updateCaptionStyle" } & CaptionStyleEdit | { "kind": "applyCaptionPreset", 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The captions.
 */
itemIds: Array<ItemId>, 
/**
 * The preset's id (`tiktokBold`, `minimal`, `karaoke`, `box`).
 */
preset: string, };
