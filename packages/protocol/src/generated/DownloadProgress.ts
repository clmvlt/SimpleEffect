// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * A download or installation in progress (or failed) of a model or of the GPU pack.
 */
export type DownloadProgress = { "kind": "downloading", 
/**
 * Bytes received so far.
 */
received: number, 
/**
 * Bytes expected.
 */
total: number, } | { "kind": "verifying" } | { "kind": "failed", 
/**
 * Why, and what to do.
 */
message: string, };
