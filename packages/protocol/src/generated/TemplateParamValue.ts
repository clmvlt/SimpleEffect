// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Color } from "./Color";

/**
 * The value of one template parameter.
 *
 * TODO(prompt-24): enum, font and asset parameters (ADR-0011), validated against the schema.
 */
export type TemplateParamValue = { "kind": "text", 
/**
 * The text.
 */
value: string, } | { "kind": "number", 
/**
 * The number.
 */
value: number, } | { "kind": "boolean", 
/**
 * The flag.
 */
value: boolean, } | { "kind": "color", 
/**
 * The colour.
 */
value: Color, };
