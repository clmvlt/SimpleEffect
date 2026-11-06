# Journal de progression

Journal de bord partagé entre les sessions. **Chaque session qui exécute un prompt ajoute une entrée en haut de la liste** (la plus récente en premier). Les entrées sont rédigées en anglais (comme le reste de la documentation technique) pour rester lisibles par toutes les IA.

## Format d'une entrée

```markdown
## YYYY-MM-DD — Prompt NN: <title> — <Done | Partial>

**Branch**: `feature/NN-<slug>`, merged into `dev` at `<short sha>`

**Done**
- What was implemented (modules, commands, components).

**Deviations from the prompt**
- What differs from the spec and why (link the ADR if a decision was recorded).

**Follow-ups**
- Remaining work, known issues, TODO(prompt-NN) items left in code.

**Notes for the next agent**
- Anything non-obvious discovered (gotchas, empirical findings, commands).
```

---

<!-- Add new entries below this line, most recent first. -->

## 2026-11-06 — Prompt 02: Protocole typé et couche RPC — Done

**Branch**: `feature/02-protocole-rpc`, merged into `dev` at `MERGE_SHA`

**Done**
- `se-protocol`: `ClientMessage::Request(RequestEnvelope)`, `ServerMessage::{Response(ResponseEnvelope), Event(EventEnvelope)}`, `Command` (`ping`, `getCoreInfo`), `CommandResult` (same kinds; `CoreInfo { version, os, arch, appDataDir, logsDir }`), `CoreEvent` (`coreReady`, `coreShuttingDown`, `resyncRequired`), `ProtocolError { code, message, details? }`, `ErrorCode` (`INVALID_REQUEST`, `UNKNOWN_COMMAND`, `INTERNAL`, `TIMEOUT`, `CONNECTION_LOST`, `CLIENT_CLOSED`). All derive `ts_rs::TS` (ts-rs 12); `CoreHandshake` too. Round-trip tests for every envelope.
- `crates/se-codegen` (new, dev tool) + `scripts/codegen.mjs`: `pnpm codegen` writes `packages/protocol/src/generated/` (one file per type + `index.ts`, `GENERATED — DO NOT EDIT` header, `u64`/`i64` → `number`); `pnpm codegen --check` (already in `pnpm check`) compares a fresh export, CRLF-normalised. Verified: adding a field to `CoreInfo` without regenerating fails the check.
- `se-core`: `rpc/` module — `RpcRouter` (parse → `dispatch` → encode; `INVALID_REQUEST` vs `UNKNOWN_COMMAND` with the supported kinds in the message), handlers per group (`rpc/handlers/system.rs`), `EventBus` (bounded `broadcast`, 256; lag → `resyncRequired`), per-connection loop (greets with `coreReady`, forwards `coreShuttingDown` before closing, tracing span per connection). `AppPaths` from `SE_DATA_DIR` (fallback `<OS data dir>/SimpleEffect`); logs: `tracing-appender` daily files `<data>/logs/se-core.YYYY-MM-DD.log`, 7 kept, filter `SE_LOG`, stderr in debug builds or when the folder is unusable. Integration tests moved to `tests/core_process/` (temporary `SE_DATA_DIR` per test; 2 clients receive `coreShuttingDown`; log file exists and never contains the token).
- `@se/protocol`: `CoreClient.connect(source)` (fixed endpoint or resolver called before every attempt), `send` typed by `ResultFor<K>` (derived, no table; compile-time check that every command has a result), per-request timeout (30 s default), `on(kind, handler)`, observable `state` (`connecting | open | reconnecting | closed`), backoff 250 ms → 5 s, queued requests survive reconnection, in-flight ones fail with `CONNECTION_LOST`, `close()` → `CLIENT_CLOSED`. Tested with `FakeWebSocket` + fake timers.
- Electron: supervisor passes `SE_DATA_DIR` = `app.getPath('userData')` (`core-environment.ts`); the handshake parser uses the generated `CoreHandshake` type.
- Renderer: `src/shared/core/` (`createCoreClient`, `CoreClientProvider`, `useCoreClient`, `useConnectionStatus`, `useCoreEvent`, `useCoreInfo` + pure `watchCoreInfo`); the hand-written ping (`shared/core-connection/`) is gone. The screen shows the status, version, platform, data and log folders from `getCoreInfo`.
- ADR-0012 (protocol conventions), `docs/architecture.md` §2/§3/§4.4/§6, glossary, CLAUDE.md of `se-protocol` (add-a-command recipe), `se-core`, `se-codegen`, `@se/protocol`, `@se/desktop`, root.
- Verified on Windows with `electron-vite dev`: screen shows « Cœur connecté — se-core 0.1.0 » plus the folders; `taskkill /F` on se-core → « Reconnexion… » → reconnected to the restarted core (new port) in < 1 s; `%APPDATA%\SimpleEffect\logs\se-core.2026-11-06.log` written; closing the window leaves no process.

