// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * A quick identity of a file's content, computed by se-media at import (ADR-0018): the size,
 * the modification time and a 64-bit xxh3 hash of the first and last mebibyte.
 *
 * Two files with the same size and sample hash are taken for the same content (deduplication at
 * import, relinking); the modification time is part of [`ContentFingerprint::cache_key`] only, so
 * a file rewritten in place gets fresh thumbnails and peaks.
 */
export type ContentFingerprint = { sizeBytes: number, modifiedUnixMs: number, sampleHash: string, };
