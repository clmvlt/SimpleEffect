// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Animatable } from "./Animatable";
import type { AssetId } from "./AssetId";
import type { Color } from "./Color";

/**
 * The value of one template parameter; its kind matches the parameter's (validation).
 *
 * Numbers and colours are animatable (keyframe times relative to the item's start, like every
 * keyframe). Reading accepts a bare number or colour as a static value (documents written before
 * prompt 24).
 */
export type TemplateParamValue = { "kind": "text", 
/**
 * The text.
 */
value: string, } | { "kind": "number", 
/**
 * The number, possibly keyframed.
 */
value: Animatable<number>, } | { "kind": "boolean", 
/**
 * The flag.
 */
value: boolean, } | { "kind": "color", 
/**
 * The colour, possibly keyframed.
 */
value: Animatable<Color>, } | { "kind": "enum", 
/**
 * The option chosen.
 */
value: string, } | { "kind": "font", 
/**
 * The family name.
 */
value: string, } | { "kind": "asset", 
/**
 * The asset, `null` for none.
 */
value: AssetId | null, };
