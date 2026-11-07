// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { CoreInfo } from "./CoreInfo";
import type { ImportReport } from "./ImportReport";
import type { ItemId } from "./ItemId";
import type { JobSnapshot } from "./JobSnapshot";
import type { LinkGroupId } from "./LinkGroupId";
import type { MarkerId } from "./MarkerId";
import type { MediaSettings } from "./MediaSettings";
import type { MediaToolsStatus } from "./MediaToolsStatus";
import type { NewProjectDefaults } from "./NewProjectDefaults";
import type { ProjectCacheInfo } from "./ProjectCacheInfo";
import type { ProjectSession } from "./ProjectSession";
import type { RecentProject } from "./RecentProject";
import type { RelinkOutcome } from "./RelinkOutcome";
import type { SequenceId } from "./SequenceId";
import type { TrackId } from "./TrackId";

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
sequenceId: SequenceId, } | { "kind": "undo" } | { "kind": "redo" } | { "kind": "addTrack", 
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
linkGroup: LinkGroupId, } | { "kind": "unlinkItems" } | { "kind": "resizeGap" } | { "kind": "importMedia" } & ImportReport | { "kind": "relinkAsset", 
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
status: MediaToolsStatus, } | { "kind": "getProjectCache", 
/**
 * The cache folder and its size.
 */
cache: ProjectCacheInfo, } | { "kind": "clearProjectCache", 
/**
 * The cache folder and what is left in it (files in use cannot be deleted).
 */
cache: ProjectCacheInfo, };
