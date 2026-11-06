# se-codegen

## Purpose
Development tool behind `pnpm codegen`: exports every Rust type deriving `ts_rs::TS` reachable from the protocol and document roots to TypeScript, one file per type plus an `index.ts`, each starting with a `GENERATED — DO NOT EDIT` header (ADR-0010, ADR-0012). Not shipped with the app; nothing depends on it.

## Public API
- `se-codegen <output directory>` (the directory must be empty or absent; messages go to stderr, exit code 1 on failure).
- Normally run through `scripts/codegen.mjs`: it generates into `target/codegen/generated/`, then replaces `packages/protocol/src/generated/` (`pnpm codegen`) or compares the two and fails on any difference (`pnpm codegen --check`, part of `pnpm check`).

## Rules and invariants
- Root types live in `bindings::export_roots`: `ClientMessage`, `ServerMessage`, `CoreHandshake` (protocol) and `Project`, `Violation`, `FormatPreset` (se-domain). Types referenced by a root are exported automatically: add a root only for a type nothing reachable references.
- `i64`/`u64` map to `number` (`LARGE_INT_TYPE`): flicks travel as JSON numbers (ADR-0004). `Flicks` itself is a branded `number` and ids are `string` (container-level `#[ts(type = …)]` in se-domain, tested here).
- Output must be deterministic (tested): the freshness check compares bytes (after CRLF normalisation).
- If a ts-rs upgrade changes the first-line note, `replace_header` fails with `UnexpectedHeader`: update `TS_RS_NOTE_PREFIX`.

## Allowed dependencies
`se-protocol`, `se-domain`, `ts-rs`, `thiserror`. Dev: `tempfile`.

## How to test
- `cargo test -p se-codegen` (generation into temporary folders: headers, index, `number` for large ints, branded flicks and string ids, determinism, refusal of a non-empty folder).
- `pnpm codegen --check` after changing protocol or document types.
