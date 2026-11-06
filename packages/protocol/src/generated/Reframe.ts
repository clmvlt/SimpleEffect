// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * How a clip's picture fits a sequence frame of another aspect ratio.
 *
 * TODO(prompt-26): automatic reframing (subject tracking) adds a variant.
 */
export type Reframe = { "kind": "fit" } | { "kind": "fill" };
