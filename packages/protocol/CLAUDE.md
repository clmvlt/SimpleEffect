# @se/protocol

## Purpose
TypeScript side of the se-core protocol: the types generated from the Rust crates (ts-rs, `pnpm codegen`), types derived from them, and the typed `CoreClient` used by the renderer. No React, no rendering code (ADR-0010, ADR-0012).

## Public API (`src/index.ts`)
- **Generated types** (`src/generated/`): protocol (`Command`, `CommandResult`, `CoreEvent`, `ErrorCode`, `ProtocolError`, `CoreInfo`, `CoreHandshake`, envelopes, `JsonValue`, project DTOs `ProjectSession`, `ProjectSnapshot`, `ProjectPatch`, entities, `RecentProject`, `NewProjectDefaults`…) and document model from se-domain (`Project`, `Sequence`, `Track`, `Item`, `ItemKind`, `Animatable<T>`, `Flicks` — a branded `number` —, ids as `string`, `Violation`, `ViolationCode`, `FormatPreset`…, see `docs/architecture.md` §5.3).
- **Derived types** (`protocol-types.ts`): `CommandKind`, `CommandOf<K>`, `ResultFor<K>` (`Extract<CommandResult, { kind: K }>`), `CoreEventKind`, `CoreEventOf<K>`, `CommandsWithoutResult` (must stay `never`).
- **`CoreClient`** (`core-client.ts`):
  - `CoreClient.connect(source, options?)`: `source` is `{ url, token }` or `() => Promise<{ url, token }>` (called before every attempt). Options: `requestTimeoutMs` (30 s), `reconnectDelay` (250 ms doubling, 5 s cap), `openSocket` (fake in tests), `logger`.
  - `send(command, { timeoutMs? })` → `Promise<ResultFor<kind>>`; rejects with `CoreRequestError { code, command, details }`.
  - `on(kind, handler)` → unsubscribe; `state` and `onStateChange(listener)` → unsubscribe; `endpoint` (the endpoint of the latest attempt, for media URLs); `close()`.
  - Requests sent while not `open` are queued; requests in flight when the connection drops fail with `CONNECTION_LOST`; `close()` fails everything with `CLIENT_CLOSED`.
- `rpcSocketUrl(endpoint)`, `CoreRequestError`, `WebSocketLike` / `WebSocketFactory`.
- **Media** (`media-url.ts`, `waveform-peaks.ts`): `assetMediaPath(assetId, file)` (`original`, `mezzanine`, `proxy`, `thumbnails` JSON, `thumbnailSheet`, `poster`, `{ peaks: level }`), `fontMediaPath(faceId)` (`/media/fonts/<id>`), `mediaUrl(endpoint, path, version?)` (adds `?t=`), `PEAK_LEVELS` (256, 1024, 4096 — mirror of `se_media::peaks::PEAK_LEVELS`), `decodeWaveformPeaks(buffer)` → `WaveformPeaks` (the `SEPK` format of `crates/se-media/CLAUDE.md`).

## Layout
| File | Role |
|---|---|
| `core-client.ts` | The client: connection lifecycle, sending, event dispatch |
| `pending-requests.ts` | In-flight/queued requests, timeouts, result-kind check |
| `server-message.ts` | Envelope validation of incoming frames |
| `core-endpoint.ts`, `reconnect-delay.ts`, `web-socket-like.ts`, `core-request-error.ts` | Small pure helpers and types |
| `media-url.ts`, `waveform-peaks.ts` | `/media` URLs and the `SEPK` decoder (`media.test.ts`) |
| `test-support/fake-web-socket.ts` | `FakeWebSocket` playing the se-core side in tests (not exported) |

## Rules and invariants
- `src/generated/` is written by `pnpm codegen` only: never hand-edit it; change the Rust types instead. ESLint and Prettier ignore it.
- Never write a command → result table by hand: rely on the same-`kind` convention (`ResultFor`).
- No imports from React, `@se/engine` or `@se/ui`; no dependency on Electron (the endpoint comes from the caller).
- The package is consumed as TypeScript source (`exports` → `src/index.ts`); no build step.
- Never log the endpoint token.

## Allowed dependencies
No workspace package, no runtime dependency. See `docs/architecture.md` §4.1.

## How to test
- `pnpm vitest run --project protocol` (or `pnpm test` at the root).
- `core-client.test.ts` drives `CoreClient` with `FakeWebSocket` and fake timers (correlation, queueing, timeouts, reconnection backoff, events, close). `protocol-types.test.ts` (lists every command and event kind: update it when adding one) and `document-types.test.ts` hold compile-time checks (run by `tsc -b`).
