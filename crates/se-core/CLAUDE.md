# se-core

## Purpose
The core binary: composition root and local server. It binds `127.0.0.1` on a random port, prints the handshake, serves `/rpc` (WebSocket, later `/media` and `/mcp`) and wires adapters to `se-app`. It contains wiring only, never domain logic.

## Public API
The process contract (consumed by `apps/desktop/electron/main/core-supervisor.ts`):
- **stdout**: exactly one line, `{"port":<u16>,"token":"<64 hex>","version":"<semver>"}` (`se_protocol::CoreHandshake`). Nothing else is ever written to stdout.
- **Environment**: `SE_DATA_DIR` = app data folder (set by the supervisor to Electron's `userData`; default `<OS data dir>/SimpleEffect`). `SE_LOG` = `tracing` filter (default `info`, e.g. `SE_LOG=se_core=debug`).
- **Logs**: daily files `<SE_DATA_DIR>/logs/se-core.<YYYY-MM-DD>.log` (7 kept), plus stderr (plain text when not a terminal) in debug builds or when the log folder is unusable.
- **stdin**: kept open by the parent and never read for data; end-of-file means "shut down".
- **HTTP**: every route requires `Host: 127.0.0.1:<port>` or `localhost:<port>` (else 403) and `?t=<token>` (else 401).
- `GET /rpc` (WebSocket): `se-protocol` messages. The core greets each connection with `coreReady`, answers requests, pushes events, and sends `coreShuttingDown` then a close frame on shutdown.

## Modules
| Module | Role |
|---|---|
| `main.rs` | Entry: resolve paths, init logging, run the server |
| `app_paths.rs` | `AppPaths { data_dir, logs_dir }` from `SE_DATA_DIR` or the OS data dir |
| `logging.rs` | `tracing` + `tracing-appender` (daily rolling file, non-blocking; hold the `LogGuard`) |
| `server.rs` | Bind, handshake, routes, graceful shutdown (publishes `coreShuttingDown`, then drains within `DRAIN_TIMEOUT`) |
| `access_guard.rs` | Host + token middleware, `SessionToken` |
| `rpc.rs` | `RpcState` and the `/rpc` upgrade handler |
| `rpc/router.rs` | `RpcRouter`: frame → parse → `dispatch` (one match arm per command) → encoded response |
| `rpc/request.rs` | Frame parsing; `INVALID_REQUEST` vs `UNKNOWN_COMMAND` with actionable messages |
| `rpc/handlers.rs`, `rpc/handlers/<group>.rs` | `HandlerContext`; one file per command group (`system.rs`: `ping`, `getCoreInfo`) |
| `rpc/event_bus.rs` | `EventBus` (bounded `broadcast`, 256 per connection); lag → `resyncRequired` |
| `rpc/connection.rs` | Per-socket loop (`biased` select: events, then requests, then shutdown), tracing span per connection |
| `rpc/encode.rs` | Message serialisation with an `INTERNAL` fallback |
| `shutdown.rs`, `handshake.rs`, `error.rs` | stdin EOF / Ctrl+C, handshake line, fatal `CoreError` |

Adding a command: recipe in `crates/se-protocol/CLAUDE.md`. Publishing an event: `EventBus::publish` (clone the bus into whatever produces events).

## Rules and invariants
- `anyhow` only in `main`; everything else returns `CoreError` (`thiserror`) or `ProtocolError` (handlers).
- Never log the session token (`SessionToken` and `CoreHandshake` redact it in `Debug`; the access guard logs the path only, never the query).
- The process must never outlive its parent: stdin EOF and Ctrl+C trigger a graceful shutdown bounded by `DRAIN_TIMEOUT`.
- Publishing an event never waits: a slow connection only delays itself and is told to resync when it overflows.
- Handlers are synchronous for now. When the first `async` handler arrives, make `RpcRouter::handle_text` async and spawn one task per request in `rpc/connection.rs`, so a slow command never delays events or other requests.
- New routes go behind the access-guard middleware (router layer in `server.rs`).

## Allowed dependencies
Every workspace crate (composition root); nothing depends on `se-core`. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-core`
- Unit tests in-file (paths, access guard, request parsing, router, event bus, encoding, shutdown).
- `tests/core_process/` spawns the real binary (`CARGO_BIN_EXE_se-core`) with a temporary `SE_DATA_DIR`: handshake, 401/403, commands and error codes, `coreShuttingDown` delivered to two clients, shutdown on stdin close, log file contents (no token).
- Manual run: `cargo run -p se-core` prints the handshake; press Ctrl+C or close stdin to stop. Logs land in `<OS data dir>/SimpleEffect/logs/` unless `SE_DATA_DIR` is set.
