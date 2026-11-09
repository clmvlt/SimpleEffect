// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { TemplateParamSpec } from "./TemplateParamSpec";

/**
 * One parameter of a template: a name the code reads (`params.title`), a label for the
 * inspector, and what values it takes.
 */
export type TemplateParam = { 
/**
 * The key in `params` (an identifier: letters, digits, `_`, not starting with a digit).
 */
name: string, 
/**
 * What the inspector shows (`title` in the schema; the name when absent).
 */
label: string, 
/**
 * A longer explanation (tooltip), if any.
 */
description: string | null, 
/**
 * The kind of values and their constraints.
 */
spec: TemplateParamSpec, };
