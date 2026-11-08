// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AiSettings } from "./AiSettings";
import type { AiStatus } from "./AiStatus";
import type { AnimationPresetInfo } from "./AnimationPresetInfo";
import type { CoreInfo } from "./CoreInfo";
import type { FontFamily } from "./FontFamily";
import type { ImportReport } from "./ImportReport";
import type { ItemId } from "./ItemId";
import type { JobSnapshot } from "./JobSnapshot";
import type { LinkGroupId } from "./LinkGroupId";
import type { MarkerId } from "./MarkerId";
import type { MediaSettings } from "./MediaSettings";
import type { MediaToolsStatus } from "./MediaToolsStatus";
import type { NewProjectDefaults } from "./NewProjectDefaults";
import type { PlacedGraphic } from "./PlacedGraphic";
import type { ProjectCacheInfo } from "./ProjectCacheInfo";
import type { ProjectSession } from "./ProjectSession";
import type { RecentProject } from "./RecentProject";
import type { RelinkOutcome } from "./RelinkOutcome";
import type { SequenceId } from "./SequenceId";
import type { Task } from "./Task";
import type { TaskEventRecord } from "./TaskEventRecord";
import type { TimeRange } from "./TimeRange";
import type { TrackId } from "./TrackId";
import type { TransitionId } from "./TransitionId";

/**
 * The result of a [`crate::Command`], tagged by the **same `kind` as the command** it answers.
 */
export type CommandResult = { "kind": "ping" } | { "kind": "getCoreInfo" } & CoreInfo | { "kind": "getNewProjectDefaults" } & NewProjectDefaults | { "kind": "createProject" } & ProjectSession | { "kind": "openProject" } & ProjectSession | { "kind": "closeProject" } | { "kind": "listRecentProjects", 
/**
 * Most recently opened first.
 */
projects: Array<RecentProject>, } | { "kind": "renameProject" } & RecentProject | { "kind": "duplicateProject" } & RecentProject | { "kind": "trashProject" } | { "kind": "getProjectSnapshot" } & ProjectSession | { "kind": "createSequence", 
/**
 * Id of the new sequence.
 */
sequenceId: SequenceId, } | { "kind": "renameSequence" } | { "kind": "setSequenceFormat" } | { "kind": "duplicateSequence", 
/**
 * Id of the copy.
 */
sequenceId: SequenceId, } | { "kind": "removeSequence" } | { "kind": "undo" } | { "kind": "redo" } | { "kind": "addTrack", 
/**
 * Id of the new track.
 */
trackId: TrackId, } | { "kind": "removeTrack" } | { "kind": "updateTrack" } | { "kind": "reorderTracks" } | { "kind": "addMarker", 
/**
 * Id of the new marker.
 */
markerId: MarkerId, } | { "kind": "updateMarker" } | { "kind": "removeMarker" } | { "kind": "placeMedia", 
/**
 * Ids of the new items, in placement order (picture before sound for each asset).
 */
itemIds: Array<ItemId>, } | { "kind": "moveItems" } | { "kind": "duplicateItems", 
/**
 * Ids of the copies, in the order of the request.
 */
itemIds: Array<ItemId>, } | { "kind": "pasteItems", 
/**
 * Ids of the copies, in the order of the request.
 */
itemIds: Array<ItemId>, } | { "kind": "trimItems" } | { "kind": "splitItems", 
/**
 * Ids of the new right parts, in the order of the request.
 */
itemIds: Array<ItemId>, } | { "kind": "deleteItems" } | { "kind": "linkItems", 
/**
 * The new link group.
 */
linkGroup: LinkGroupId, } | { "kind": "unlinkItems" } | { "kind": "resizeGap" } | { "kind": "updateItems" } | { "kind": "setItemAnimation" } | { "kind": "addTransitions", 
/**
 * Ids of the transitions added, in the order of the request (cuts without room left
 * out).
 */
transitionIds: Array<TransitionId>, } | { "kind": "updateTransition" } | { "kind": "removeTransitions" } | { "kind": "setItemAnimations" } | { "kind": "addGraphicItem" } & PlacedGraphic | { "kind": "applyAnimationPreset" } | { "kind": "listAnimationPresets", 
/**
 * Every preset, in display order.
 */
presets: Array<AnimationPresetInfo>, } | { "kind": "listFonts", 
/**
 * Bundled families first, then installed ones, each sorted by name.
 */
families: Array<FontFamily>, } | { "kind": "importMedia" } & ImportReport | { "kind": "relinkAsset", 
/**
 * Whether the asset was relinked or the user must confirm first.
 */
outcome: RelinkOutcome, } | { "kind": "removeAsset" } | { "kind": "listJobs", 
/**
 * Oldest first.
 */
jobs: Array<JobSnapshot>, } | { "kind": "cancelJob" } | { "kind": "reportDecodability" } | { "kind": "reportDecoderCapabilities" } | { "kind": "generateDerivedMedia" } | { "kind": "getMediaSettings", 
/**
 * The current settings.
 */
settings: MediaSettings, } | { "kind": "updateMediaSettings", 
/**
 * The settings as saved.
 */
settings: MediaSettings, } | { "kind": "getMediaToolsStatus", 
/**
 * Tools found and hardware encoders.
 */
status: MediaToolsStatus, } | { "kind": "getAudioSilences", 
/**
 * The silences in source time, in order; `null` while the sound is not analysed.
 */
silences: Array<TimeRange> | null, } | { "kind": "getProjectCache", 
/**
 * The cache folder and its size.
 */
cache: ProjectCacheInfo, } | { "kind": "clearProjectCache", 
/**
 * The cache folder and what is left in it (files in use cannot be deleted).
 */
cache: ProjectCacheInfo, } | { "kind": "submitTask", 
/**
 * The new task.
 */
task: Task, } | { "kind": "cancelTask" } | { "kind": "retryTask", 
/**
 * The task.
 */
task: Task, } | { "kind": "followUpTask", 
/**
 * The task.
 */
task: Task, } | { "kind": "listTasks", 
/**
 * Oldest first.
 */
tasks: Array<Task>, } | { "kind": "getTaskEvents", 
/**
 * The task's log, in order.
 */
events: Array<TaskEventRecord>, } | { "kind": "getAiStatus", 
/**
 * The provider in use.
 */
status: AiStatus, } | { "kind": "refreshAiStatus", 
/**
 * The provider in use, checked now.
 */
status: AiStatus, } | { "kind": "getAiSettings", 
/**
 * The current settings.
 */
settings: AiSettings, } | { "kind": "updateAiSettings", 
/**
 * The settings as saved.
 */
settings: AiSettings, } | { "kind": "reportFrameRenderFailure" };
