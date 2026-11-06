// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol).
// Regenerate with `pnpm codegen`.
import type { CommandResult } from "./CommandResult";
import type { ProtocolError } from "./ProtocolError";

/**
 * The answer to a request: either a result or an error, never both.
 */
export type ResponseEnvelope = { 
/**
 * Id of the request being answered, or `None` when the request could not be read at all.
 */
id: number | null, } & ({ "result": CommandResult } | { "error": ProtocolError });
