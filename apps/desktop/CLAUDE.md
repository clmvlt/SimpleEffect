# @se/desktop

## Purpose
The Electron application: the main process supervises se-core and manages windows; the preload exposes a minimal typed bridge; the renderer is the React app. No business logic in `electron/`: the core owns the document, and the renderer sends commands to it.

## Public API
- `electron/shared/bridge-api.ts`: the contract shared by main, preload and renderer (`CoreConnection`, `ChooseFolderOptions`, `SimpleEffectBridge`, `IPC_CHANNELS`).
- `window.simpleEffect.getCoreConnection()`: resolves with `{ url, token, version }` of the running core, waiting while it (re)starts. `CoreClient` calls it before every connection attempt (a restarted core has a new port and token).
- `window.simpleEffect.chooseFolder({ title, buttonLabel, defaultPath? })`: native folder picker (absolute path or `null`); `showInFolder(path)`: reveals a file or folder in the OS file manager. Renderer code reaches the bridge through `useDesktopBridge()` (`src/shared/desktop/`), never `window` directly (except `main.tsx`).
- Renderer code talks to se-core only through `src/shared/core/` hooks and `@se/protocol` types: `useCoreClient().send({ kind: 'getCoreInfo' })` resolves with the typed result.

## Layout
| Path | Role |
|---|---|
| `electron/main/index.ts` | App lifecycle: start the supervisor, register IPC, open the window, stop the core before quitting |
| `electron/main/core-supervisor.ts` | Spawns se-core (with `SE_DATA_DIR` = `userData`), reads the handshake, restarts with backoff, stops it gracefully (stdin close) |
| `electron/main/core-environment.ts` | Pure: the core's environment (`SE_DATA_DIR`) |
| `electron/main/ipc-handlers.ts`, `ipc-arguments.ts` | IPC handlers (core connection, folder dialog, reveal) and the validation of their arguments |
| `electron/main/core-handshake.ts` | Pure parser of the handshake line (type `CoreHandshake` generated in `@se/protocol`) |
| `electron/main/restart-backoff.ts` | Pure backoff delays (250 ms doubling, 10 s cap) |
| `electron/main/core-binary-path.ts` | Locates the binary: `target/debug` in dev (`SE_CORE_PATH` override), `resources/bin` when packaged |
| `electron/main/platform/` | OS-specific helpers (executable names) |
| `electron/main/main-window.ts` | Hardened `BrowserWindow` (context isolation, sandbox, no Node, no pop-ups or navigation) |
| `electron/preload/index.ts` | `contextBridge` exposure of the bridge (built as CommonJS: required by the sandbox) |
| `tooling/` | Build-time helpers: Content-Security-Policy builder and its Vite plugin |
| `src/main.tsx` | Composition root: one `CoreClient`, the project store and its sync, the shortcut registry, one i18next instance, the hash router built from `createAppRoutes({ layoutStorage })`, rendered inside `AppProviders` |
| `src/app/AppProviders.tsx` | App-wide contexts: `I18nextProvider`, `CoreClientProvider`, `DesktopBridgeProvider`, `ProjectStoreProvider`, `ShortcutProvider`, `TooltipProvider`, and the `Toaster` |
| `src/app/routes.tsx` | Routes (`/` home, `/project/:projectId` editor) under `AppLayout`; dependencies injected so tests use a memory router |
| `src/app/layout/AppLayout.tsx` | Window frame: the route's page above, the status bar below |
| `src/app/editor/EditorPage.tsx` | The editor of `/project/:projectId`: loading/failed/closed states (`use-editor-project.ts`, `load-editor-project.ts`: snapshot, or reopen from the recent list), then `EditorHeader` (home, project and sequence names, undo/redo with Ctrl/⌘+Z and Ctrl/⌘+Maj+Z via `use-history-commands.ts`, labels via `history-label.ts`) and `EditorLayout` |
| `src/app/editor/EditorLayout.tsx` | Editor layout: nested `Dock`s (ids `editor.rows`, `editor.columns`) holding the feature panels; Inspecteur and IA share the right panel as tabs |
| `src/app/status-bar/` | `StatusBar` (core connection announced as a `status`, AI availability, `__APP_VERSION__`) and the pure `connectionIndicator(state)` |
| `src/app/styles.css` | Only imports `@se/ui/styles.css` |
| `src/shared/core/` | The core connection: `createCoreClient(bridge)`, `CoreClientProvider`, `useCoreClient()`, `useConnectionStatus()`, `useCoreEvent(kind, handler)`, `useCoreInfo()` (logic in `watch-core-info.ts`) |
| `src/shared/project-store/` | Read-only mirror of the open project (ADR-0017): `createProjectStore()` (Zustand vanilla), `connectProjectSync(client, store)` (patches in revision order, history, close; resync on gap / `resyncRequired` / reconnection; reopen after a core restart), pure `mirrorFromSession` / `applyPatch`, selectors, `useProjectSelector(selector)`, `useProjectSync()` (for `app/editor` only); fixtures in `testing/` |
| `src/shared/shortcuts/` | `ShortcutRegistry` (`mod+shift+z` syntax, latest binding wins, ignored in text fields), `ShortcutProvider` (window `keydown`), `useShortcut(keys, run, enabled?)` |
| `src/shared/desktop/` | `DesktopBridgeProvider`, `useDesktopBridge()` |
| `src/shared/routing/` | `PROJECT_ROUTE_PATTERN`, `projectPath(id)`, `HOME_PATH` |
| `src/shared/errors/` | `errorMessageKey(error)` → `errors.<CODE>` / `errors.unknown`; `ERROR_CODES` (compile-time check that every protocol code is listed) |
| `src/shared/storage/` | `createNamespacedStorage(storage, namespace)` (errors swallowed) and `uiPreferencesStorage()` (`localStorage` under `se.ui.`): panel sizes and other machine preferences |
| `src/features/<name>/` | One folder per feature (`components/`, `hooks/`, `store/`, `services/`, `api.ts`, `index.ts`; create only what is needed). Today: `projects` (`HomePage`: recent grid with search and menus, `NewProjectDialog`, rename and trash dialogs; `api.ts`, `store/recent-projects-store.ts`, `hooks/use-project-actions.ts`, pure `services/` for the form, resolutions, labels, search), `media-bin` (`MediaBinPanel`), `viewer` (`ViewerPanel`), `inspector` (`InspectorView`), `ai` (`AiView`), `timeline` (`TimelinePanel`), each an empty state with a `TODO(prompt-NN)` |
| `src/i18n/` | `createI18n()` (synchronous, bundled resources), `locales/fr.json` (keys by area: `app`, `home`, `projects`, `editor`, `statusBar`, `errors`, `common`), `i18next.d.ts` (typed keys) |

