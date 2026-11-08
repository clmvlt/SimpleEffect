// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { EffortLevel } from "./EffortLevel";
import type { ModelOption } from "./ModelOption";

/**
 * What a provider supports.
 */
export type ProviderCapabilities = { 
/**
 * Whether a finished run's session can be resumed (follow-ups).
 */
resumeSessions: boolean, 
/**
 * Whether runs accept images (frames sent by MCP tools).
 */
imageInput: boolean, 
/**
 * Models a user can pick; empty when the CLI decides alone.
 */
models: Array<ModelOption>, 
/**
 * Effort levels a user can pick; empty when not supported.
 */
effortLevels: Array<EffortLevel>, };