**Deviations from the prompt**
- The event enum is `CoreEvent`, not `Event` (the generated TS type would shadow the DOM `Event` in the renderer).
- Results use the same `kind` as their command (`ping` → `{"kind":"ping"}`, no more `pong`): that is what lets `ResultFor` be derived without a table (ADR-0012).
- `CoreClient.connect` is a static factory taking `{ url, token }` **or** a resolver function, because a restarted core has a new port and token.
- `TIMEOUT`, `CONNECTION_LOST`, `CLIENT_CLOSED` live in the Rust `ErrorCode` although only clients raise them (one list to match on).
- Log filter variable is `SE_LOG` (was `RUST_LOG` in prompt 01), as the prompt asks.

**Follow-ups**
- React hooks of `src/shared/core/` have no component tests: there is no DOM test environment yet (Testing Library arrives with prompt 03); their logic lives in tested plain functions.
- Handlers are synchronous; the first async handler must make `RpcRouter::handle_text` async and spawn a task per request (noted in `crates/se-core/CLAUDE.md`).
- Nothing reacts to `resyncRequired` or to reconnections yet: prompt 07+ (project snapshot) must refetch state then.
- `release.yml` still unverified on macOS/Linux (unchanged by this prompt).

**Notes for the next agent**
- `pnpm codegen` builds and runs `se-codegen`; it refuses a non-empty output folder because ts-rs merges into existing files.
- `serde(flatten)` on `ResponseOutcome` accepts a frame with both `result` and `error` (first wins): harmless since only se-core produces responses.
- tracing-appender prunes old logs before creating the folder, so `logging.rs` creates `logs/` first (otherwise a spurious error is printed on a fresh install).
- `TaskStop` on a backgrounded `electron-vite dev` leaves Electron running on Windows; close it with `taskkill /PID <main electron pid>` (graceful: se-core stops through its stdin).

## 2026-11-06 — Prompt 01: Bootstrap du monorepo — Done

**Branch**: `feature/01-bootstrap-monorepo`, merged into `dev` at `4d8888a`

