# se-core

## Purpose
The core binary: composition root and local server. It binds `127.0.0.1` on a random port, prints the handshake, serves `/rpc` (WebSocket, later `/media` and `/mcp`) and wires adapters to `se-app`. It contains wiring only, never domain logic.

## Public API
The process contract (consumed by `apps/desktop/electron/main/core-supervisor.ts`):
- **stdout**: exactly one line, `{"port":<u16>,"token":"<64 hex>","version":"<semver>"}` (`se_protocol::CoreHandshake`). Nothing else is ever written to stdout.
- **stderr**: `tracing` logs (filter with `RUST_LOG`, plain text when not a terminal).
- **stdin**: kept open by the parent and never read for data; end-of-file means "shut down".
- **HTTP**: every route requires `Host: 127.0.0.1:<port>` or `localhost:<port>` (else 403) and `?t=<token>` (else 401).
- `GET /rpc` (WebSocket): JSON request/response messages of `se-protocol`.

Modules: `main.rs` (entry), `server.rs` (bind, handshake, serve, drain), `access_guard.rs` (Host + token middleware, `SessionToken`), `rpc.rs` (socket loop), `dispatch.rs` (pure frame → reply), `shutdown.rs` (stdin EOF / Ctrl+C), `handshake.rs`, `logging.rs`, `error.rs`.

## Rules and invariants
- `anyhow` only in `main`; everything else returns `CoreError` (`thiserror`).
- Never log the session token (`SessionToken` and `CoreHandshake` redact it in `Debug`).
- The process must never outlive its parent: stdin EOF and Ctrl+C trigger a graceful shutdown bounded by `DRAIN_TIMEOUT`.
- New routes go behind the access-guard middleware (added as a router layer in `server.rs`).

## Allowed dependencies
Every workspace crate (composition root); nothing depends on `se-core`. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-core`
- Unit tests in-file (access guard, dispatch, shutdown).
- `tests/rpc_endpoint.rs` spawns the real binary (`CARGO_BIN_EXE_se-core`) and checks the handshake, ping/pong, 401/403 rejections and shutdown on stdin close.
- Manual run: `cargo run -p se-core` prints the handshake; press Ctrl+C or close stdin to stop.
