// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Color } from "./Color";
import type { TemplateChoice } from "./TemplateChoice";

/**
 * The values a parameter takes, with its default.
 */
export type TemplateParamSpec = { "kind": "string", 
/**
 * Value when the item sets none.
 */
default: string, 
/**
 * Whether line breaks are expected (a text area in the inspector).
 */
multiline: boolean, 
/**
 * Longest value, in characters.
 */
maxLength: number, } | { "kind": "number", 
/**
 * Value when the item sets none.
 */
default: number, 
/**
 * Smallest value allowed, if any.
 */
minimum: number | null, 
/**
 * Largest value allowed, if any.
 */
maximum: number | null, 
/**
 * Step of the inspector's control, if any (> 0).
 */
step: number | null, } | { "kind": "color", 
/**
 * Value when the item sets none.
 */
default: Color, } | { "kind": "boolean", 
/**
 * Value when the item sets none.
 */
default: boolean, } | { "kind": "enum", 
/**
 * Value when the item sets none (one of the options).
 */
default: string, 
/**
 * The options, in order.
 */
options: Array<TemplateChoice>, } | { "kind": "font", 
/**
 * Family when the item sets none.
 */
default: string, } | { "kind": "asset" };