## Rules and invariants
- Security: `contextIsolation: true`, `nodeIntegration: false`, `sandbox: true`; the CSP (meta tag written by `tooling/csp-plugin.ts`) only allows scripts from the bundle and connections to `127.0.0.1` in production; inline `<style>` elements are allowed for Radix and sonner (ADR-0013).
- UI: components, icons (`@se/ui/icons`) and toasts come from `@se/ui` only; ESLint rejects imports of `radix-ui`, `lucide-react`, `sonner`, `class-variance-authority` and `react-resizable-panels` here. Style with token utilities (`bg-panel`, `text-text-muted`…): the default Tailwind palette does not exist and raw colours fail `pnpm check`.
- The session token is never logged (the supervisor logs the URL and version only).
- Features import each other only through their `index.ts`; `shared/` never imports features or `app/` (enforced by `eslint-plugin-boundaries` in the root `eslint.config.js`).
- Keep hooks thin: put connection logic in plain functions taking a `CoreClient` (or a `Pick` of it) so it is testable in Node (see `watch-core-info.ts`).
- Components hold no business logic; every user-visible string goes through i18n (`useTranslation()` → `t('area.key')`). ESLint (`i18next/no-literal-string`, mode `jsx-only`) rejects JSX text and literal `aria-label`, `title`, `placeholder`, `label`, `description`, `content`, `alt`, `*Label`… in `src/**/*.tsx` (tests exempt; digits/punctuation and CAPITALS allowed). Pass `@se/ui` widgets their labels from `t()`, overriding their English defaults.
- Routing: React Router in library mode with a **hash router** (the renderer is a file in production). Add a screen as a route object in `app/routes.tsx`; inject what it needs through `AppRouteDependencies`.
- The project store is read-only for features: read with `useProjectSelector(selector)` (selectors return stored references, never new arrays), change the project by sending a command to the core; the patch updates the store. Only `app/editor` loads and forgets the project (`useProjectSync()`).
- Every error shown to the user is translated by code: `t(errorMessageKey(error))`. A new protocol code needs `errors.<CODE>` in `fr.json` and an entry in `ERROR_CODES`.
- Global shortcuts go through `useShortcut`; they never fire inside inputs, textareas, selects or editable content.
- Panel sizes are machine preferences (`uiPreferencesStorage()`), never project data. Dock `layoutId`s are a compatibility surface: renaming one resets users' sizes.
- `__APP_VERSION__` (from `package.json`) is injected by Vite `define` in `electron.vite.config.ts` and `vitest.config.ts`.
- All runtime dependencies are bundled by electron-vite, so they are `devDependencies` and the packaged app ships no `node_modules`.

