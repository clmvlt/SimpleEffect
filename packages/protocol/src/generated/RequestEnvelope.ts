// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Command } from "./Command";

/**
 * A command to execute, with the id its response will carry.
 */
export type RequestEnvelope = { 
/**
 * Echoed back in the response. Unique per client connection lifetime.
 */
id: number, 
/**
 * The command to execute.
 */
command: Command, };
