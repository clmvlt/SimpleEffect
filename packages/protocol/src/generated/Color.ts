// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * An 8-bit sRGB colour with straight (non-premultiplied) alpha.
 *
 * Serialized as `#rrggbb` when opaque and `#rrggbbaa` otherwise, in lowercase; parsing accepts
 * either form in any letter case. This is how users, agents and the UI colour fields write it.
 */
export type Color = string;
