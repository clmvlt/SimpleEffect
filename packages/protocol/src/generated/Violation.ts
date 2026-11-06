// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ViolationCode } from "./ViolationCode";

/**
 * A broken document invariant, reported to users and AI agents.
 */
export type Violation = { 
/**
 * Stable, matchable code.
 */
code: ViolationCode, 
/**
 * Where, in JSON field names with element ids:
 * `sequences[<id>].tracks[<id>].items[<id>].kind.speed`; keyframes use their index.
 */
path: string, 
/**
 * What is wrong and how to fix it.
 */
message: string, };
