# se-protocol

## Purpose
Data transfer objects exchanged between `se-core` and its clients: the startup handshake line and the JSON messages of the `/rpc` WebSocket. DTOs only: no logic beyond serde (de)serialisation. Business rules live in `se-domain` and `se-app`.

## Public API
- `CoreHandshake { port, token, version }`: the single JSON line `se-core` prints on stdout at startup. `Debug` redacts the token.
- `ClientMessage::Request { id, command }` → `ServerMessage::Response(Response { id, outcome })`.
- `Command` (`ping`), `CommandResult` (`pong { version }`), `ResponseOutcome` (`result` | `error`), `ProtocolError { code, message }`, `ErrorCode` (`INVALID_REQUEST`, `INTERNAL`), `RequestId`.

Wire format (`docs/architecture.md` §6):
```json
{"type":"request","id":1,"command":{"kind":"ping"}}
{"type":"response","id":1,"result":{"kind":"pong","version":"0.1.0"}}
{"type":"response","id":1,"error":{"code":"INVALID_REQUEST","message":"…"}}
```

## Rules and invariants
- Enums are internally tagged: `type` for messages, `kind` for commands/results/events.
- Error codes are stable (`SCREAMING_SNAKE_CASE`); messages are actionable (read by users and AI agents).
- Never log a `CoreHandshake` token or any session token.
- Prompt 02 adds `ts_rs::TS` derives and generates `@se/protocol`; after that, every change here requires `pnpm codegen`.

## Allowed dependencies
`se-domain`, `serde` (+ `ts-rs` from prompt 02). Nothing else; see `docs/architecture.md` §3.

## How to test
- `cargo test -p se-protocol`
- Unit tests in-file assert the exact JSON shape of each message.
