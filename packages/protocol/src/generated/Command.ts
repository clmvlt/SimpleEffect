// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { CreateProjectRequest } from "./CreateProjectRequest";
import type { DecodabilityReport } from "./DecodabilityReport";
import type { Format } from "./Format";
import type { ImportMediaRequest } from "./ImportMediaRequest";
import type { JobId } from "./JobId";
import type { ProjectId } from "./ProjectId";
import type { RelinkAssetRequest } from "./RelinkAssetRequest";

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
projectId: ProjectId, } | { "kind": "importMedia" } & ImportMediaRequest | { "kind": "relinkAsset" } & RelinkAssetRequest | { "kind": "removeAsset", 
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
jobId: JobId, } | { "kind": "reportDecodability" } & DecodabilityReport;
