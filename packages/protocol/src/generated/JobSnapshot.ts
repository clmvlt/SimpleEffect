// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { JobId } from "./JobId";
import type { JobKind } from "./JobKind";
import type { JobState } from "./JobState";
import type { ProjectId } from "./ProjectId";

/**
 * A job as clients see it.
 */
export type JobSnapshot = { 
/**
 * Stable id.
 */
id: JobId, 
/**
 * The project it works for.
 */
projectId: ProjectId, 
/**
 * The asset it works on, if any.
 */
assetId: AssetId | null, 
/**
 * What it produces.
 */
kind: JobKind, 
/**
 * Where it stands.
 */
state: JobState, 
/**
 * Attempt number, from 1 (2 for the automatic retry).
 */
attempt: number, };
