// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";
import type { SubjectFrame } from "./SubjectFrame";

/**
 * Full result of a subject detection (stored in the project database): pictures sampled at a
 * regular interval over the whole asset, each with the faces it shows.
 */
export type SubjectDetection = { 
/**
 * The detector that found the faces (`yunet-2026may`): a result of another model is run
 * again.
 */
model: string, 
/**
 * Time between two samples.
 */
interval: Flicks, 
/**
 * The samples, by increasing source time.
 */
frames: Array<SubjectFrame>, };