## Allowed dependencies
`@se/protocol` (renderer; type imports only in `electron/`), `@se/engine`, `@se/ui` (renderer only), React, Zustand. Electron APIs only in `electron/`. See `docs/architecture.md` §4.

## How to add a translation key
1. Add it to `src/i18n/locales/fr.json` under its area (`editor.timeline.zoomIn`), in French, with typographic apostrophes (’) and ellipses (…).
2. Use it: `const { t } = useTranslation(); … t('editor.timeline.zoomIn')`. Interpolation: `"version": "Version {{version}}"` → `t('statusBar.version', { version })`.
3. `tsc` fails on an unknown key; `create-i18n.test.ts` fails on an empty value.

## How to test
- `pnpm vitest run --project desktop` (or `pnpm test` at the root): `electron/**/*.test.ts`, `tooling/**/*.test.ts`, `src/**/*.test.{ts,tsx}`. Tests run in Node; a test that renders React starts with a `/** @vitest-environment jsdom */` docblock (`vitest.setup.ts` adds jest-dom matchers, cleanup and jsdom stubs).
- Shell tests (`src/app/app-shell.test.tsx`) render `AppProviders` + a memory router from `createAppRoutes`, with a real `CoreClient` whose endpoint never resolves (stays « connecting », opens no socket), an idle bridge, a project store preloaded with `shared/project-store/testing/session-fixture.ts` for editor routes, and an in-memory layout storage. Inside docks, drive tabs with the keyboard: in jsdom the panel library takes clicks for resize grabs.
- Logic taking the core client is tested with small fakes of `send`/`on` (`project-sync.test.ts`, `load-editor-project.test.ts`); dialogs take their effects as props (`NewProjectDialog.test.tsx`).
- Manual: `pnpm dev` from the root (builds se-core, downloads the Electron binary on first run, opens the window).
- Built app with an isolated profile (as done for prompt 07 on macOS): `pnpm exec electron-vite build --outDir <tmp>/out`, add `<tmp>/out/package.json` with `{"type":"module","main":"main/index.js"}`, then `SE_CORE_PATH=<repo>/target/debug/se-core pnpm exec electron <tmp>/out --user-data-dir=<tmp>/profile --remote-debugging-port=9223` and drive it through `http://127.0.0.1:9223/json` (CDP). After rebuilding, reload with `Page.reload { ignoreCache: true }` (a plain reload keeps the old `index.html`).
- Packaging: `pnpm package` from the root → `apps/desktop/release/` (config in `electron-builder.yml`).
