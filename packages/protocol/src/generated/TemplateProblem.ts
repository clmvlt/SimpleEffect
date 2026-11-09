// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";
import type { TemplateCheckStage } from "./TemplateCheckStage";

/**
 * One problem of a template module, read by the agent that wrote it.
 */
export type TemplateProblem = { 
/**
 * When it was found.
 */
stage: TemplateCheckStage, 
/**
 * What is wrong and how to fix it.
 */
message: string, 
/**
 * Line in the module (1-based), when known.
 */
line: number | null, 
/**
 * Column in that line (1-based), when known.
 */
column: number | null, 
/**
 * Instant being rendered (relative to the instance's start), for render problems.
 */
time: Flicks | null, };
