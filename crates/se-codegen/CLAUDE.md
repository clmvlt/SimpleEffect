# se-codegen

## Purpose
Development tool behind `pnpm codegen`: exports every Rust type deriving `ts_rs::TS` reachable from the protocol roots to TypeScript, one file per type plus an `index.ts`, each starting with a `GENERATED — DO NOT EDIT` header (ADR-0010, ADR-0012). Not shipped with the app; nothing depends on it.

## Public API
- `se-codegen <output directory>` (the directory must be empty or absent; messages go to stderr, exit code 1 on failure).
- Normally run through `scripts/codegen.mjs`: it generates into `target/codegen/generated/`, then replaces `packages/protocol/src/generated/` (`pnpm codegen`) or compares the two and fails on any difference (`pnpm codegen --check`, part of `pnpm check`).

## Rules and invariants
- Root types live in `bindings::export_roots` (`ClientMessage`, `ServerMessage`, `CoreHandshake`). Types referenced by a root are exported automatically: add a root only for a type no envelope references (e.g. a `se-domain` type the renderer needs on its own).
- `i64`/`u64` map to `number` (`LARGE_INT_TYPE`): flicks travel as JSON numbers (ADR-0004).
- Output must be deterministic (tested): the freshness check compares bytes (after CRLF normalisation).
- If a ts-rs upgrade changes the first-line note, `replace_header` fails with `UnexpectedHeader`: update `TS_RS_NOTE_PREFIX`.

## Allowed dependencies
`se-protocol` (later `se-domain`), `ts-rs`, `thiserror`. Dev: `tempfile`.

## How to test
- `cargo test -p se-codegen` (generation into temporary folders: headers, index, `number` for large ints, determinism, refusal of a non-empty folder).
- `pnpm codegen --check` after changing protocol types.
