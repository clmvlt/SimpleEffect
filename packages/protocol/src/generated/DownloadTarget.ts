// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * What a download is about.
 */
export type DownloadTarget = { "kind": "model", 
/**
 * Its id (`base`, `large-v3-turbo`…).
 */
modelId: string, } | { "kind": "voiceModel" } | { "kind": "gpuRuntime" };
