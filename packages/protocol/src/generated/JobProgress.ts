// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { JobId } from "./JobId";
import type { ProjectId } from "./ProjectId";

/**
 * Progress of a running job, pushed by [`crate::CoreEvent::JobProgress`].
 */
export type JobProgress = { 
/**
 * The job.
 */
jobId: JobId, 
/**
 * Its project.
 */
projectId: ProjectId, 
/**
 * Its asset, if any.
 */
assetId: AssetId | null, 
/**
 * Fraction done in `[0, 1]`, when the job can tell.
 */
progress: number | null, };
