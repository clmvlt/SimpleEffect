# 0012 — RPC protocol conventions: codegen binary, command/result pairing, event delivery

- Status: Accepted
- Date: 2026-11-06

## Context
ADR-0010 makes Rust the source of truth for the protocol and asks for a typed `CoreClient`. Implementing it (prompt 02) required choices ADR-0010 leaves open: how the TypeScript export runs, how the client knows the result type of each command, which side raises which error codes, how events reach several connections, and what reconnection guarantees the client gives.

## Decision
- **Codegen is a binary, not export tests.** `crates/se-codegen` calls ts-rs `export_all` on the root types (`ClientMessage`, `ServerMessage`, `CoreHandshake`; everything they reference follows) into a scratch folder, replaces the ts-rs note with a `GENERATED — DO NOT EDIT` header and writes an `index.ts`. `scripts/codegen.mjs` copies the result to `packages/protocol/src/generated/`, or with `--check` compares the two (CRLF-normalised) and fails on any difference. `i64`/`u64` are exported as `number` (ADR-0004).
- **Every `Command` variant has a `CommandResult` variant of the same name**, hence the same `kind` on the wire (`{"kind":"ping"}` → `{"kind":"ping"}`). TypeScript derives `ResultFor<K> = Extract<CommandResult, { kind: K }>` with no hand-written table; a type-level test fails the build if a command has no result, and the client rejects a response whose result kind differs from its command's (`INTERNAL`).
- **One error-code list for both sides.** `ErrorCode` includes the codes clients raise (`TIMEOUT`, `CONNECTION_LOST`, `CLIENT_CLOSED`) next to those se-core sends (`INVALID_REQUEST`, `UNKNOWN_COMMAND`, `INTERNAL`, …), so UI code and agents match one union.
- **Events.** `CoreEvent` (not `Event`, which would shadow the DOM type) is fanned out with a bounded `tokio::sync::broadcast` channel (256 per connection). A connection that falls behind skips the oldest events and receives `ResyncRequired { missedEvents }`; publishers never wait. Each new connection first receives `CoreReady { version }`; on shutdown every connection receives `CoreShuttingDown` before its socket closes.
- **Client API.** `CoreClient.connect(source)` creates and connects a client; `send(command)` returns `Promise<ResultFor<kind>>`; events are subscribed with `on(kind, handler)` (named `subscribe` in ADR-0010), which returns the unsubscribe function; `state` / `onStateChange` expose `connecting | open | reconnecting | closed`.
- **Reconnection.** `CoreClient.connect(source)` accepts a fixed endpoint or a function called before every attempt (a restarted core has a new port and token). Backoff: 250 ms doubling, capped at 5 s, reset on open. Requests sent while disconnected are queued and sent on the next connection; requests already written when the connection drops fail with `CONNECTION_LOST` (they may have run). Every request has a timeout (30 s default).
- **App data folder.** The Electron app passes `app.getPath('userData')` to se-core as `SE_DATA_DIR`; se-core writes daily logs to `<SE_DATA_DIR>/logs/` (7 kept) and falls back to `<OS data dir>/SimpleEffect` when run alone.

## Consequences
- Adding a command is mechanical (recipe in `crates/se-protocol/CLAUDE.md`): Rust variants, handler, router arm, `pnpm codegen`. The compiler (Rust match, TS types) catches missing pieces.
- `cargo test` never writes generated files; only `pnpm codegen` does.
- Result variants are named after commands (`CommandResult::GetCoreInfo`), not after what they contain; the payload struct carries the descriptive name (`CoreInfo`).
- A client must treat `ResyncRequired` and reconnections (state `reconnecting` → `open`) as "state may be stale": later prompts refetch the project snapshot then.

## Alternatives considered
- **ts-rs `#[ts(export)]` tests**: export as a side effect of `cargo test`, configured through environment variables; harder to check for staleness and to post-process. Rejected.
- **A hand-written `CommandResultMap` (or a macro generating it)**: an extra table to keep in sync, or macro magic. Rejected in favour of the naming convention.
- **Client-only error codes in TypeScript**: two lists of codes to match on. Rejected.
- **Unbounded per-connection queues**: a stuck client would grow memory without limit. Rejected.
