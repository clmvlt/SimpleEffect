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

## 2026-11-06 — Prompt 03: Design system : fondations — Done

**Branch**: `feature/03-design-system-fondations`, merged into `dev` at `MERGE_SHA`

**Done**
- `@se/ui` tokens (`src/tokens/*.css`, Tailwind v4 `@theme static`, OKLCH): surfaces (`bg`, `panel`, `panel-raised`, `overlay`, `input`, `control`, `control-hover`, `border`, `border-strong`, `hover`, `scrim`), text, accent (bright yellow, dark `accent-foreground`; `accents.css` keeps the `lemon`/`yellow`/`gold` shades under `[data-accent]`), semantic (+ muted, `danger-solid*`), item kinds and AI (+ muted/strong), 4 px spacing, control heights `h-control-xs|sm|md`, radii, `shadow-overlay`, type scale 11–20 px, weights, leading, easing, durations, keyframes (overlay/content in-out, progress sweep), z-index layers. Default Tailwind palette and scales removed. Structure ready for a `[data-theme='light']` override.
- `src/styles.css` (the only stylesheet apps import): Tailwind, bundled Inter Variable + JetBrains Mono (fontsource), tokens, base styles (dark colour scheme, focus-visible safety ring, thin scrollbars, `timecode` utility, reduced motion).
- Utilities: `cn()` (clsx + tailwind-merge taught our custom tokens), `focusRing`, `floatingSurfaceClasses`, `Slot`/`Slottable`; `@se/ui/icons` entry (lucide re-export) + `Icon` (12/14/16/20 px, decorative unless labelled).
- 28 primitives in `src/primitives/<name>/{name.tsx, .stories.tsx, .test.tsx, index.ts}`: Button, IconButton, Icon, Input (+ shared `textFieldClasses`), Textarea, Label, Field (Slot-based wiring of id/aria-describedby/aria-invalid), Select, Checkbox (indeterminate), Switch, Slider (`thumbLabels`, range), RadioGroup, Tooltip (+ `TooltipProvider`, shortcut key caps), Popover, Dialog (sizes, opt-in close button via `closeLabel`), AlertDialog, DropdownMenu, ContextMenu (shared `menu/menu-styles.ts`), Tabs (`segmented`/`underline`), ToggleGroup, Separator, ScrollArea, Kbd, Badge (semantic + item kinds), Toast (`Toaster` + `toast`, sonner `unstyled` with tokens), Progress (determinate/indeterminate, tones), Spinner, Skeleton.
- Storybook 10 (`pnpm storybook`, Vite builder, dark manager/docs, a11y + docs addons, telemetry and update checks off): "Foundations/Principles" (MDX), "Foundations/Tokens" (generated from the token stylesheets: swatches with hex and contrast, type scale, spacing, radii, layers, motion), "Foundations/Theme" (the 3 yellow shades side by side), every primitive with variants and states.
- Tests (Vitest jsdom + Testing Library + user-event, 247 in `ui`): behaviour of Select, Dialog, DropdownMenu, Tabs, ToggleGroup, Slider (keyboard, focus, open/close) and of every other primitive; `src/stories.a11y.test.tsx` runs axe (WCAG 2.2 A/AA) on all 109 stories; `tokens/contrast.test.ts` asserts that every colour is inside the sRGB gamut and that text is AA on every surface for every accent shade (OKLCH → sRGB in `color-math.ts`). axe colour-contrast also ran once in a real browser on all 109 stories: no violation.
- Raw-colour guard: `scripts/check-raw-colors.mjs` (+ `scripts/lib/raw-colors.mjs`) in `pnpm check`, scanning `packages/ui/src` and `apps/desktop/src` outside `tokens/`; tested with fixtures (new Vitest project `scripts`). ESLint forbids `radix-ui`, `@radix-ui/*`, `lucide-react`, `sonner`, `class-variance-authority` in the renderer; react-hooks rules now cover `packages/ui`.
- Desktop: `styles.css` imports `@se/ui/styles.css`; the temporary status screen uses tokens; CSP `style-src` allows inline `<style>` (Radix, sonner).
- Docs: ADR-0013, `packages/ui/CLAUDE.md` (anatomy, tokens cheat sheet, how to add a component, accent), `docs/architecture.md` §4.1/§4.4/§13, glossary (design token, accent, primitive), desktop and root `CLAUDE.md`.

**Deviations from the prompt**
- Accent: the prompt asked for an indigo-violet default with 3 variants. Indigo, violet and blue were presented in the Theme story; the user chose **yellow**, then the "Jaune vif" shade (OKLCH 0.87 0.175 96) over "Citron" and "Or". Consequences: `accent-foreground` is dark; `warning` moved from amber to orange (hue 65) and `item-caption` from yellow to violet (hue 290), `ai` to magenta (335), so nothing else reads as the accent.
- Extra tokens beyond the list: `control`, `control-hover`, `hover`, `scrim`, `danger-solid`, `danger-solid-hover`, `danger-foreground` (secondary buttons, ghost hovers, modal backdrop and filled destructive buttons need them to stay AA).
- Z-index order is `panel < modal < dropdown < toast < tooltip` (the prompt listed `dropdown` before `modal`): a select or menu opened inside a dialog must float above it.
- Production CSP now allows inline styles (`style-src 'self' 'unsafe-inline'`), see ADR-0013.
- Accessibility in CI is checked with axe on portable stories in jsdom (no Playwright browser download); colour contrast is covered by the token test and was checked once in a real browser with axe on all stories.
- `scripts/not-yet-available.mjs` removed (no remaining placeholder script).

**Follow-ups**
- Prompt 04: mount `TooltipProvider` and `<Toaster label=…>` at the app root, add i18n strings, build `editor/` widgets on `textFieldClasses`, `floatingSurfaceClasses` and the tokens.
- ContextMenu keyboard opening (Shift+F10 / context-menu key) is not unit-tested: jsdom does not synthesise the `contextmenu` event.
- The renderer bundles every fontsource subset (cyrillic, greek, vietnamese…, ~0.6 MB of woff2): fine for a desktop app, revisit in prompt 29 if startup matters.

**Notes for the next agent**
- When a story's `component` is a Radix re-export, type the meta as `const meta: Meta<typeof X> = {…}`: `satisfies` makes `tsc -b` fail with TS2883 (inferred type names Radix internals). Exported Radix values re-assigned in our modules need an explicit type (`typeof SlotPrimitive.Root`) for the same reason (TS4023).
- Vitest blanks CSS imports; `packages/ui/vitest.config.ts` enables CSS for `src/tokens/*.css` so `?raw` imports work in token tests.
- Radix single ToggleGroup renders `radiogroup`/`radio`; RadioGroup selects on arrow key *down* (tests use `{ArrowDown>}{/ArrowDown}`).
- sonner hard-codes a huge z-index: the Toaster uses `z-(--z-toast)!`.

## 2026-11-06 — Prompt 02: Protocole typé et couche RPC — Done

**Branch**: `feature/02-protocole-rpc`, merged into `dev` at `c8ca8dc`

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
