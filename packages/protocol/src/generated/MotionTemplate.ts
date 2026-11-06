// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { TemplateId } from "./TemplateId";

/**
 * An AI-authored, parameterised render module stored in the project.
 *
 * TODO(prompt-24): source code, parameter schema, version and validation status.
 */
export type MotionTemplate = { 
/**
 * Stable id, referenced by template items.
 */
id: TemplateId, 
/**
 * Name shown in the library and the inspector.
 */
name: string, };
