// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * Whether the original can be decoded in the renderer (checked with `isConfigSupported`).
 *
 * TODO(prompt-08): refine per stream if a file can have a decodable video and an undecodable
 * audio stream.
 */
export type Decodability = "unknown" | "decodable" | "needsMezzanine";
