// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ExportId } from "./ExportId";
import type { ExportPipeline } from "./ExportPipeline";
import type { ExportSettings } from "./ExportSettings";
import type { Flicks } from "./Flicks";
import type { ProjectId } from "./ProjectId";
import type { ProjectSnapshot } from "./ProjectSnapshot";
import type { SequenceId } from "./SequenceId";

/**
 * What the renderer must render for an export (event `exportRenderRequested`).
 */
export type ExportRenderRequest = { 
/**
 * The export (upload routes, progress and outcome reports).
 */
exportId: ExportId, 
/**
 * Its project.
 */
projectId: ProjectId, 
/**
 * The exported sequence.
 */
sequenceId: SequenceId, 
/**
 * The sequence as it was when the export started: that sequence, its tracks and items, the
 * media they use and the project's header (templates, caption styles). Later edits never
 * reach the export.
 */
snapshot: ProjectSnapshot, 
/**
 * What to write.
 */
settings: ExportSettings, 
/**
 * Sequence time of the first frame.
 */
start: Flicks, 
/**
 * Sequence time where the export ends (exclusive).
 */
end: Flicks, 
/**
 * How the file is put together.
 */
pipeline: ExportPipeline, };
