# @se/desktop

## Purpose
The Electron application: the main process supervises se-core and manages windows; the preload exposes a minimal typed bridge; the renderer is the React app. No business logic in `electron/`: the core owns the document, and the renderer sends commands to it.

## Public API
- `electron/shared/bridge-api.ts`: the contract shared by main, preload and renderer (`CoreConnection`, `SimpleEffectBridge`, `IPC_CHANNELS`).
- `window.simpleEffect.getCoreConnection()`: resolves with `{ url, token, version }` of the running core, waiting while it (re)starts. Call it again after every disconnect (new port and token).

## Layout
| Path | Role |
|---|---|
| `electron/main/index.ts` | App lifecycle: start the supervisor, register IPC, open the window, stop the core before quitting |
| `electron/main/core-supervisor.ts` | Spawns se-core, reads the handshake, restarts with backoff, stops it gracefully (stdin close) |
| `electron/main/core-handshake.ts` | Pure parser of the handshake line |
| `electron/main/restart-backoff.ts` | Pure backoff delays (250 ms doubling, 10 s cap) |
| `electron/main/core-binary-path.ts` | Locates the binary: `target/debug` in dev (`SE_CORE_PATH` override), `resources/bin` when packaged |
| `electron/main/platform/` | OS-specific helpers (executable names) |
| `electron/main/main-window.ts` | Hardened `BrowserWindow` (context isolation, sandbox, no Node, no pop-ups or navigation) |
| `electron/preload/index.ts` | `contextBridge` exposure of the bridge (built as CommonJS: required by the sandbox) |
| `tooling/` | Build-time helpers: Content-Security-Policy builder and its Vite plugin |
| `src/app/` | Providers, root layout (`App.tsx`); temporary connection screen until prompt 04 |
| `src/shared/` | Cross-cutting code (`core-connection/`: live status of the core connection) |
| `src/features/<name>/` | One folder per feature (`components/`, `hooks/`, `store/`, `services/`, `api.ts`, `index.ts`) |
| `src/i18n/` | Locales (`fr` default), from prompt 04 |

## Rules and invariants
- Security: `contextIsolation: true`, `nodeIntegration: false`, `sandbox: true`; the CSP (meta tag written by `tooling/csp-plugin.ts`) only allows connections to `127.0.0.1` in production.
- The session token is never logged (the supervisor logs the URL and version only).
- Features import each other only through their `index.ts`; `shared/` never imports features or `app/` (enforced by `eslint-plugin-boundaries` in the root `eslint.config.js`).
- Components hold no business logic; all UI strings go through i18n from prompt 04 (temporary French strings are marked `TODO(prompt-04)`).
- All runtime dependencies are bundled by electron-vite, so they are `devDependencies` and the packaged app ships no `node_modules`.

## Allowed dependencies
`@se/protocol`, `@se/engine`, `@se/ui` (renderer only), React, Zustand. Electron APIs only in `electron/`. See `docs/architecture.md` §4.

## How to test
- `pnpm vitest run --project desktop` (or `pnpm test` at the root): `electron/**/*.test.ts`, `tooling/**/*.test.ts`, `src/**/*.test.{ts,tsx}`.
- Manual: `pnpm dev` from the root (builds se-core, downloads the Electron binary on first run, opens the window).
- Packaging: `pnpm package` from the root → `apps/desktop/release/` (config in `electron-builder.yml`).
