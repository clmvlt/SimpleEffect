// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * The curve of a clip fade, applied to its sound gain and to its picture opacity.
 *
 * The curves themselves live in `@se/engine` (`fadeGain`), the single evaluator shared by
 * preview and export; the domain only names them.
 */
export type FadeShape = "linear" | "constantPower" | "logarithmic";
