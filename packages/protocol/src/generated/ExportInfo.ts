// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ExportId } from "./ExportId";
import type { ExportSettings } from "./ExportSettings";
import type { ExportState } from "./ExportState";
import type { Flicks } from "./Flicks";
import type { JobId } from "./JobId";
import type { ProjectId } from "./ProjectId";
import type { SequenceId } from "./SequenceId";

/**
 * An export as clients see it.
 */
export type ExportInfo = { 
/**
 * Stable id.
 */
id: ExportId, 
/**
 * Its job in the core's job queue (`null` only while it is being queued).
 */
jobId: JobId | null, 
/**
 * Its project.
 */
projectId: ProjectId, 
/**
 * The exported sequence.
 */
sequenceId: SequenceId, 
/**
 * The sequence's name when the export started.
 */
sequenceName: string, 
/**
 * The preset it was started with.
 */
presetName: string, 
/**
 * Absolute path of the file.
 */
path: string, 
/**
 * What it writes.
 */
settings: ExportSettings, 
/**
 * Duration of the exported span.
 */
duration: Flicks, 
/**
 * When it was started (Unix milliseconds).
 */
createdAt: number, 
/**
 * Whether a picture of it is served at `/media/exports/{id}/thumbnail`.
 */
hasThumbnail: boolean, 
/**
 * Where it stands.
 */
state: ExportState, };
