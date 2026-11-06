# se-protocol

## Purpose
Data transfer objects exchanged between `se-core` and its clients: the startup handshake line and the JSON messages of the `/rpc` WebSocket. DTOs only: no logic beyond serde (de)serialisation and trivial constructors. Business rules live in `se-domain` and `se-app`. Every type derives `ts_rs::TS` and is exported to `@se/protocol` by `pnpm codegen` (ADR-0010, ADR-0012).

## Public API
| Module | Types |
|---|---|
| `handshake.rs` | `CoreHandshake { port, token, version }`: the single stdout line of `se-core`. `Debug` redacts the token |
| `envelope.rs` | `ClientMessage::Request(RequestEnvelope { id, command })`; `ServerMessage::Response(ResponseEnvelope { id, outcome })` / `ServerMessage::Event(EventEnvelope { event })`; `ResponseOutcome` (`result` \| `error`); `RequestId` (`u64`, exported as `number`) |
| `command.rs` | `Command` (`ping`, `getCoreInfo`) |
| `command_result.rs` | `CommandResult` (`ping`, `getCoreInfo(CoreInfo)`), `CoreInfo { version, os, arch, appDataDir, logsDir }` |
| `event.rs` | `CoreEvent` (`coreReady { version }`, `coreShuttingDown`, `resyncRequired { missedEvents }`) |
| `error.rs` | `ProtocolError { code, message, details? }`, `ErrorCode` |

Wire format (`docs/architecture.md` §6):
```json
{"type":"request","id":1,"command":{"kind":"getCoreInfo"}}
{"type":"response","id":1,"result":{"kind":"getCoreInfo","version":"0.1.0","os":"windows",…}}
{"type":"response","id":1,"error":{"code":"UNKNOWN_COMMAND","message":"…","details":{"kind":"explode"}}}
{"type":"event","event":{"kind":"resyncRequired","missedEvents":3}}
```

## Rules and invariants
- Enums are internally tagged: `type` for messages, `kind` (camelCase) for commands, results and events. Struct-variant fields are camelCase (`rename_all_fields`).
- **Every `Command` variant has a `CommandResult` variant with the same name** (same wire `kind`): the TypeScript client derives result types from it. Put a non-trivial payload in its own struct (`GetCoreInfo(CoreInfo)`).
- Error codes are stable (`SCREAMING_SNAKE_CASE`): never rename or reuse one. Messages are actionable (read by users and AI agents). `TIMEOUT`, `CONNECTION_LOST`, `CLIENT_CLOSED` are raised by clients, never sent by `se-core`.
- Integers that can exceed 2^53 must not appear in DTOs: `u64`/`i64` are exported as `number`.
- Optional fields that may be omitted on the wire use `#[serde(default, skip_serializing_if = "Option::is_none")]` + `#[ts(optional)]`.
- Never log a `CoreHandshake` token or any session token.
- After **any** change here: `pnpm codegen`, then commit `packages/protocol/src/generated/` (`pnpm check` fails otherwise).

## Recipe: add a command
1. `command.rs`: add the variant to `Command` (doc comment naming its result).
2. `command_result.rs`: add the variant **with the same name** to `CommandResult` (unit, or wrapping a new payload struct deriving `TS`). Add a JSON-shape test.
3. `crates/se-core/src/rpc/handlers/<group>.rs`: write the handler (returns `CommandResult`, or `Result<CommandResult, ProtocolError>` if it can fail; new error codes go in `ErrorCode`).
4. `crates/se-core/src/rpc/router.rs`: add the match arm in `dispatch` (the compiler points at it); extend `every_result_has_the_kind_of_its_command`.
5. `pnpm codegen`; the renderer can now call `client.send({ kind: '<newCommand>', … })` with a typed result.
6. Document it in `docs/architecture.md` §6 if it introduces a new concept.

Adding an event: add the variant to `CoreEvent`, publish it with `EventBus::publish` in se-core, run `pnpm codegen`; clients subscribe with `client.on('<kind>', handler)`.

## Allowed dependencies
`se-domain`, `serde`, `serde_json` (for `details`), `ts-rs`. Nothing else; see `docs/architecture.md` §3.

## How to test
- `cargo test -p se-protocol`
- Unit tests in-file assert the exact JSON shape of each message and round-trip every envelope.