**Done**
- Rust workspace: 11 crates (edition 2024, toolchain pinned to 1.99.1), inter-crate dependencies declared per `docs/architecture.md` §3, workspace lints (`unsafe_code` forbidden, `missing_docs`, clippy `pedantic` + `unwrap_used`/`expect_used`/`panic`/`todo`/`dbg_macro` denied; tests allowed through `clippy.toml`).
- `se-core`: axum 0.8 on `127.0.0.1:0`, 256-bit session token, single-line stdout handshake, `/rpc` WebSocket ping/pong with hand-written `se-protocol` DTOs (plus `INVALID_REQUEST`/`INTERNAL` error responses), access-guard middleware (foreign `Host` → 403, missing/wrong token → 401, constant-time compare), graceful shutdown on stdin EOF or Ctrl+C with a 2 s drain bound. `tests/rpc_endpoint.rs` spawns the real binary (handshake, ping, 401, 403, stdin close with and without an open socket).
- pnpm workspace (pnpm 12.18), `tsconfig.base.json` + project references (`tsc -b`), ESLint 10 flat config (`strictTypeChecked`, react-hooks 7 with React Compiler rules, `eslint-plugin-boundaries` 7 policies — checked with deliberate violations), Prettier, Vitest 5 projects per package.
- `apps/desktop`: Electron 45 + electron-vite 6 (Vite 8) + React 19.3 + React Compiler (Babel preset) + Tailwind 4.3. `CoreSupervisor` (handshake parsing, 15 s handshake timeout, backoff 250 ms → 10 s reset after 10 s of stable run, stop = close stdin then kill after 3 s), sandboxed CommonJS preload bridge, hardened window, CSP meta tag generated per mode, renderer status screen with automatic reconnection.
- Scripts (`scripts/*.mjs`, cross-platform): `check`, `dev`, `build`, `package`, `set-version`, `codegen` and `storybook` placeholders.
- Packaging (`electron-builder.yml`: NSIS / dmg / AppImage + deb, `se-core` in `resources/bin`), `ci.yml` (ubuntu + windows, `pnpm check`) and `release.yml` (tags `v*.*.*`, pre-release when the tag contains `-`, notes generated, `contents: write` on the publish job only). `actionlint` 1.7.12: no findings.
- Verified on Windows: `pnpm dev` shows « Cœur connecté — se-core 0.1.0 »; killing `se-core` shows « Reconnexion… » then reconnects to a new core within ~0.5 s; closing the window and force-killing Electron both leave no `se-core`; the NSIS installer installs to `%LOCALAPPDATA%\Programs\simpleeffect`, the app starts the core from `resources\bin` and connects; silent uninstall is clean.

**Deviations from the prompt**
- TypeScript 6.0.3, not the latest 7.0.2: TypeScript 7 (native) has no JS API and `typescript-eslint` supports `<6.1` (gotcha added to the root `CLAUDE.md`).
- Tailwind 4.3.4, not 4.3.5: 4.3.5 was less than a day old and pnpm's `minimumReleaseAge` rejects it.
- `@babel/core` 7.29, not 8: `babel-plugin-react-compiler` 1.0 depends on `@babel/types` 7. The native (oxc) compiler of `@vitejs/plugin-react` 6 is experimental and not used.
- Electron ≥ 45 no longer downloads its binary at install time: the desktop `dev` script runs `install-electron` (idempotent) first.
- `productName: SimpleEffect` in `apps/desktop/package.json` and `extraMetadata.name: simpleeffect` in `electron-builder.yml`, otherwise the install folder is `@sedesktop` and the user-data folder `%APPDATA%\@se`.
- `pnpm check` runs every step and reports all failures instead of stopping at the first one.

**Follow-ups**
- TODO(prompt-02): replace the hand-written ping (`se-protocol` types, `apps/desktop/src/shared/core-connection/`) with generated types and `CoreClient`; implement `pnpm codegen`.
- TODO(prompt-04): move the temporary status strings to i18n and replace `App.tsx` with the shell.
- `release.yml` has not run yet (needs a tag, or `gh workflow run release.yml` once the workflow is on `main`): the macOS and Linux packaging steps are unverified.
- `actionlint` ran without shellcheck (not installed locally).
- The renderer bundle is not minified (electron-vite default); reconsider in prompt 29 if startup time matters.

**Notes for the next agent**
- Remote-debug the renderer: `pnpm --filter @se/desktop exec electron-vite dev --remoteDebuggingPort 9223`, then use the Chrome DevTools Protocol on `http://127.0.0.1:9223/json`.
- Run the core alone: `cargo run -p se-core` prints the handshake; Ctrl+C stops it.
- pnpm 12 may leave paths longer than 260 characters after a failed install on Windows: delete with `rd /s /q "\\?\<absolute path>"`.
