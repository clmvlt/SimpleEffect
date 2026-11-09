// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ProjectId } from "./ProjectId";

/**
 * A template module the renderer must check.
 */
export type TemplateCheckRequest = { 
/**
 * Identifies the request in the report (a UUID).
 */
requestId: string, 
/**
 * The project the template is for (its fonts and images).
 */
projectId: ProjectId, 
/**
 * The TypeScript module.
 */
source: string, 
/**
 * Frame the template draws in, in design pixels (the task sequence's aspect, shorter side
 * 1080).
 */
width: number, 
/**
 * Height of that frame.
 */
height: number, 
/**
 * Width of the pictures returned, in pixels.
 */
pictureWidth: number, 
/**
 * Height of the pictures returned, in pixels.
 */
pictureHeight: number, };
