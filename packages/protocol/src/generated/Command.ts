// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * A command sent inside a [`crate::RequestEnvelope`], tagged by `kind` (camelCase).
 *
 * Every variant has a [`crate::CommandResult`] variant **with the same name**: that pairing is
 * how the TypeScript client derives the result type of each command without a hand-written table
 * (ADR-0012). Adding a command: see the recipe in `crates/se-protocol/CLAUDE.md`.
 */
export type Command = { "kind": "ping" } | { "kind": "getCoreInfo" };
