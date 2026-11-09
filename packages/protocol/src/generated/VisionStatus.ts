// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * Whether the face detector can run here (`getVisionStatus`).
 */
export type VisionStatus = { 
/**
 * Detection can run.
 */
available: boolean, 
/**
 * The detection model (`yunet-2026may`), bundled with the app.
 */
model: string, 
/**
 * The inference library and its version (`ONNX Runtime 1.31.0`) once loaded.
 */
runtime: string | null, 
/**
 * Where inference runs (`CPU`, `CoreML`) once loaded.
 */
accelerator: string | null, 
/**
 * Why detection cannot run (missing library, Intel Mac…), when it cannot.
 */
reason: string | null, };
