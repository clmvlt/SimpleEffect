// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol).
// Regenerate with `pnpm codegen`.
import type { RequestEnvelope } from "./RequestEnvelope";

/**
 * A message sent by a client to `se-core`.
 */
export type ClientMessage = { "type": "request" } & RequestEnvelope;
