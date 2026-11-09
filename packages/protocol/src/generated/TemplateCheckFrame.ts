// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";

/**
 * One picture of a checked template.
 */
export type TemplateCheckFrame = { 
/**
 * Instant drawn, relative to the instance's start.
 */
time: Flicks, 
/**
 * The picture: a JPEG, base64-encoded.
 */
jpegBase64: string, };
