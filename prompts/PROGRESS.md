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

## 2026-11-06 — Prompt 05: Domaine : temps et modèle de document — Done

**Branch**: `feature/05-domain-model`, merged into `dev` at `<merge>`

**Done**
- `se-domain::time`: `Flicks` (±`MAX_SAFE` = 2^53 − 1, checked `add/sub/mul/mul_f64`, `from/to_seconds_f64` nearest-flick, `from_frames`/`to_frames` with `RoundingMode` Floor/Ceil/Nearest, validating `Deserialize`), `FrameRate` (reduced fraction, 1–1000 fps, exact `frame_duration`, 9 standard constants, `is_ntsc`, `nominal_fps`, `Display` like `29.97`), `TimeRange` (half-open, duration ≥ 0 and end in range enforced at construction and deserialization, `contains`, `contains_range`, `intersects`, `intersection`, `shifted`), `parse_timecode` (`51s`, `51.5s`, `00:51`, `1:02.500`, `1:00:00`, `HH:MM:SS:FF`, `;FF` read as non-drop, leading `-`, bare numbers refused; errors `'1:75': seconds must be < 60`) and `format_timecode` (`HH:MM:SS:FF`, nominal-rate frame counting).
- Shared vectors `docs/test-vectors/flicks.json` and `timecode.json` (values cross-checked with an independent BigInt computation), run by `crates/se-domain/tests/test_vectors.rs`; proptest in `tests/time_properties.rs` (frames ↔ flicks at every standard rate with all roundings, floor containment, SMPTE round trip, `TimeRange` commutativity/containment/shift).
- `ids`: 10 UUIDv7 newtypes (`…, LinkGroupId, MarkerId`) over `uuid::Uuid` (`Copy`), strict hyphenated v7 parsing (`IdError`), string serde, `from_uuid`, `from_test_index` (feature `test-support`).
- `spatial` (`Vec2`, `DesignPixels`, `design_scale`), `color` (`Color`, `#rrggbb[aa]`).
- `document`: `Project`, `ProjectSettings`, `Asset`/`AssetKind`/`Decodability`, `MediaInfo`/`VideoStreamInfo`/`AudioStreamInfo`, `DerivedMedia`/`DerivedFile`/`DerivedStatus`, `Sequence`, `Format`/`FormatPreset`, `Track`/`TrackKind`, `Item`/`Origin`, `ItemKind` (`MediaClip` with `Reframe`, `TextItem`/`TextStyle`, `ShapeItem`/`ShapeGeometry`, `ImageItem`, `TemplateItem`/`TemplateParamValue`, `CaptionItem`), `Transform2D`, `Animatable`/`Keyframe`, `Easing`/`EasingPreset`, `Marker`/`MarkerColor`, `MotionTemplate`, `BrandKit`; `CURRENT_SCHEMA_VERSION = 1`.
- `validation`: `validate_project` → `Vec<Violation { code, path, message }>`, 15 `ViolationCode`s (table in `docs/architecture.md` §5.3), paths like `sequences[<id>].tracks[<id>].items[<id>].kind.speed`, at least one test per code.
- `test_support` (feature `test-support`): `example_project()` (valid, every item kind, keyframes, link group, AI origin, marker, template, brand kit), `example_ids`, `millis`, `range_ms`, `item`, `track`. insta snapshot of its JSON fixes the document format.
- `se-codegen` exports `Project`, `Violation`, `FormatPreset` roots (69 generated files); `Flicks` is `number & { readonly __brand: 'Flicks' }`, ids are `string`. `packages/protocol/src/document-types.test.ts` adds compile-time checks.
- Docs: ADR-0015 (spatial units) + index, `docs/architecture.md` §2/§3/§5.1–5.3, glossary (14 terms), backlog (drop-frame timecodes), CLAUDE.md of `se-domain`, `se-codegen`, `@se/protocol`.

