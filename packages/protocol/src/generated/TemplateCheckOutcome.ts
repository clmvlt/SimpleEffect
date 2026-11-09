// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { TemplateCheckFrame } from "./TemplateCheckFrame";
import type { TemplateProblem } from "./TemplateProblem";

/**
 * What the renderer found.
 */
export type TemplateCheckOutcome = { "kind": "passed", 
/**
 * `JSON.stringify` of the `meta` export.
 */
metaJson: string, 
/**
 * The `params` export as JSON `[name, schema]` pairs in declaration order (a whole
 * object schema is unwrapped first).
 */
paramsJson: string, 
/**
 * Pictures of the entrance, middle and exit, rendered with the default values.
 */
frames: Array<TemplateCheckFrame>, 
/**
 * Longest render of one frame during the check, in milliseconds.
 */
slowestFrameMs: number, } | { "kind": "failed", 
/**
 * Every problem found, in the order met.
 */
problems: Array<TemplateProblem>, };
