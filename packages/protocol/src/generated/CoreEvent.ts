// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * An event pushed inside a [`crate::EventEnvelope`], tagged by `kind` (camelCase).
 *
 * Named `CoreEvent` rather than `Event` so the generated TypeScript type does not shadow the DOM
 * `Event` type in the renderer.
 */
export type CoreEvent = { "kind": "coreReady", 
/**
 * Version of the running `se-core` binary.
 */
version: string, } | { "kind": "coreShuttingDown" } | { "kind": "resyncRequired", 
/**
 * Number of events that were dropped.
 */
missedEvents: number, };
