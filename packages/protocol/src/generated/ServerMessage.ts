// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol).
// Regenerate with `pnpm codegen`.
import type { EventEnvelope } from "./EventEnvelope";
import type { ResponseEnvelope } from "./ResponseEnvelope";

/**
 * A message sent by `se-core` to a client.
 */
export type ServerMessage = { "type": "response" } & ResponseEnvelope | { "type": "event" } & EventEnvelope;