**Deviations from the prompt**
- ADR number: the prompt asked for « ADR 0012 Spatial units », but 0012 was already taken (RPC conventions, prompt 02): it is **ADR-0015**.
- Design pixels are measured on the **shorter side** = 1080, not the height (user's choice, recorded in ADR-0015): 1 design px = 1 px in 16:9, 9:16, 1:1 and 4:5 at 1080, 2 px in 4K.
- `MAX_SAFE` is 2^53 − 1 (JavaScript's `Number.MAX_SAFE_INTEGER`) rather than 2^53, which JS cannot distinguish from 2^53 + 1.
- `MediaClip` has no separate `opacity`: it lives in `Transform2D` (the prompt listed both). Volume is `volume_db: Animatable<f64>` so automation is keyframes.
- Single-value invariants (flicks range, `TimeRange` duration ≥ 0, frame rates, ids, colours) are enforced by the types, including at deserialization; `validate_project` checks only cross-entity rules. Hence no violation code for « negative duration ».
- `FrameRate` only represents rates of 1–1000 fps with a whole number of flicks per frame; `VideoStreamInfo::frame_rate` is `None` for others (variable-frame-rate footage goes through a mezzanine).
- Extra validation codes beyond the prompt's list: `DUPLICATE_ID`, `ASSET_KIND_MISMATCH`, `INVALID_FORMAT`, `EMPTY_ITEM`, `EMPTY_KEYFRAMES`, `VALUE_OUT_OF_RANGE`.
- `Easing::Preset` holds an `EasingPreset` enum (ease-in/out/in-out) instead of a free string, so the engine never meets an unknown name. `Reframe` is a `kind`-tagged enum (`{"kind":"fit"}`) so prompt 26 can add a variant with data without changing the format.
- Timecode API is `parse_timecode` / `format_timecode` (functions), not a `Timecode` type; ADR-0004's mention of `Timecode::parse` predates this.
- `MediaInfo` has `video_stream: Option<…>` and `audio_streams: Vec<…>` (several audio streams in pro files) and `duration: Option<Flicks>` (images).
- Template parameter values cover text, number, boolean and colour; enum, font and asset parameters wait for prompt 24.

**Follow-ups**
- Prompt 06: operations must keep track items sorted by start (documented on `Track::items`, not validated), bump `Item::revision` and `Project::revision`, and run `validate_project` on the result.
- Prompt 07: `schema_version` migrations; `Asset::relative_hint` relinking; `missing` flag on open.
- Prompt 08: fill `MediaInfo`/`Decodability` (`TODO(prompt-08)` on per-stream decodability).
- Prompt 10: TypeScript flicks/timecode helpers must run `docs/test-vectors/*.json` (same error codes, camelCase variant names).
- Prompts 15, 23, 24, 25, 26: complete `TextStyle`/`ShapeItem`/easing presets, `CaptionItem`, `MotionTemplate`/params, `BrandKit`, `Reframe` (`TODO(prompt-NN)` in code).
- Link groups are not validated (e.g. members on different sequences): add a rule when operations create them (prompt 06/11).

**Notes for the next agent**
- ts-rs 12 (`serde-compat`) prints fake « failed to parse serde attribute » warnings for `#[serde(try_from)]`, `transparent`…: validated types implement `Deserialize` by hand (private `Raw…` struct + constructor) instead. Container-level `#[ts(type = "…")]` gives custom TS types (`Flicks`, ids, `Color`).
- Internally tagged enums need struct variants (`Static { value }`), not newtype variants around primitives; add `rename_all_fields = "camelCase"` for multi-word variant fields.
- Accept a changed snapshot with `INSTA_UPDATE=always cargo test -p se-domain` after reviewing the `.snap.new` (ignored by Git).
- clippy: `UUIDv7` is in `doc-valid-idents`; test files outside `#[test]` functions need `#![allow(clippy::unwrap_used/expect_used)]` (see `tests/*.rs`); `case_sensitive_file_extension_comparisons` fires on `ends_with(".pan")`.

## 2026-11-06 — Prompt 04: Design system : composants montage, shell, i18n — Done

**Branch**: `feature/04-design-system-editeur-shell`, merged into `dev` at `bf5f752`

**Done**
- `@se/ui` editor widgets (`packages/ui/src/editor/`, Storybook « Editor », 17 folders, stories + tests each): Panel (named region from its title or `aria-label`), Dock/DockPanel/DockHandle (`react-resizable-panels` 4.18, nestable, user resizes saved in an injected `DockLayoutStorage` under `dock-layout.v1.<layoutId>`, corrupt entries ignored), Toolbar (Radix toolbar: one tab stop, arrows inside, `pressed` toggles), PropertyGroup/PropertyRow (collapsible, keyframe slot, modified mark), ScrubInput (drag, Shift ×10, Alt ×0.1, arrows, double-click/Enter/digit typing with decimal comma, injected format/parse, **one `onCommit` per gesture**), TimecodeInput/TimecodeDisplay (injected `format`/`parse`, invalid → `aria-invalid`, Escape restores), ColorField (OKLCH sliders with live gradients, OKLCH and hex fields, EyeDropper when present, recent colours, `onChange`/`onCommit`), KeyframeButton (3 states, prev/next), EasingCurveEditor (draggable + arrow-key handles, presets linear/ease-in/ease-out/ease-in-out/back/hold, animated preview), TransportBar, Meter (dBFS zones, peak lines, pure `updatePeakHold`), AspectRatioPicker (16:9, 9:16, 1:1, 4:5, custom with thumbnails to scale), StatusDot, EmptyState, KeyboardShortcut (⌘⌥⇧ on Apple, localisable names elsewhere), AiTaskCard (5 statuses, log, error, cost, actions), PromptComposer (auto-growing, removable scope chips, Backspace removes the last, Ctrl/⌘+Enter sends, model/effort slots).
- Shared helpers: `usePointerDrag` (pointer capture, per-move deltas so modifiers can change mid-drag, one `onEnd` per press), `lib/platform.ts` (the only OS check). Colour maths gains `srgbToOklch` and `parseHex`. New token `animate-easing-preview`. Storybook wraps stories in a `TooltipProvider`.
- Renderer: i18next + react-i18next (`src/i18n/`, synchronous bundled `fr.json` grouped by `app`, `home`, `editor`, `statusBar`, `common`; keys typed through `CustomTypeOptions`), `eslint-plugin-i18next` `no-literal-string` (`jsx-only`, user-visible attributes listed, tests exempt; checked with deliberate violations). React Router 8 in library mode, hash router: `/` home (empty state, disabled « Nouveau projet », dev-only « Ouvrir l’éditeur » preview link), `/project/:projectId` editor (Médias | Viewer | Inspecteur+IA tabs over Timeline, every panel an explicit empty state), status bar (core connection as a polite `status`, « IA non configurée », version from `__APP_VERSION__`). `AppProviders` mounts `I18nextProvider`, `CoreClientProvider`, `TooltipProvider` and the `Toaster`. Panel sizes go to `localStorage` under `se.ui.` (`shared/storage/`). The temporary connection screen of prompts 01–02 is gone.
- Features created with their panel only: `projects`, `media-bin`, `viewer`, `inspector`, `ai`, `timeline` (each with a `TODO(prompt-NN)`).
- Tests: 461 in `ui` (axe on every story, all new stories included), 40 in `desktop` (shell routes and panels, saved layout restored, status bar, i18n completeness, storage adapter, connection indicator). ESLint forbids `react-resizable-panels` in the renderer.
- Verified in the built app (Electron, isolated profile, CDP): home and editor render, dragging handles and arrow keys on a focused handle resize panels, sizes are saved and identical after quitting and relaunching; core connects (« Cœur connecté »).
- Docs: ADR-0014, `docs/architecture.md` §4.1–4.2, glossary (editor widget, panel/dock, layout id, gesture commit, shell, translation key), `packages/ui/CLAUDE.md` (editor inventory, conventions, testing gotchas), `apps/desktop/CLAUDE.md` (routing, i18n, how to add a key, tests).

**Deviations from the prompt**
- Panel-size persistence uses our own `dock-layout-storage.ts` instead of the library's `useDefaultLayout`, so loading and saving are pure, unit-tested and save only user-made resizes (`requestedLayout`).
- `EasingCurveEditor` uses its own presentational `EasingCurve` type (`cubicBezier` | `hold`): the domain easing type arrives with prompt 15 and the app will map it.
- `ColorField` edits in OKLCH with three gradient sliders rather than a 2D area (keyboard-accessible out of the box); its value is a `#rrggbb` string (no alpha yet).
- Meter peak hold is computed by a pure helper the audio engine calls (`updatePeakHold`), not inside the component: values stay in props as the prompt asks, and the widget has no clock.
- A dev-only link on the home screen opens the editor (`/project/preview`) so the shell can be tried before projects exist (TODO(prompt-07)).
- Optional props of editor widgets are typed `?: T | undefined` for `exactOptionalPropertyTypes` consumers.

**Follow-ups**
- Prompt 07: enable « Nouveau projet », load the project of `/project/:projectId`, remove the preview link.
- Prompts 08/10/12/15/20: fill the panels (media bin, timeline, viewer + TransportBar, inspector property groups, AI composer and task cards) with the widgets of this prompt; send document commands from `onCommit` only (ADR-0014).
- `ColorField` has no alpha channel; `ScrubInput` has no expression evaluation (`=10*2`) nor pointer-lock infinite drag: add if a property needs them.
- Colour contrast of the new stories was not re-run in a real browser (axe in jsdom skips it); they use existing token pairs already covered by `contrast.test.ts`.

**Notes for the next agent**
- jsdom has no layout: `react-resizable-panels` sees 0 px, so resizing cannot be unit-tested, and clicks inside a `Dock` are taken for resize grabs (pointer-down default prevented): use the keyboard for tabs/buttons inside docks in tests. Do not mock `offsetWidth` to fix it: the library then fails to find panels.
- A shell test can use a real `CoreClient` whose endpoint resolver never resolves: it stays « connecting » and opens no socket (`app-shell.test.tsx`).
- To try the built app without disturbing a running `pnpm dev`: `electron-vite build --outDir <tmp>/out`, add a `package.json` with `"type": "module"` there, then `electron <tmp>/out --user-data-dir=<tmp>/profile --remote-debugging-port=9223` with `SE_CORE_PATH` pointing at `target/debug/se-core.exe`; Electron honours `--user-data-dir`.
- A running `pnpm dev` locks `target/debug/se-core.exe` on Windows, which makes `cargo test` in `pnpm check` fail with « Accès refusé »: close the app or use another `CARGO_TARGET_DIR`.

## 2026-11-06 — Prompt 03: Design system : fondations — Done

**Branch**: `feature/03-design-system-fondations`, merged into `dev` at `66efd7f`

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
