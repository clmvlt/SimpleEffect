// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Keyframe } from "./Keyframe";

/**
 * A property that is either constant or animated by keyframes.
 *
 * Interpolation between keyframes lives in `@se/engine` (prompt 12), so preview and export share
 * it; the domain only stores and validates the keyframes. A keyframed property has at least one
 * keyframe, sorted by strictly increasing time, all within the item's duration.
 *
 * Reading also accepts a bare value (`72` for `{"kind": "static", "value": 72}`): properties that
 * became animatable (prompt 15: font size, text colour, shape fill) keep reading journals written
 * before, without a journal migration. Values are always written tagged.
 */
export type Animatable<T> = { "kind": "static", 
/**
 * The value.
 */
value: T, } | { "kind": "keyframed", 
/**
 * Keyframes sorted by strictly increasing `time`.
 */
keyframes: Array<Keyframe<T>>, };
