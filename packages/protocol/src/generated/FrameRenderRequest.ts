// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";
import type { ProjectId } from "./ProjectId";
import type { SequenceId } from "./SequenceId";

/**
 * Frames of a sequence the renderer must draw and upload.
 */
export type FrameRenderRequest = { 
/**
 * Identifies the request in the upload route and the failure report (a UUID).
 */
requestId: string, 
/**
 * The project (the renderer draws only the project it has open).
 */
projectId: ProjectId, 
/**
 * The sequence to draw.
 */
sequenceId: SequenceId, 
/**
 * Sequence times to draw, in order; frame `i` is uploaded at index `i`.
 */
times: Array<Flicks>, 
/**
 * Width of the pictures, in pixels.
 */
width: number, 
/**
 * Height of the pictures, in pixels (the sequence's aspect).
 */
height: number, };
