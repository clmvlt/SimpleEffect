// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { CreateProjectRequest } from "./CreateProjectRequest";
import type { DecodabilityReport } from "./DecodabilityReport";
import type { DecoderCapabilities } from "./DecoderCapabilities";
import type { Flicks } from "./Flicks";
import type { Format } from "./Format";
import type { GenerateDerivedRequest } from "./GenerateDerivedRequest";
import type { ImportMediaRequest } from "./ImportMediaRequest";
import type { JobId } from "./JobId";
import type { MarkerColor } from "./MarkerColor";
import type { MarkerId } from "./MarkerId";
import type { MarkerPatch } from "./MarkerPatch";
import type { MediaSettings } from "./MediaSettings";
import type { ProjectId } from "./ProjectId";
import type { RelinkAssetRequest } from "./RelinkAssetRequest";
import type { SequenceId } from "./SequenceId";
import type { TrackId } from "./TrackId";
import type { TrackKind } from "./TrackKind";
import type { TrackPatch } from "./TrackPatch";

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
format: Format | null, } | { "kind": "undo", 
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
markerId: MarkerId, } | { "kind": "importMedia" } & ImportMediaRequest | { "kind": "relinkAsset" } & RelinkAssetRequest | { "kind": "removeAsset", 
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
jobId: JobId, } | { "kind": "reportDecodability" } & DecodabilityReport | { "kind": "reportDecoderCapabilities" } & DecoderCapabilities | { "kind": "generateDerivedMedia" } & GenerateDerivedRequest | { "kind": "getMediaSettings" } | { "kind": "updateMediaSettings" } & MediaSettings | { "kind": "getMediaToolsStatus" } | { "kind": "getProjectCache", 
/**
 * The open project.
 */
projectId: ProjectId, } | { "kind": "clearProjectCache", 
/**
 * The open project.
 */
projectId: ProjectId, };
