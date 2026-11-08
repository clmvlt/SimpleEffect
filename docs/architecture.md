# Architecture — SimpleEffect

> Authoritative technical reference for humans and AI agents working on this codebase.
> Decisions and their rationale live in `docs/adr/`. Domain vocabulary lives in `docs/glossary.md`.
> When the code and this document disagree, fix one of them in the same change.

## 1. System overview

SimpleEffect is a local, AI-first, non-linear video editor for Windows, macOS and Linux.
It runs as three kinds of processes:

```
┌──────────────────────────── Electron app ─────────────────────────────┐
│ Main process (TypeScript)              Renderer (React)                │
│  - windows, menus, native dialogs       - feature UI built on @se/ui   │
│  - spawns and supervises se-core        - @se/engine: decode (WebCodecs│
│  - preload exposes core URL + token,      via Mediabunny), composite   │
│    OS file paths (drag & drop)            (PixiJS), audio (Web Audio), │
│  - NO business logic                      export worker                │
│                                         - @se/protocol typed WS client │
└───────────────┬───────────────────────────────┬───────────────────────┘
                │ spawn + stdout handshake       │ WS /rpc · HTTP /media
                ▼                                ▼
┌──────────────────────────── se-core (Rust) ───────────────────────────┐
│ axum on 127.0.0.1:<random port>, bearer token, Host header check       │
│   /rpc    WebSocket: commands (request/response) + events (push)       │
│   /media  HTTP with Range: originals, mezzanines, proxies, thumbnails, │
│           waveform peaks, uploads of rendered chunks                   │
│   /mcp    MCP streamable HTTP endpoint for AI agents (per-task token)  │
│ se-app: ProjectActor (single writer), use cases, JobQueue, TaskScheduler│
│ adapters: se-store · se-media · se-transcribe · se-vision · se-agents  │
│           · se-export · se-mcp                                         │
└───────────────┬───────────────────────────────┬───────────────────────┘
                │ spawn per AI task              │ spawn per job
                ▼                                ▼
   claude -p --output-format stream-json    ffmpeg · ffprobe · whisper-cli
   (or another AI CLI) ──calls back──► /mcp
```

Key properties:
- **Local only.** No runtime network access except the AI CLI process itself and explicit, user-approved downloads of transcription models and of the GPU pack (ADR-0031).
- **AI acts only through MCP tools.** Agents have no built-in tools (no shell, no file edits).
- **Single writer.** All document mutations go through `ProjectActor` as `Operation`s.
- **WYSIWYG.** Preview and export run the same `@se/engine` code.
- **Rust is the source of truth for types.** TypeScript types are generated.

## 2. Repository layout

```
SimpleEffect/
├─ CLAUDE.md                 root rules for AI agents (read first)
├─ GUIDE-GIT.md              branches, promotion and release commands (French, user-facing)
├─ .github/workflows/        ci.yml (pnpm check) · release.yml (installers on v* tags)
├─ docs/
│  ├─ architecture.md        this file
│  ├─ analyse-stack.md       stack analysis (French, user-facing)
│  ├─ glossary.md            domain vocabulary
│  ├─ ai-tools.md            MCP tool catalog = contract with AI agents (created in prompt 18)
│  ├─ backlog.md             ideas not yet scheduled
│  ├─ test-vectors/          JSON vectors shared by Rust and TypeScript tests (flicks, timecodes, interpolation)
│  └─ adr/                   architecture decision records
├─ prompts/                  roadmap: one prompt per feature, executed in order (French)
├─ apps/
│  └─ desktop/
│     ├─ electron/           main + preload + shared bridge contract (TypeScript, no business logic)
│     ├─ tooling/            build-time helpers (Content-Security-Policy plugin)
│     ├─ electron-builder.yml  installer configuration (`pnpm package`)
│     └─ src/                renderer React app
│        ├─ app/             providers, routing, shell layout
│        ├─ features/        one folder per feature (see §4.3)
│        ├─ shared/          cross-cutting hooks/utils (core client, time formatting…)
│        └─ i18n/            locales (fr default)
├─ crates/
│  ├─ se-domain/             pure document model + operations (no IO, no async)
│  ├─ se-protocol/           RPC commands/responses/events DTOs, TS export
│  ├─ se-app/                use cases, ports (traits), ProjectActor, JobQueue, TaskScheduler
│  ├─ se-store/              persistence adapter (project folder, JSON, SQLite)
│  ├─ se-media/              ffprobe/ffmpeg adapter: probe, jobs, analysis
│  ├─ se-transcribe/         whisper.cpp adapter
│  ├─ se-vision/             ONNX Runtime adapter: face/subject detection
│  ├─ se-fonts/              font catalogue adapter: bundled fonts (embedded) + installed fonts (fontdb)
│  ├─ se-agents/             AI CLI providers (claude_code first), stream parsing
│  ├─ se-mcp/                MCP server: tool definitions → use cases
│  ├─ se-export/             FCP7 XML, OTIO, SRT, render orchestration
│  ├─ se-core/               binary: composition root, axum server
│  ├─ se-codegen/            dev tool: exports Rust types to TypeScript (`pnpm codegen`)
│  └─ se-devtools/           dev tools: large test projects (`pnpm generate-test-project`)
├─ packages/
│  ├─ protocol/              @se/protocol: generated TS types + typed RPC client
│  ├─ engine/                @se/engine: evaluation, decode, composite, audio, export (golden frames in test/golden/)
│  └─ ui/                    @se/ui: design system (shadcn-style, editor components)
├─ scripts/                  Node scripts behind pnpm commands (check, dev, build, package, set-version, codegen…)
└─ resources/bin/<platform>/ ffmpeg, ffprobe, whisper/ (whisper-cli + libraries) (gitignored, fetched)
```

Platform identifiers: `win32-x64`, `darwin-arm64`, `darwin-x64`, `linux-x64`.

## 3. Rust crates and dependency rules

```
se-domain  ◄── se-protocol
    ▲              ▲
    │              │
  se-app ◄─────────┤   (ports = traits defined in se-app)
    ▲              │
    ├── se-store   │
    ├── se-media   │
    ├── se-transcribe
    ├── se-vision  │
    ├── se-fonts   │
    ├── se-agents  │
    ├── se-export  │
    └── se-mcp ────┘
          ▲
       se-core (composition root: depends on everything, nothing depends on it)
```

| Crate | May depend on | Must NOT |
|---|---|---|
| `se-domain` | serde, serde_json (embedded preset library, lenient `Animatable` reads), thiserror, uuid, ts-rs (derive only); dev: proptest, insta | do IO, use async/tokio, read the clock, spawn anything |
| `se-protocol` | se-domain, serde, serde_json, ts-rs | contain logic |
| `se-app` | se-domain, se-protocol, tokio, async-trait, tracing | touch `std::fs`, `std::process`, network, concrete adapters |
| adapters | se-app, se-domain, se-protocol (DTOs they write, e.g. `ThumbnailSheet`, `Task`, `AgentEvent`), their IO libraries | depend on another adapter crate |
| `se-mcp` | se-app, se-domain, se-protocol (DTOs read through se-app), rmcp, schemars, http, base64 | mutate documents except through se-app use cases |
| `se-core` | everything | contain domain logic |
| `se-codegen` | se-protocol, se-domain, ts-rs | be depended on, ship in the app |
| `se-devtools` | se-domain, se-app, se-store, se-media, tokio, uuid | be depended on, ship in the app |

Adapters implement **ports** declared in `se-app::ports`: `ProjectRepository` (+ `ProjectStore`, one open and locked project, and its `AnalysisStore`), `AppLibraryRepository`, `MediaProber`, `MediaFiles` (existence, fingerprints, folder size and removal, small file reads), `MediaJobRunner`, `FrameExtractor`, `MediaTools` (ffmpeg folder, version, hardware encoders), `FontCatalog` (bundled and installed fonts, se-fonts), `Transcriber`, `SubjectDetector`, `AgentProvider` + `AgentWorkspaces` (se-agents), `TaskStore` (`ProjectStore::tasks()`, se-store), `Exporter`, `EventSink`, `Clock`, `IdGenerator`, `TokenSource` (MCP run tokens), `RendererLink` (frames drawn by the renderer). Small ports with no adapter crate (`Clock`, `IdGenerator`, `TokenSource`, `EventSink` and `RendererLink` over the event bus) are implemented in se-core (`system_ports.rs`).
Every port has an in-memory/fake implementation in `se-app` (feature `test-support`) so use cases are tested without IO.

## 4. TypeScript packages and rules

### 4.1 Packages

| Package | Role | Must NOT |
|---|---|---|
| `@se/protocol` | generated types (`src/generated/`, never hand-edited) + typed client (`CoreClient`: commands, event subscriptions, reconnect) | import React or the engine |
| `@se/engine` | `evaluate`, decoders, compositor, audio, export worker | import React or `@se/ui`; do any network call except to se-core |
| `@se/ui` | presentational design system + generic editor widgets | know about the domain model or the protocol |

**`@se/ui` in practice** (ADR-0009, ADR-0013): three entry points — `@se/ui` (components and `cn`), `@se/ui/icons` (lucide re-export, drawn with `Icon`) and `@se/ui/styles.css` (Tailwind v4, bundled fonts, tokens, base styles; the app imports only this stylesheet). Design tokens are CSS variables in `packages/ui/src/tokens/*.css` (`@theme`, OKLCH), the only place where colour literals may appear; `scripts/check-raw-colors.mjs` (in `pnpm check`) rejects them anywhere else in `packages/ui/src` and `apps/desktop/src`, and the default Tailwind palette is removed. Layers: `tokens/` → `primitives/` (Radix wrappers, prompt 03) → `editor/` (domain-agnostic editor widgets, prompt 04: Panel, Dock, Toolbar, PropertyGroup/Row, ScrubInput, Timecode, ColorField, KeyframeButton, EasingCurveEditor, TransportBar, Meter, AspectRatioPicker, StatusDot, EmptyState, KeyboardShortcut, AiTaskCard, PromptComposer). Widgets take every label as a prop (English defaults for generic accessible names) and follow the **gesture commit** rule: `onChange` while a value moves, `onCommit` once per gesture, so one gesture is one undo step (ADR-0014). Storybook (`pnpm storybook`) documents every component; every story is checked by axe in `pnpm test`, and text tokens are tested for WCAG AA contrast.

### 4.2 Renderer app (`apps/desktop/src`)

- Features depend on `@se/ui`, `@se/engine`, `@se/protocol` and `shared/`.
- A feature imports another feature **only through its `index.ts`** (enforced by ESLint boundaries).
- Components contain no business logic: logic lives in hooks, stores and services inside the feature.
- All user-visible strings come from i18n keys (French default): i18next + react-i18next, resources bundled in `src/i18n/locales/fr.json`, keys typed for `t()`; `eslint-plugin-i18next` rejects literal JSX text and user-visible attributes (ADR-0014).
- `app/` is the composition root and the shell: `main.tsx` creates the core client, the i18n instance and a **hash router** (React Router, library mode) from `createAppRoutes()`: `/` home (`features/projects`), `/project/:projectId` editor. The editor layout (`app/editor/EditorPage.tsx`) nests `Dock`s around the feature panels — media left, viewer centre, inspector and AI tabs right, timeline bottom — and the status bar shows the core connection, AI availability and version.
- Panel sizes are machine preferences saved in `localStorage` under `se.ui.` (`shared/storage/`), never in projects.
- `shared/` modules created by prompt 07: `project-store/` (read-only Zustand mirror of the open project, fed by the core, §5.6), `shortcuts/` (named shortcut actions, see below), `desktop/` (the Electron bridge through context: folder picker, reveal in folder), `routing/` (screen paths), `errors/` (one translation key per protocol error code, `errors.<CODE>` in `fr.json`).
- `features/media-bin` (prompt 08): the media panel — import by button (native dialog) or by dropping files from the OS, grid/list with posters, hover scrubbing through the thumbnail sheet and waveforms (fetched from `/media`), kind filter and accent-insensitive search, job progress from the job events (`store/jobs-store.ts`), details of the selected asset, « Manquant » badge and « Relier… ». It reports decodability for assets with an `unknown` stream (`@se/engine/capabilities`). Media URLs come from `useCoreEndpoint()` (`shared/core`) and carry the token as `?t=`.
- **Keyboard shortcuts** (`shared/shortcuts/`, prompt 10): `SHORTCUT_ACTIONS` is the single table of named actions (`undo`, `markIn`, `nextFrame`, `zoomToFit`…), each with its contexts (`global`, `timeline`, `viewer`), default keys (`mod+shift+z`; `appleKeys` when macOS differs) and whether holding repeats it. Components bind handlers with `useShortcutAction(action, run)`; the `ShortcutRegistry` runs the latest handler of the action a key press matches, when the action is global or belongs to the active context — the `ShortcutRegion` last clicked or focused (`data-shortcut-context`; the timeline claims it on mount). Keys typed in text fields are ignored, symbols ignore Shift (layouts differ), ⌥ letters on macOS are read from `code`; the registry also tracks held keys (`isKeyHeld`, released on key up or window blur) for combinations like K+L. `findShortcutConflicts` must stay empty for the table (tested); tooltips show keys with `useShortcutHint(action)`, and « Raccourcis clavier » (`?`, `app/shortcuts/`) lists every action by context.
- **Time in the renderer** (`shared/time/`, prompt 10): branded flicks, frame rates, rounding modes, `parseTimecode`/`formatTimecode` and pixels at a zoom, mirroring `se_domain::time` and replaying `docs/test-vectors/*.json` for parity. `shared/canvas/` gives canvases cached token values (`tokenValue`) and `prepareCanvas` (resize only when the size changes).
- **Transcription** (prompt 21, ADR-0031): « Réglages → Transcription » (`features/settings`) shows whisper.cpp, the GPU and where the next transcription runs, the GPU pack (Windows), the models with the recommended one, downloads behind a consent dialog (size, source, SHA-256), imports, the model used and the options; live from `transcriptionStatusChanged`. The « Transcription » tab (`features/transcript`) shows the active sequence's words — mapped locally from the assets' transcripts (`getTranscript`, cached per `transcribedAt`) through its sound clips by `shared/transcript` (mirror of `se_domain::sequence_transcript`) — or the selected media's; the word under the playhead is highlighted outside React, a click seeks, a drag marks in/out, search steps through matches, unsure words are underlined, untranscribed media get « Transcrire » with their job's progress, and a popover sets the project's language and vocabulary. The media bin shows « Transcrit » and offers « Transcrire » in its menus.
- **Sequences** (`features/sequences`, prompt 16, ADR-0026): the timeline's sequence tabs (always shown) open, from their context menu or the « … » button, rename (also by double click), « Format de la séquence… », « Créer une version… », « Dupliquer » and « Supprimer… » (confirmation). The format dialog reuses the format fields of the new-project form (`shared/format`) and locks the frame rate of a sequence holding items; after a change the items newly out of the frame are listed in a toast with « Sélectionner ». The viewer draws the guides chosen per sequence (safe areas of TikTok, Reels, Shorts, YouTube 16:9, thirds, centre cross) and, in reframing mode (double click on a filling clip, or the inspector), moves a clip's kept area by dragging its picture.
- **AI** (`features/ai`, prompt 17, ADR-0027): `AppLayout` keeps an app-wide AI status store in step with the core (`watchAiStatus`: `getAiStatus` at each connection, then `aiStatusChanged`); the status bar's `AiStatusIndicator` shows the provider green (logged in), orange (logged out — the tooltip gives the terminal login steps from the provider's `command` and `loginCommand`) or grey (not installed) and opens « Réglages → IA » (`openSettings('ai')` of `features/settings`): CLI path and version, login state with « Actualiser », a custom CLI path (bridge `chooseFile`), the installation guide (bridge `openExternal`, `https:` only), default model and effort, tasks running at once. Prompt 20 (ADR-0030): the « IA » tab (`AiView`) lists the project's tasks — running first, then queued, then the most recent — as live `AiTaskCard`s (status, number « #3 », elapsed time, scope that reveals its range on the timeline, the latest log lines with tool calls as compact lines that select the items they touched, failures translated by code, tokens and cost, actions cancel / retry / follow-up inline / revert with its conflicts listed first (`previewTaskRevert`) / « Voir les modifications » / complete log) above the composer (`TaskComposer` on `PromptComposer`: scope chips derived by `deriveScope` — a time range read in the French text by `findTimePhrase`, the item of « Refaire avec l'IA », the selection, the in/out span, or the whole sequence —, a « Portée » popover for the sequence, the whole sequence and the tracks, model and effort from the provider's capabilities, ↑/↓ prompt history saved under `se.ui.ai.v1`, and for a redo « reprendre la session » widening the task's scope to the item). `useAiTaskSync` (mounted by `EditorLayout`) mirrors the open project's tasks into `shared/ai-tasks` (`listTasks` per connection, `taskUpdated`, `projectPatched`: items of AI patches pulse, running tasks whose scope a user patch touched are flagged), announces each finished run (toast with « Voir », system notification through the bridge when the window is not focused), binds Ctrl/⌘+I (`askAi`) and applies composer requests. The timeline reads the mirror: shaded scopes of active tasks labelled « IA #3 en cours » (with a mark after a user edit), the brief glow of items an AI patch changed, the « IA » badge of AI-made items (« Créé par la tâche #3 — voir »), « Refaire avec l'IA… » in the item menu.
- **Timeline** (`features/timeline`, prompt 10, ADR-0020): shows the active sequence (tabs, « + » creates one). Tracks are DOM rows — video tracks above in reverse document order (they stack first = bottom), audio below — with only the items in view rendered (binary search over each track's sorted items) as `ClipShell`s drawing thumbnails from the sheet and waveforms in a canvas at the peak level matching the zoom. Horizontal scroll and zoom are virtual (Ctrl/⌘+wheel around the pointer, Maj+wheel or swipe, slider, fit, minimap), applied once per animation frame; the playhead moves by style from its own store. Stores: `timeline-view-store` (per sequence: zoom, scroll, in/out points; track heights; snapping; active sequence — saved per project in `localStorage`, never in the project), `playhead-store`, `selection-store`. Track and marker edits are one protocol command each (one undo step). Geometry lives in pure `services/`. Editing (prompt 11, ADR-0021): one pointer handler on the lanes selects (click semantics apply on release, so a drag moves the whole selection), draws the rectangle or starts a gesture controller (`hooks/gestures/`: move with linked items — Alt copies, Ctrl/⌘ inserts —, edge trim — Ctrl/⌘ ripples —, gap handle); media dragged from the bin (`shared/drag`) drops as `placeMedia`; shortcuts cut (`S`, Ctrl/⌘+K), delete (Maj: ripple), copy/cut/paste (internal clipboard, pasted at the playhead on the active track), nudge (`,` `.`, `<` `>`) and toggle snapping (`N`); a context menu on the lanes offers « Couper ici », « Dupliquer », « Lier/Délier », « Propriétés » (inspector tab through `shared/side-panel`), « Afficher dans le panneau Médias » (`revealAsset` from `features/media-bin`) and deletes. Gestures snap by whole frames to the playhead, item edges, markers and in/out points, preview in `gesture-store` and send one command on release.
- **Viewer** (`features/viewer`, prompt 12): the frame of the active sequence at the playhead. The engine lives outside React: `ViewerEngineController` mounts a fresh canvas in the picture element and starts `@se/engine/preview` on it (imported lazily; a missing WebCodecs shows « Aperçu indisponible »), and `useViewerEngine` forwards the scene (built from the mirror's entities), media URLs (`createMediaLocator`: `/media/assets/<id>/<variant>?t=…&v=<version>`), options and canvas size, and follows `playheadStore` directly. Zoom (Ajuster, 50 %, 100 %, 200 %), « Lire les proxys », the transparency checkerboard, the performance HUD, loop and audio scrub are machine preferences (`se.ui.viewer.v1`); the canvas draws at most the sequence resolution; a spinner appears when decoding takes more than 150 ms; a badge reports media that could not be drawn. Playback (prompt 13, ADR-0023): `PlaybackSession` (outside React, started/stopped by `usePlaybackSession`) runs the engine's `PlaybackController` on the audio clock of `@se/engine/audio` (imported lazily; silent on the performance clock without Web Audio), writes the playhead store (a move made elsewhere is a seek, with an audio scrub snippet when paused), switches the picture engine to playback decoding and publishes the rate in `shared/playback` (the timeline pages while playing). The transport bar under the picture shows the timecode and duration (leaf components: the only per-frame re-renders), go to start/end, frame steps, play/pause, stop, loop, the master meter (RMS bars, held peaks) and the audio scrub toggle; the HUD over the picture shows real frames per second, dropped frames, readers and queued decodes, caches and the prefetch window. Time and playback shortcuts work there too (context `viewer`): Space, J/K/L (K held + J/L steps), Ctrl/⌘+L.
- The editor (`app/editor/`) loads the project named by `/project/:projectId` (`getProjectSnapshot`, or reopens it from the recent list after a reload or a core restart), shows its name and active sequence, binds Ctrl/⌘+Z and Ctrl/⌘+Maj+Z to `undo`/`redo`, and closes the project when the user goes back home.

### 4.3 Features

`projects` (home, create/open) · `media-bin` · `timeline` · `sequences` (rename, format, versions, removal, out-of-frame warning; prompt 16) · `viewer` · `inspector` · `ai` (prompt composer, tasks panel, scope overlays) · `transcript` (prompt 21: the « Transcription » tab of the right panel; prompt 22: text-based editing, struck cut passages, the « Silences » and « Hésitations » views) · `captions` (prompt 23: the « Sous-titres » tab — generation, SRT import/export, inline editing, styles; ADR-0033) · `templates` (library, brand kit) · `export` · `settings` (the « Réglages » dialog opened by a gear button on the home screen and in the editor header, or by `openSettings(section)`; sections Médias since prompt 09, IA since prompt 17 and Transcription since prompt 21).

Feature folder layout:

```
features/<name>/
├─ components/   React components (presentational composition of @se/ui)
├─ hooks/        React hooks (glue between stores, core client and components)
├─ store/        Zustand store(s) for feature UI state
├─ services/     pure logic specific to the feature
├─ api.ts        protocol calls used by this feature
└─ index.ts      public API of the feature
```

### 4.4 Electron (`apps/desktop/electron`)

- `main/`: app lifecycle, windows, menus, dialogs, `CoreSupervisor` (spawn se-core, read the handshake line `{"port":…,"token":…,"version":…}` on stdout, restart on crash with exponential backoff, stop it by closing its stdin when the app quits).
- `preload/`: exposes a minimal typed API via `contextBridge` as `window.simpleEffect` (also `showNotification({ title, body, tag })` and `onNotificationClick(listener)` since prompt 20: system notifications whose click focuses the window and gives the tag back): `getCoreConnection()`, `chooseFolder({ title, buttonLabel, defaultPath? })` (native folder picker), `chooseFiles({ title, buttonLabel, filterName, extensions, allFilesName })` (native multi-file picker), `showInFolder(path)`, `getPathForFile(file)` (absolute path of a file dropped from the OS, `webUtils.getPathForFile`). Main validates every IPC argument (`ipc-arguments.ts`). Built as CommonJS because sandboxed preloads cannot be ES modules.
- `shared/`: the bridge contract (types and IPC channel names) bundled into main, preload and renderer.
- Security: `contextIsolation: true`, `nodeIntegration: false`, `sandbox: true`, no pop-ups or navigation, strict CSP written as a meta tag at build time (`tooling/`): production connects only to `127.0.0.1`, scripts come only from the app bundle; inline `<style>` elements are allowed because Radix and sonner insert them (ADR-0013).

**se-core process contract**: stdout carries exactly one handshake line; stdin stays open and its end-of-file means "shut down" (so the core dies with the app, even if the app crashes); `SE_DATA_DIR` (set by the supervisor to `app.getPath('userData')`) is the app data folder, and logs go to daily files in `<SE_DATA_DIR>/logs/` (7 kept, filter `SE_LOG`) plus stderr in debug builds. The packaged binary lives in `resources/bin/`, the development one in `target/debug/`.

## 5. Core concepts

### 5.1 Time

Module `se_domain::time` (ADR-0004).

- `Flicks(i64)`: 705,600,000 flicks per second. Exact for all common frame rates (incl. NTSC 1001 rates) and sample rates. Values stay within ±`Flicks::MAX_SAFE` = 2^53 − 1 (JavaScript's `Number.MAX_SAFE_INTEGER`, ~147 days): constructors and deserialization reject anything else. Arithmetic is checked (`checked_add`, `checked_sub`, `checked_mul`, `checked_mul_f64` for speeds), never panicking. Negative values exist (deltas); documents reject negative positions in validation.
- Conversions: `from_seconds_f64` / `to_seconds_f64` (nearest flick, halves away from zero), `from_frames(n, rate)` / `to_frames(rate, RoundingMode::{Floor, Ceil, Nearest})` (exact).
- `FrameRate { numerator, denominator }` (e.g. 30000/1001), stored reduced. Only rates between 1 and 1000 fps whose frame lasts a whole number of flicks are representable, so `frame_duration()` is exact (23,543,520 flicks at 29.97). Constants `FPS_23_976` … `FPS_120`, `STANDARD`, `is_ntsc()`, `nominal_fps()` (30 for 29.97).
- `TimeRange { start, duration }`: half-open `[start, end)`, `duration ≥ 0` and an end within range enforced at construction and deserialization. `contains`, `contains_range`, `intersects` (touching ranges do not), `intersection`, `shifted`. Empty ranges intersect nothing.
- `parse_timecode(text, Option<FrameRate>)` accepts `51s`, `51.5s` (≤ 9 decimals), `00:51`, `1:02.500`, `1:00:00`, `00:00:51:12` (SMPTE, needs a frame rate) and a leading `-`; a bare number is refused as ambiguous. SMPTE timecodes count frames at the nominal rate (non-drop: `00:00:01:00` at 29.97 is frame 30, at 1.001 s); the drop-frame notation `00:00:51;12` is accepted and read as non-drop for now. Errors quote the input: `'1:75': seconds must be < 60`. `format_timecode(time, rate)` writes `HH:MM:SS:FF` (displayed frame, i.e. rounded down; `-` prefix for negative times).
- Shared vectors `docs/test-vectors/flicks.json` and `timecode.json` are run by the Rust tests (`crates/se-domain/tests/test_vectors.rs`) and must be run by the TypeScript helpers (prompt 10) for parity.
- In TypeScript flicks are a branded `number` (generated: `type Flicks = number & { readonly __brand: 'Flicks' }`).
- Conversions happen only at boundaries: UI display, MCP tool input/output, export writers.

### 5.2 Identifiers

UUIDv7 newtypes over `uuid::Uuid` (`ProjectId`, `SequenceId`, `TrackId`, `ItemId`, `AssetId`, `TaskId`, `ChangesetId`, `TemplateId`, `LinkGroupId`, `MarkerId`, `TransactionId`; module `se_domain::ids`). They serialize as hyphenated lowercase strings (`string` in TypeScript); `parse` accepts only the hyphenated form of a version-7 UUID, with an actionable `IdError`. se-domain never generates ids: the `IdGenerator` port does and wraps them with `from_uuid`. Tests use `from_test_index(n)` (feature `test-support`), e.g. `00000000-0003-7000-8000-000000000000`.

### 5.3 Document model (summary — source of truth is `se-domain`)

```
Project { id, name, schema_version, revision, settings: ProjectSettings { default_format, transcription }, assets, sequences, templates, brand_kit: Option<BrandKit>, caption_styles: [CaptionStyle] }
Asset { id, original_path, relative_hint, kind: Video|Audio|Image, media_info: Option<MediaInfo>, fingerprint: Option<ContentFingerprint { size_bytes, modified_unix_ms, sample_hash }>, derived: DerivedMedia, analysis: AssetAnalysis { audio: Option<AudioSummary { integrated_lufs, loudness_range_lu, true_peak_dbtp, silence_count, silence_duration }>, scenes: Option<SceneSummary { change_count }> }, decodability: Decodability { video, audio: Unknown|Decodable|Undecodable|NoStream }, missing }
MediaInfo { duration: Option<Flicks>, container, video_stream: Option<VideoStreamInfo { codec, profile, level, codec_tag, pixel_format, bit_depth, chroma_subsampling, width, height, frame_rate, rotation }>, audio_streams: [AudioStreamInfo { codec, profile, channels, sample_rate }] }
DerivedMedia { mezzanine, proxy: Option<DerivedFile { relative_path, status: Pending|Ready|Failed{message} }> }
Sequence { id, name, format: Format { width, height, frame_rate, sample_rate }, tracks, markers }
Track { id, kind: Video|Audio, name, muted, solo, locked, hidden, volume_db, pan, items, transitions }
Transition { id, kind: CrossDissolve|AudioCrossfade, left_item, right_item, duration, alignment: Center|Start|End }
Item { id, revision, origin: Origin, timeline_range: TimeRange, link_group: Option<LinkGroupId>, kind: ItemKind }
ItemKind = MediaClip { asset_id, source_start, speed, transform, volume_db: Animatable<f64>, fade_in, fade_out, fade_in_shape, fade_out_shape: Linear|ConstantPower|Logarithmic, reframe: Fit | Fill { focus: Animatable<Vec2> } | Custom | Tracked { focus } }
         | Text(TextItem { content, style: TextStyle, transform, reveal: Animatable<f64> })
         | Shape(ShapeItem { geometry: Rectangle { width, height, corner_radius } | Ellipse { width, height } | Line { length, thickness }, fill: Animatable<Color>, stroke: Stroke, cover_frame, transform })
         | Image(ImageItem { asset_id, fit: Fit|Fill|Natural, transform })
         | MotionTemplate(TemplateItem { template_id, params: name → Text|Number|Boolean|Color })
         | Caption(CaptionItem { words: [CaptionWord { text, start, end (item-relative), emphasis }], styleId })
TextStyle { font_family, font_weight (1–1000), italic, font_size: Animatable<px>, color: Animatable<Color>, stroke: Stroke, shadow: TextShadow { enabled, color, blur, offset }, background: TextBackground { enabled, color, padding, corner_radius }, letter_spacing: Animatable<px>, line_height, align: Left|Center|Right, max_width: Option<px>, text_case: Normal|Uppercase }
Stroke { enabled, color: Animatable<Color>, width: Animatable<px> }   (visible width outside the glyphs or edge)
Transform2D { position, scale, rotation, anchor, opacity, blur }   (each Animatable)
Animatable<T> = Static { value } | Keyframed { keyframes: Vec<Keyframe<T>> }
Keyframe<T> { time: Flicks (item-relative), value: T, easing: Easing }
Easing = Linear | Hold | CubicBezier { x1, y1, x2, y2 } | Preset { name: EaseIn|EaseOut|EaseInOut }
Origin = User | AiTask { task_id }
CaptionStyle { id, preset?, fontFamily, fontWeight, italic, fontSize (design px), textCase, lineHeight, textColor, activeColor, emphasisColor, outline { enabled, color, width }, shadow { enabled, color, blur, offset }, background { enabled, color, padding, cornerRadius }, position (bottom centre, normalised), animation: none|wordHighlight|popPerWord|boxPerLine|revealPerWord, highlightSpoken, maxCharsPerLine (4–80), maxLines (1–4) }   (Project.captionStyles, schema version 8)
Marker { id, time, label, color: MarkerColor (Premiere's 8 colours) }
```

`FormatPreset` (`Hd16x9`, `Uhd16x9`, `Vertical9x16`, `Square1x1`, `Portrait4x5`, `Custom`) maps to dimensions; `Format::preset()` detects it. Templates and brand kits are minimal skeletons completed by prompts 24 and 25 (`TODO(prompt-NN)` in code); captions are complete since prompt 23 (§5.8, ADR-0033).

**Animatable properties** (ADR-0025): `AnimatableProperty` lists the 18 properties keyframes can drive — position, scale, rotation, anchor, opacity, blur (visual items), volumeDb (media clips), reframeFocus (clips that fill the frame, ADR-0026), fontSize, fillColor (text colour or shape fill), letterSpacing, reveal, strokeColor, strokeWidth (texts and shapes), shadowColor, shadowBlur, shadowOffset, backgroundColor (texts). `ItemKind::property(p)` / `property_mut(p)` give a typed slot (`PropertyRef`/`PropertyMut`: number, design pixels, pair, colour) or `None` when the kind lacks it; `SetKeyframes`, presets and validation use it. Disabled outlines, shadows and plates keep their values. `Animatable` reads a bare value as static (journals written before properties became animatable).

**Reframing** (ADR-0026): a clip's `Reframe` says how its picture fits the frame before the transform — `fit` (whole picture), `fill` (cropped to the frame around `focus`, a point of the picture normalised in it, independent of the frame's aspect), `custom` (natural size: one picture pixel per design pixel), `tracked` (a fill whose focus keyframes come from subject tracking, prompt 26). Documents and journals without a focus read the centre (schema version 6).

**Format changes and versions** (`se_domain::reformat`, ADR-0026): `format_change(project, sequence, format, adaptation)` gives the operations of an in-place change (`UpdateSequence` + `UpdateItem`s; items on locked tracks stay), `sequence_version(source, name, format, adaptation, ids)` an independent copy in another format (`copy_sequence` with fresh ids from a `SequenceIds`: link groups and transitions remapped, revisions reset). `FormatAdaptation { clipReframe: keep | fit | fill, fitTextWidths }` (texts wider than 90 % of the new frame get that wrapping width). Both refuse a frame-rate change once the sequence holds items (`FRAME_RATE_LOCKED`).

**Animation presets** (`se_domain::animation_preset`, ADR-0025): data in `crates/se-domain/assets/animation-presets.json` (format in `crates/se-domain/CLAUDE.md`), expanded into ordinary keyframes by `expand_preset` / the `ApplyAnimationPreset` operation; `preset_samples` gives each preset's expansion on a reference text for previews.

Serialization (`project.json` and the protocol): `camelCase` fields, enums with data internally tagged by `kind` (an item is `{ …, "kind": { "kind": "mediaClip", "assetId": … } }`), unit enums as `camelCase` strings, flicks as JSON numbers, ids and colours (`#rrggbb[aa]`) as strings, `None` as `null`. The insta snapshot in `crates/se-domain/src/document/snapshots/` of `test_support::example_project()` fixes the format: changing it is a document format change.

Spatial units (ADR-0015): positions normalised in the sequence frame (`(0,0)` top-left, `(1,1)` bottom-right), anchors normalised in the item's box, relative scales, clockwise degrees, opacity in `[0, 1]`; sizes in design pixels, i.e. pixels of a frame whose shorter side is 1080 (`Format::design_scale()` = shorter side / 1080).

Invariants. Single-value invariants are enforced by the value types (`Flicks` range, `TimeRange` duration, `FrameRate`, UUIDv7 ids, hex colours). Cross-entity invariants are reported by `validate_project(&Project) -> Vec<Violation { code, path, message }>`, which never stops at the first problem. Paths use JSON names with element ids (`sequences[<id>].tracks[<id>].items[<id>].kind.speed`; keyframes by index). Codes:

| Code | Rule |
|---|---|
| `DUPLICATE_ID` | ids unique per entity kind across the project (transitions included) |
| `UNKNOWN_REFERENCE` | asset and template references exist |
| `ASSET_KIND_MISMATCH` | media clips use video/audio assets, image items use image assets |
| `INVALID_FORMAT` | even dimensions in 2..=16384; sample duration a whole number of flicks |
| `NEGATIVE_TIME` | item start, source start and marker time ≥ 0 |
| `TIME_OUT_OF_RANGE` | a clip's source end stays within the flicks range |
| `EMPTY_ITEM` | items have a positive duration |
| `OVERLAPPING_ITEMS` | items of a track never overlap (touching is fine) |
| `ITEMS_NOT_SORTED` | items of a track are listed by start time (operations keep them so) |
| `ITEM_NOT_ALLOWED_ON_TRACK` | audio tracks hold only media clips; video tracks no clips of audio-only assets |
| `INVALID_SPEED` | speed is a number > 0 |
| `SOURCE_RANGE_EXCEEDS_MEDIA` | a clip's source range fits in its asset (when the duration is known) |
| `EMPTY_KEYFRAMES` | keyframed properties have at least one keyframe |
| `KEYFRAMES_NOT_SORTED` | keyframe times strictly increase |
| `KEYFRAME_OUTSIDE_ITEM` | keyframe times are in `[0, item duration]` |
| `VALUE_OUT_OF_RANGE` | opacity, reveal and each component of a reframe focus in `[0, 1]`, clip volume in `[-96, 24]` dB, font size > 0, blur, outline width and shadow blur ≥ 0, pan in `[-1, 1]`, fades ≥ 0 and fitting the item, Bézier `x1`/`x2` in `[0, 1]`; static styles: a non-empty font family, weight 1–1000, line height in `(0, 10]`, wrapping width, shape extents > 0, paddings and corner radii ≥ 0 |
| `LINK_GROUP_ACROSS_SEQUENCES` | items of a link group are in one sequence |
| `INVALID_TRANSITION` | a transition joins two adjacent items of its own track, matches the track kind, has a positive duration and a span inside both items, and shares no item edge nor overlaps another transition inside a shared item |
| `INSUFFICIENT_HANDLES` | the items of a transition have enough media beyond their edges for its span (`item_handles`) |
| `INVALID_CAPTION_WORDS` | caption words are non-empty without line breaks, in speaking order, apart (`end ≤ next start`) and inside their item; a caption's style exists (`UNKNOWN_REFERENCE`) and caption style values are in range (`VALUE_OUT_OF_RANGE`, `CaptionStyle::problems`) |

### 5.4 Operations, transactions, history, changesets

Modules `se_domain::{operation, transaction, history, changeset, error}` (ADR-0005, ADR-0016).

**Operations.** `Operation` is an enum of intent-level edits, serialized with its fields next to `"kind"` (the journal format, fixed by insta snapshots in `crates/se-domain/src/snapshots/`). One file per operation (or small group) in `operation/`:

| Group | Operations |
|---|---|
| Items | `AddItem { track, item }`, `RemoveItem { item }`, `RestoreItem { track, item }` (puts a recorded state back; the inverse of item edits), `MoveItem { item, track, start }` (any track, refuses overlaps), `TrimItem { item, edge: start\|end, to }` (adjusts `sourceStart`, stays inside the asset), `SplitItem { item, at, newItemId }`, `UpdateItem { item, patch: ItemPatch }` (one typed patch per item kind, `null` fields unchanged), `SetKeyframes { item, animation: PropertyAnimation }` (keyframes or static value of any animatable property), `ApplyAnimationPreset { item, preset, phase: in \| out \| emphasis, duration, start }` (a preset expanded into keyframes), `LinkItems { items, group }`, `UnlinkItems { items }` |
| Time | `RippleDeleteRange { sequence, range, tracks, splitIds }`, `RippleDeleteRanges { sequence, ranges, tracks, splitIds: {item: [ids]} }` (prompt 22, ADR-0032: several sorted, apart ranges in one pass, one snapshot per changed item in the inverse; equal to single ripple deletes from the last range; `remaining_parts` gives each item's parts), `InsertGap { sequence, at, duration, tracks, splitIds }` (`tracks: null` = every unlocked track; items crossing the edit point are split with the given ids; markers stay) |
| Tracks | `AddTrack { sequence, index, track }`, `RemoveTrack`, `UpdateTrack { track, patch }`, `ReorderTracks { sequence, order }` |
| Sequences | `AddSequence { index, sequence }`, `RemoveSequence`, `UpdateSequence { sequence, patch }` (name, format) |
| Markers | `AddMarker { sequence, index, marker }`, `RemoveMarker`, `UpdateMarker { marker, patch }` |
| Transitions | `AddTransition { track, index, transition }`, `UpdateTransition { transition, patch: TransitionPatch { duration, alignment } }`, `RemoveTransition { transition }` (they check ids, indices and locks; whether a transition fits is validated at the end of the transaction) |
| Assets | `AddAsset { index, asset }`, `UpdateAsset { asset, patch }`, `RemoveAsset` (refused while used) |
| Project | `RenameProject { name }`, `SetTranscriptionSettings { settings }` |
| Caption styles | `AddCaptionStyle { index, style }`, `UpdateCaptionStyle { style, patch: CaptionStylePatch }`, `RemoveCaptionStyle { style }` (refused while a caption uses it: `CAPTION_STYLE_IN_USE`; prompt 23, ADR-0033). Caption words follow trims, splits, ripple deletes and gaps by their middle (`reshape::retime_words`) |

Operations on one item take an optional `expectedRevision` (`REVISION_CONFLICT` when stale). Templates and the brand kit get operations with prompts 24 and 25.

**`apply(&Project, &Operation) -> Result<Applied { project, inverse, touched }, DomainError>`** is pure and deterministic: ids it needs are in the operation. The result always passes `validate_project` (otherwise `INVALID_DOCUMENT` and no change); `apply` does not touch revisions, so applying the `inverse` operations in order gives back exactly the original project. Item edits are inverted by `RestoreItem` snapshots; split, ripple delete and insert gap run as primitive steps whose inverses replay backwards. Rules: no overlap on a track, items sorted by start, no edit of items on a locked track and no removal of a locked track; a trimmed edge keeps its fade (shortened to fit), a cut edge gets none; keyframes stay at their instants (those left outside are dropped, a property left without keyframes keeps the value it held). `TouchedEntities` lists the items, tracks, sequences, markers and assets added, changed or removed, for the UI patches (§5.6).

**Transitions follow their items** (ADR-0024). The primitive item moves (`place_item`, `remove_item`) remove the transitions of the touched item that no longer fit and record an `AddTransition` after the item's own inverse, so inverses stay exact step by step. `SplitItem`, `RippleDeleteRange`, `InsertGap` and se-app's `EditPlan::finish` then put back the transitions they removed whose items touch again (`TransitionSnapshot::restorations`, shortened to the room left, following the right part of a split). Moving both items together or ripple-trimming at a cut keeps a transition; separating its items removes it.

**Transactions.** `Transaction { id, actor: Actor (user | aiTask { taskId } | system), label, ops }`. `system` transactions record facts the app observed (missing media at opening, decodability reports): journaled, never undone, never in a changeset. `commit(&Project, &Transaction) -> Result<Committed { project, inverse: Transaction, touched }, DomainError>` applies all operations or none (`apply_all`: each operation checks its own preconditions, `validate_project` runs **once, after the last one**, so a transaction may pass through states only its later operations repair; ADR-0021), then bumps the project revision by 1 and the revision of every touched item that still exists by 1.

**History.** `History` (default depth 200) keeps the user's transactions only (AI work is reverted per task). `record` stores each user transaction's inverse operations and empties the redo stack (transactions that changed nothing are skipped); `undo(&Project, TransactionId)` / `redo` commit the stored operations as a new user transaction (to journal), and keep the entry if it fails. `can_undo`, `can_redo`, `undo_label`, `redo_label` feed the menus.

**Changesets.** `Changeset { id, taskId, entries: [ChangesetEntry { transaction, inverse, touchedRevisions }] }` records every transaction of an AI task with the revision each touched item had right after it. `plan_revert(&Project, &Changeset) -> RevertPlan { ops, conflicts }` dry-runs the inverses, last transaction first, against the current document. `Conflict`: `itemChanged` (the item changed or disappeared since the task's last edit: its inverses are skipped), `foreignItems` (an inverse would also affect items the task never touched, e.g. user items on a track the task created), `cannotApply` (the inverse no longer applies). The plan's operations are committed as a user transaction (`into_transaction`), so a revert can be undone; without interleaved edits the project comes back as before the task (revisions aside). Changesets are owned by the project's actor and stored in `project.db` (§5.5, §11, ADR-0029).

**Errors.** `DomainError` (thiserror): one variant per refusal with ids, times as `HH:MM:SS.mmm` and the fix, e.g. `item 0193… would overlap item 0194… on track V1 between 00:00:51.000 and 00:00:52.500; move or trim one of them, or use another track`. `DomainError::code()` gives a stable `DomainErrorCode` (`ITEM_NOT_FOUND`, `OVERLAPPING_ITEMS`, `TRACK_LOCKED`, `REVISION_CONFLICT`, `INVALID_EDIT_POINT`, `TRIM_BEYOND_SOURCE`, `WRONG_ITEM_KIND`, `MISSING_SPLIT_ID`, `INVALID_DOCUMENT`, `NOTHING_TO_UNDO`…) for the protocol and MCP layers.

### 5.5 ProjectActor (single writer)

Module `se_app::project_actor` (ADR-0005, ADR-0017).

- One tokio task per open project, owning the `Project`, its undo `History`, the open **changesets** of its AI tasks (loaded at opening) and its `ProjectStore` (the locked folder). `ProjectActorHandle` sends messages: `Apply(Transaction)`, `ApplyAi { transaction, baseRevision? }` (an AI task's edit: committed only if the project is still at the revision it was planned on, recorded in the task's changeset → `AiCommit { revision, project, touchedItems, changesetId, changesetCreated }`), `RevertTask { taskId, label }` (`plan_revert` committed as one user transaction → `TaskRevertReport`), `Changeset { taskId }`, `Undo`, `Redo`, `Snapshot` (query → `ProjectView { project: Arc<Project>, history }`), `Close`. Dropping every handle closes the project too.
- An AI commit or a revert writes its journal entry and its changeset change (`ChangesetWrite::Record` / `Revert`) in one atomic store call (`append_journal_with_changeset`), then publishes `projectPatched`, `historyChanged` and `taskChangesetUpdated { projectId, taskId, changesetId, touchedItems }` (ADR-0029).
- On commit: `se_domain::commit` on a draft (validation included) → append the transaction to the journal (durable) → install the new state, record history → publish `projectPatched` then `historyChanged` → schedule the snapshot. A journal failure changes nothing.
- The snapshot is saved 2 s after the last commit (`DEFAULT_SAVE_DELAY`, a tokio timer) and on close; a failed save is retried later (the journal keeps every commit).
- `ProjectService` (`se_app::project_service`) holds the registry of open actors and the use cases behind the protocol: create (default sequence with tracks V1, V2, A1, A2), open (journal replay, then the recovered snapshot is saved), close, recent list, rename, duplicate, trash, create sequence, undo, redo. Operations on a closed project use a temporary actor whose events are discarded.

### 5.6 UI synchronisation

- On open, the renderer receives a `ProjectSession { path, snapshot: ProjectSnapshot, history }`. The snapshot is **normalised**: `ProjectHeader` plus lists of `SequenceEntity { trackIds, markerIds }`, `TrackEntity { sequenceId, itemIds }`, `ItemEntity` (an item + `trackId`), `MarkerEntity` (a marker + `sequenceId`) and assets (`se_app::project_view` builds them).
- Each commit emits `projectPatched { projectId, revision, actor, upserts, deletes }` (`actor`: who committed — `user`, `aiTask { taskId }` or `system`, prompt 20) with **whole entities** replaced by id; parents are refreshed with their children (a moved item also upserts its old and new tracks). Then `historyChanged { projectId, history }`. The renderer never re-implements operation logic.
- The renderer's `shared/project-store` applies a patch only if it is the next revision; on a gap, on `resyncRequired` and after a reconnection it reloads the snapshot (`getProjectSnapshot`); if the core restarted (`PROJECT_NOT_OPEN`) it reopens the project from its folder. Features read it through `useProjectSelector(selector)`.
- Drags, trims, gap resizes and drops use local ephemeral state in the timeline feature (`gesture-store`: items drawn elsewhere, ghosts, the snap point); one command is sent on release, and the ephemeral state clears when it resolves (se-core publishes the patch before answering) or fails (ADR-0021).

### 5.7 Smart cuts (prompt 22)

Module `se_app::smart_cuts` (ADR-0032): text-based editing, filler words and silence removal, for
the user (`SmartCutService`, one user transaction per action) and for agents
(`AgentEditService::apply_cuts`, one AI transaction per tool call).

- **Resolve** (pure, on the current document): `CutRequest { sequenceId, target: words { ranges } |
  fillers { range, lists, maxDuration } | silences { range, params, tracks }, allTracks, limit }`
  with `CutMedia` (transcripts and level envelopes of the sound clips' assets, read beforehand).
  Words go through `sequence_transcript` and get natural margins (`words.rs`: keep the longer pause,
  a 60 ms breath before the next kept word, 30 ms before a removed word; edges in sound snapped back
  to its onset within 150 ms from the level envelope); fillers are matched case-,
  accent- and punctuation-insensitively per transcript language (`fillers.rs`); silences are the
  windows where every listened sound track is quiet, measured over the whole sequence then clipped
  (`silences.rs`). `plan_cuts` turns candidates into frame-exact, apart ranges (padding, clips and
  limit, merge under 80 ms, slivers under 80 ms at clip edges removed, at least a frame).
- **Tracks** (`apply::cut_tracks`): the speech's tracks, then the unlocked tracks holding items
  linked to items crossing or following the cut; every unlocked track with `allTracks`.
- **Plan** (`apply::plan_cut`): one `RippleDeleteRanges` with ids for every new part, then
  `LinkItems` linking the parts of a link group between the same two cuts.
- Settings (`library.db`, `smartCuts.*`): filler lists per language (French and English defaults),
  the longest filler (1.5 s), silence defaults (−40 dBFS, 0.5 s, 0.12 s padding).
- Renderer (`features/transcript`): Suppr on selected words (shortcut context `transcript`),
  struck cut passages found between touching clips of one asset and restored by a ripple trim of the
  clip before the cut, the « Silences » view previewing cuts as temporary timeline bands
  (`showCutPreview` of the timeline feature), the « Hésitations » view (each occurrence ticked,
  heard through `shared/playback`'s `requestPlayRange`).

### 5.8 Captions (prompt 23)

ADR-0033. A caption is timed words plus a shared caption style; everything else is ordinary
editing.

- **Pure rules** (`se_domain::captions`): `caption_lines` (greedy line breaking by characters —
  shared vectors `docs/test-vectors/caption-lines.json` with the engine's `captionLines`),
  `segment_words` (spoken words → caption drafts: pause ≥ 0.6 s, full lines cut after a clause
  end when possible, maximum duration, sentence and clause ends, punctuation never first; 0.7–6 s,
  short gaps closed, frame-snapped edges), `retimed_words` (text edits keep unchanged words'
  timing, redistribute changed runs by length), `distribute_words`, `merged_words`,
  `split_point`, `tidy_words`. Predefined styles: `caption_style_presets()` /
  `caption_style_from_preset` over the engine's `packages/engine/src/caption-styles/*.json`.
- **Planning** (`se_app::captions`): the caption track (`track.rs`: highest unlocked video track
  holding captions only, « Sous-titres » created on top), the style to use and copy on write
  (`styles.rs`: in place when every user is targeted, a copy otherwise, unused styles removed;
  portrait sequences raise preset styles above the bottom third), generation and imports
  (`generate.rs`: captions of the caption track crossing the range replaced, the range grown over
  them), edits (`edits.rs`: text, emphasis, merge of adjacent captions of one track, split between
  two words, shift), SRT cues (`subtitles.rs`). `CaptionService` runs the user's commands (one
  transaction each, through `ProjectService::plan_user_edit`); the agent planner reuses the same
  planning for `AgentEdit::{AddCaptions, EditCaptionText, SetCaptionEmphasis, UpdateCaptionStyle,
  ApplyCaptionPreset}`; the scope check refuses a style change when a caption using it lies outside
  the task's scope.
- **Smart cuts** cut caption tracks with the speech (captions follow).
- **SRT**: se-export's `write_srt`/`parse_srt` and `FsSubtitleFiles` (se-app port `SubtitleFiles`).
- **Renderer**: `features/captions` (the « Sous-titres » tab), `shared/caption-preview` (looks
  previewed in the viewer: hovered presets, dragged captions), the viewer's caption drag.

## 6. Protocol (`se-protocol` → `@se/protocol`)

Conventions and their rationale: ADR-0010 and ADR-0012.

- Transport: one WebSocket at `/rpc`. Messages are JSON, tagged by `type`:
  - `{"type":"request","id":…,"command":{"kind":…}}` → `{"type":"response","id":…,"result":{"kind":…}}` or `{"type":"response","id":…,"error":{code,message,details?}}` (`id` is `null` when the request could not be read).
  - `{"type":"event","event":{"kind":…}}` for server push. Every connection first receives `coreReady`; `coreShuttingDown` precedes the close on shutdown; `resyncRequired` tells a connection that fell behind that it missed events (later: project patches, job progress, task events).
- `Command`, `CommandResult`, `CoreEvent` are Rust enums (`#[serde(tag = "kind", rename_all = "camelCase")]`); every `Command` variant has a `CommandResult` variant of the same name, so `ResultFor<K>` is derived in TypeScript without a table.
- `pnpm codegen` runs the `se-codegen` binary (ts-rs) into `packages/protocol/src/generated/`; `pnpm codegen --check` (part of `pnpm check`) fails when the committed files are stale; `tsc -b` checks they compile.
- se-core side (`crates/se-core/src/rpc/`): `RpcRouter` parses a frame (`INVALID_REQUEST` / `UNKNOWN_COMMAND` with actionable messages), dispatches to one handler per command group (`rpc/handlers/`), encodes the response. `EventBus` (bounded `broadcast`) fans events out; a slow connection never blocks the others.
- Client side: `CoreClient` (`@se/protocol`) correlates ids, times requests out (30 s default), reconnects with a capped exponential backoff (re-resolving the endpoint each time), queues requests while disconnected and fails in-flight ones with `CONNECTION_LOST`. The renderer creates one client in `src/shared/core/` and exposes it through React context and hooks.
- Project commands (prompt 07): `getNewProjectDefaults`, `createProject`, `openProject { path }`, `closeProject { projectId }`, `listRecentProjects`, `renameProject { path, name }`, `duplicateProject { path, name }`, `trashProject { path }`, `getProjectSnapshot { projectId }`, `createSequence { projectId, name, format? }`, `undo`, `redo`; sequences (prompt 16, ADR-0026): `renameSequence`, `setSequenceFormat { format, adaptation }`, `duplicateSequence { name, format?, adaptation }` (a copy, or a version in another format) → `{ sequenceId }`, `removeSequence` (`LAST_SEQUENCE` for the only one). Events: `projectOpened` (session), `projectPatched`, `projectClosed`, `historyChanged`. Paths are absolute strings.
- se-core runs every request in its own task: a slow command never delays events or other requests, and requests of one client may complete in any order (ADR-0017).
- Timeline commands (prompt 10), one undoable user transaction each: `addTrack { projectId, sequenceId, trackKind, index }` → `trackId` (named after the next free `V<n>`/`A<n>`), `removeTrack`, `updateTrack { trackId, patch: TrackPatch }`, `reorderTracks { sequenceId, order }`, `addMarker { sequenceId, time, label, color }` → `markerId`, `updateMarker { markerId, patch: MarkerPatch }`, `removeMarker`. Labels say what changed (`Mute track`, `Rename marker`…, translated by the renderer). Patch fields are optional in TypeScript.
- Item commands (prompt 11, ADR-0021), one undoable user transaction each, planned by `se_app::edit_plan::EditPlan` (operations applied to a draft, committed together): `placeMedia(PlaceMediaRequest { sequenceId, assetIds, start, videoTrack, audioTrack: TrackTarget (existing | new), mode })` → `itemIds` (clips one after the other, a video with sound as two linked items, images `DEFAULT_IMAGE_DURATION`), `moveItems { moves: [ItemPlacement { itemId, trackId, start }], mode }`, `duplicateItems { copies, mode }` → `itemIds`, `pasteItems { items: [PastedItem { trackId, item }], mode }` → `itemIds`, `trimItems { trims: [ItemTrim { itemId, edge, to }], ripple }`, `splitItems { itemIds, at }` → right parts (linked parts stay linked), `deleteItems { itemIds, ripple }`, `linkItems` → `linkGroup`, `unlinkItems`, `resizeGap { trackId, start, end, duration }` (0 closes). `PlacementMode`: `overwrite` (placed items replace what they cover on their tracks) or `insert` (time opens at the block's start). Ripple edits shift the **concerned tracks**: the edited tracks plus every unlocked track holding items linked to what moves; removing time over other content of a concerned track fails with `RIPPLE_BLOCKED`. Labels `Add clips`, `Move items`, `Ripple trim`, `Close gap`… (`se_app::item_labels`, translated by the renderer).
- Sound and transition commands (prompt 14, ADR-0024), one undoable user transaction each: `updateItems { updates: [ItemUpdate { itemId, patch: ItemPatch }], animations? }` (fades and shapes, speed, reframe modes…; optional `ItemAnimation`s set in the same step; planned like item edits; label `Change fades` when only fades change, `Reframe items` when only reframe modes change, else `Edit items`), `setItemAnimation { itemId, animation: PropertyAnimation }` (keyframes or static value of one property; `Change volume` / `Change keyframes`), `addTransitions { transitions: [NewTransition { trackId, leftItemId, rightItemId, duration, alignment }] }` → `transitionIds` (kind from the track, durations shortened to each cut's room, cuts without room skipped, `TRANSITION_NO_ROOM` when none fits), `updateTransition { transitionId, patch }` (duration capped by the cut), `removeTransitions { transitionIds }`; `getAudioSilences { assetId }` → `silences` (source time, `null` while not analysed). `TrackEntity` carries its `transitions`.
- Graphics commands (prompt 15, ADR-0025), one undoable user transaction each: `addGraphicItem(AddGraphicRequest { sequenceId, at, duration, graphic: text { content } | shape { shape } | image { assetId } })` → `PlacedGraphic { itemId, trackId }` (highest unlocked, visible video track free over the span with nothing above it, or a new video track on top; labels `Add text`/`Add shape`/`Add image`), `setItemAnimations { animations: [ItemAnimation { itemId, animation }] }` (several properties of several items; `Transform items` for placement only, `Change keyframes` when keyframes are written, else `Edit items`), `applyAnimationPreset { itemIds, preset, phase, duration, start }` (`Apply animation`); `listAnimationPresets` → `AnimationPresetInfo { id, name, description, phases, defaultDuration, properties, samples }`; `listFonts` → `FontFamily { name, source: bundled | system, faces: [FontFace { id, weightMin, weightMax, italic, unicodeRange, format }] }` (bundled first). Text, shape and image patches (`updateItems`) carry the static style fields.
- Media commands (prompt 09): `reportDecoderCapabilities { hevcMain10, h264High, aac, flac }`, `generateDerivedMedia { projectId, assetId, derived: mezzanine | proxy }`, `getMediaSettings` / `updateMediaSettings(MediaSettings { mezzanineQuality, proxyResolution: 540p | 720p, autoProxies, toolsDir })`, `getMediaToolsStatus` → `MediaToolsStatus { ffmpegPath, ffprobePath, ffmpegVersion, hardwareEncoders, problem }`, `getProjectCache { projectId }` / `clearProjectCache { projectId }` → `ProjectCacheInfo { path, sizeBytes }`. Job kinds gain `mezzanine`, `proxy`, `audioAnalysis`, `sceneDetection`; codes `DERIVED_NOT_APPLICABLE`, `MEDIA_TIME_OUT_OF_RANGE`.
- Media commands (prompt 08): `importMedia { projectId, paths }` → `ImportReport { imported, duplicates, failures }` (one user transaction `Import media`; per-file failures carry a code), `relinkAsset { projectId, assetId, path, force }` → `outcome: relinked | contentDiffers`, `removeAsset`, `listJobs { projectId }`, `cancelJob { jobId }`, `reportDecodability { projectId, assetId, video, audio }`. Events: `jobQueued { job }`, `jobProgress { jobId, projectId, assetId, progress }`, `jobCompleted { job }`, `jobFailed { job }`, `jobCancelled { job }` (`JobSnapshot { id, projectId, assetId, kind, state, attempt }`).
- Smart cuts (prompt 22, ADR-0032), one undoable user transaction each: `removeWords { projectId, sequenceId, ranges, purpose: words | fillers, allTracks }` → `CutSummary { cuts, removedDuration, wordCount, trackIds, untranscribedAssets, unanalysedAssets }` (labels `Remove words` / `Remove fillers`), `findFillers { projectId, sequenceId, range? }` → `{ occurrences: [FillerOccurrence { text, start, end, itemId, assetId }], untranscribedAssets }`, `previewSilenceCuts { projectId, sequenceId, range?, params: SilenceCutParams { thresholdDbfs, minSilence, padding }, trackIds?, allTracks }` → `CutSummary` (nothing changes), `removeSilences { … }` → `CutSummary` (`Remove silences`), `getSmartCutSettings` / `updateSmartCutSettings(SmartCutSettings { fillers: [FillerList { language, words }], fillerMaxDuration, silence })`. `TranscriptionPreferences.keepHesitations` (whisper's prompt opens with hesitations).
- Captions (prompt 23, ADR-0033), one undoable user transaction each: `generateCaptions(GenerateCaptionsRequest { projectId, sequenceId, range?, style?: CaptionStyleChoice (preset { preset } | existing { styleId }), segmentation?, trackName? })` → `CaptionsReport { itemIds, trackId, styleId, replaced, untranscribedAssets }` (« Generate captions »), `importSubtitles(ImportSubtitlesRequest { path, style?, trackName? })` → `CaptionsReport` (« Import subtitles »), `exportSubtitles { sequenceId, path }` → `{ cueCount }`, `editCaptionText(CaptionTextEdit)`, `mergeCaptions { itemIds }` → `{ itemId }`, `splitCaption { itemId, wordIndex }` → `{ itemId }`, `shiftCaptions { itemIds, delta }`, `setCaptionEmphasis(CaptionEmphasisEdit)`, `updateCaptionStyle(CaptionStyleEdit { itemIds, patch })` → `{ styleIds }` (copy on write), `applyCaptionPreset { itemIds, preset }` → `{ styleId }`. `ProjectHeader.captionStyles` (patched with the header when styles change). Codes `SUBTITLE_FILE_FAILED`, `INVALID_SUBTITLE_FILE`.
- Frames for AI agents (prompt 18, ADR-0028): event `renderFramesRequested(FrameRenderRequest { requestId, projectId, sequenceId, times, width, height })`; the renderer uploads one JPEG per time with `PUT /media/renders/{requestId}/{index}` (session token, CORS for that route) or answers `reportFrameRenderFailure { requestId, message }`. Codes `RENDERER_UNAVAILABLE`, `RENDER_FAILED`, `RENDER_REQUEST_NOT_FOUND`.
- AI commands (prompt 17, ADR-0027): `submitTask(SubmitTaskRequest { projectId, prompt, scope: TaskScope { sequenceId, range?, tracks? }, provider?, model?, effort? })` → `{ task }` (queued), `cancelTask { projectId, taskId }`, `retryTask` → `{ task }`, `followUpTask { prompt }` → `{ task }`, `listTasks { projectId }` → `{ tasks }` (oldest first, without events), `getTaskEvents { projectId, taskId }` → `{ events: [TaskEventRecord { sequence, run, at, event: AgentEvent }] }`, `getAiStatus` / `refreshAiStatus` → `{ status: AiStatus { provider: ProviderInfo, installation, auth, capabilities } }`, `getAiSettings` / `updateAiSettings(AiSettings { provider, executablePath, defaultModel, defaultEffort, maxConcurrentTasks })`. Events: `taskUpdated { task }` (every change of a task), `taskEventAppended { projectId, taskId, event }`, `aiStatusChanged(AiStatus)`. Codes `TASK_NOT_FOUND`, `TASK_NOT_ACTIVE`, `TASK_STILL_ACTIVE`, `TASK_NOT_RESUMABLE`, `AI_UNAVAILABLE`.
- AI edits (prompt 19, ADR-0029): `Task.changesetId` (the open changeset of its edits, `null` while it changed nothing or once reverted); `revertTask { projectId, taskId }` → `{ report: TaskRevertReport { taskId, reverted, revertedItems, conflicts: [Conflict (itemChanged | foreignItems | cannotApply)] } }` (refused while the task is active: `TASK_STILL_ACTIVE`; `NOTHING_TO_REVERT` without an open changeset); event `taskChangesetUpdated { projectId, taskId, changesetId, touchedItems }`; code `OUT_OF_SCOPE` (agent edits outside the task's scope).
- AI task UX (prompt 20, ADR-0030): `followUpTask { prompt, scope? }` (a new scope on the task's sequence, kept by the task), `previewTaskRevert { projectId, taskId }` → `{ report }` (what `revertTask` would do, nothing changes), `getTaskChanges { projectId, taskId }` → `{ changesetId, touchedItems }`; `Task.usage: TaskUsage { inputTokens, outputTokens, costUsd }` (the runs' `usage` events added up); `ProjectPatch.actor`. Agent events name MCP tools by their own name (`add_text`).
- Errors carry a stable `code` (`ErrorCode`) and a human/agent-readable message: `INVALID_REQUEST`, `UNKNOWN_COMMAND`, `INTERNAL`, project codes `PROJECT_NOT_FOUND`, `PROJECT_LOCKED`, `SCHEMA_TOO_NEW`, `PROJECT_CORRUPTED`, `PROJECT_ALREADY_EXISTS`, `PROJECT_NOT_OPEN`, `STORAGE_FAILED`, edit codes `EDIT_REJECTED` (`details.domainCode` holds the `DomainErrorCode`), `NOTHING_TO_UNDO`, `NOTHING_TO_REDO`, media codes `ASSET_NOT_FOUND`, `MEDIA_NOT_FOUND`, `MEDIA_UNSUPPORTED`, `MEDIA_KIND_MISMATCH`, `MEDIA_TOOLS_MISSING`, `JOB_NOT_FOUND`, `RIPPLE_BLOCKED` (prompt 11), AI codes (prompts 17–19) `TASK_*`, `AI_UNAVAILABLE`, `RENDER*`, `OUT_OF_SCOPE`, `NOTHING_TO_REVERT`. MCP tools map domain refusals to their own codes (`REVISION_CONFLICT`, `LOCKED_TRACK`, `VALIDATION_ERROR`; `docs/ai-tools.md`). The renderer gives the most common `EDIT_REJECTED` domain codes their own message (`errors.edit.<DOMAIN_CODE>`). Codes raised by clients (`TIMEOUT`, `CONNECTION_LOST`, `CLIENT_CLOSED`) are part of the same enum. The renderer translates every code (`errors.<CODE>`).

## 7. Media pipeline (`se-media`)

Decisions: ADR-0007 (sidecars, mezzanines), ADR-0018 (import, jobs, cache) and ADR-0019 (mezzanines, proxies, analyses, settings). File formats: `crates/se-media/CLAUDE.md`.

- **Binaries**: `ffmpeg`, `ffprobe` looked up on every use (`ToolLocator`): the folder set by the user (library setting `media.toolsDir`) → the folder of se-core (packaged: `resources/bin/`, bundled by electron-builder) → `resources/bin/<platform>/` in an ancestor (development, filled by `pnpm fetch-binaries`: FFmpeg 9.0.2 pinned by SHA-256) → `PATH`. Missing → `MEDIA_TOOLS_MISSING`.
- **Processes**: one helper (`se_media::process::Sidecar`): no stdin, no console window, job object (Windows) or process group (Unix); dropping it before exit kills the tree (cancellation = dropping the job's future).
- **Import** (`MediaService::import_files`): each path is checked, fingerprinted (size + mtime + xxh3 of the first and last MiB) and compared with the project's assets (same content → `duplicates`), then probed (`ffprobe -show_format -show_streams`, 4 at a time) → `Asset` (absolute path, relative hint to the project folder, kind, `MediaInfo`, fingerprint, decodability `unknown`/`noStream` per stream). New assets form one user transaction; their jobs are queued.
- **Decodability**: the renderer builds WebCodecs codec strings from `MediaInfo` (`@se/engine/capabilities`: avc1, hvc1/hev1, vp09, av01, vp8; mp4a, opus, flac, PCM…), asks `VideoDecoder`/`AudioDecoder.isConfigSupported` (`ImageDecoder.isTypeSupported` for stills) and sends `reportDecodability` (a system transaction). Before that, once per session, it reports its **decoder capabilities** for the codecs of derived copies (`checkDecoderCapabilities`: 4K HEVC Main 10 and H.264 High, AAC, FLAC → `reportDecoderCapabilities`, kept in `library.db`).
- **Mezzanine policy** (`se_app::media_policy`, pure, decision table in its tests): `Waiting` while a stream is `unknown`; needed when the picture or the first sound stream is `undecodable`, or the picture is not 4:2:0 at 8–10 bits. Picture: HEVC Main 10 when the source has more than 8 bits and the machine decodes HEVC Main 10, else H.264 High 8 bits, CRF from the settings (14), a keyframe about every second; copied when only the sound needs work. Sound: copied when decodable and MP4-compatible (AAC, MP3, Opus, FLAC), else AAC 320 kb/s (FLAC without AAC decoder). Sound files → M4A; undecodable stills → PNG.
- **Proxies**: H.264 High 8 bits, shorter side 540/720 (never enlarged), CRF 23, fast decode, no B-frames, a keyframe every 15 frames, sound copied or AAC 160 kb/s. Automatic (setting, on by default) for videos whose shorter side exceeds 1080 or whose picture needs a mezzanine; on request through `generateDerivedMedia` (« Générer un proxy »).
- **Derived states**: `Asset.derived.{mezzanine, proxy}` = `DerivedFile { relativePath, status: pending | ready | failed { message } }`, the file in the content's cache folder (`mezzanine.mp4|m4a|png`, `proxy-540|720.mp4`). Queued → `pending` (system transaction `Queue derived media`); a job records `ready` or, on its last attempt, `failed` (`Record derived media`) only while the asset still waits for that file. Relinking to other content clears derived copies and analysis summaries. At opening and after a cache cleanup: pending copies are queued again, ready copies whose file is gone regenerated, failed ones retried.
- **Hardware encoding** (`se_media::encoders`): NVENC (Windows), VideoToolbox (macOS), NVENC then VAAPI (Linux), each validated once per ffmpeg by a 5-frame test encode (detection starts in the background at startup); CRF → `-cq` / `-q:v` / `-qp`; a failing hardware run is redone with libx264/libx265. Derived copies keep the coded size and the display rotation (`-noautorotate`).
- **Analyses**: `audioAnalysis` (one PCM pass on the blocking pool: EBU R128 integrated loudness, range, true and sample peaks, short-term loudness every 0.5 s, silences = 20 ms RMS windows under −40 dBFS for ≥ 0.5 s, and since prompt 22 the **level envelope** `LevelEnvelope` — every 20 ms window's RMS level in whole dB below full scale — from which smart cuts find silences for any threshold; analyses without it are `is_outdated` and run again when their project opens) and `sceneDetection` (scene score > 0.3 on a 320 px copy, `showinfo` parsed from stderr). Results in `project.db` (`analyses`, keyed by asset and kind, with the content key analysed); the asset keeps a summary (`Asset.analysis`, system transaction `Record media analysis`). An analysis runs once per content.
- **Transcription** (prompt 21, ADR-0031; `se_app::TranscriptionService`, adapter se-transcribe): a job extracts the first sound stream as 16 kHz mono PCM WAV (`SpeechAudioExtractor`, se-media: `-map 0:a:0 -ac 1 -ar 16000 -c:a pcm_s16le` into `cache/media/<key>/transcription/`), runs whisper-cli (full JSON, DTW alignment for catalogue models, voice detection when the voice model is installed, the project's language and vocabulary, GPU unless disabled; a failed GPU run is redone on the CPU), stores the transcript in `project.db` and records `Asset.analysis.transcript` (system transaction « Record transcript ») while the asset holds that content; the run's folder is removed. Words: whisper's tokens joined into words (French spacing and apostrophes), timed by DTW, mapped back from voice detection's processed time and bounded by the speech spans (`se_transcribe::{words, voice_spans}`). Models in `<app data>/models/whisper/` (catalogue: tiny, base, small, medium, large-v3-turbo + the Silero voice model; imports), downloaded only by explicit commands (`startTranscriptionDownload`, `transcriptionStatusChanged` progress at most once per percent); the GPU pack in `<app data>/runtimes/`. Preferences in `library.db` (`transcription.model`, `useGpu`, `autoTranscribe`, `voiceDetection`, `whisperDir`). The media service tells the transcription service about imports (`ImportListener`).
- **Frame extraction** (service): `MediaService::extract_frame(project, asset, time, maxWidth)` → cached upright JPEG `cache/media/<key>/frames/<flicks>-<width>.jpg` (16–3840 px wide, never enlarged; the last frame at the very end). For MCP tools (prompt 18).
- **Playback source** (`@se/engine/sources`, `playbackSource(asset)`): a ready mezzanine whenever one exists; else the original once no stream was refused and no mezzanine is pending; else « en préparation » (or unavailable: missing original, failed mezzanine). The media bin shows it as a badge.
- **Opening a project**: originals found at their path clear `missing`; those found through their relative hint with the same content are relinked silently; others get `missing` (system transaction). Then missing derived files are queued. `relinkAsset` checks the fingerprint (other content → `contentDiffers`, confirm with `force`) and the kind.
- **Jobs** (`se_app::JobQueue`): generic `JobWork` futures (told their attempt number and whether it is the last) with a kind (slots per kind: thumbnails 2, peaks 2, mezzanine 1, proxy 1, audio analysis 2, scene detection 1) and **at most 4 running in all**, so priorities order kinds: `High` thumbnails and peaks > `Normal` proxies and mezzanines of used media > `Low` analyses > `Idle` mezzanines of unused media (FIFO within a priority; running jobs are never pre-empted). One automatic retry, cancellation, events `jobQueued/Progress/Completed/Failed/Cancelled`; a project's jobs are cancelled and forgotten when it closes. `Transcription` (slot 1, prompt 21: `Normal` on request, `Low` after an import). Later kinds: `SubjectDetection`, `OverlayRender`.
- **Thumbnails**: up to 40 tiles 90 px high (keyframes only beyond 30 s), 8 per row in `thumbnails.jpg`, their source times in `thumbnails.json` (`ThumbnailSheet`), a poster up to 360 px high at 10 % of the duration. **Peaks**: PCM `f32` from ffmpeg's stdout, min/max per channel (≤ 2) per 256/1024/4096 samples, `peaks-<level>.sepk`.
- **Cache**: `<project>/cache/media/<cache key>/` (key = `<hash>-<size>-<mtime>`), files written under temporary names then renamed; thumbnails, peaks and frames are not recorded in the document (regenerated when missing), mezzanines and proxies are (`DerivedMedia`). The settings screen shows the project cache's location and size and cleans it (`clearProjectCache`).
- **Settings** (`library.db`, `MediaService::media_settings` / `update_media_settings`): `media.mezzanineQuality` (CRF 8–28, default 14), `media.proxyResolution`, `media.autoProxies`, `media.toolsDir` (applied at once to the shared `ToolLocator`), `media.decoders.*` (capabilities).
- Decode source selection (prompts 12/13): `playbackSource` (original or mezzanine); preview may switch to the proxy.
- **`/media/fonts/{id}`** (se-core, ADR-0025): a face of the font catalogue (bundled faces from memory, installed ones through `ServeFile`), 404 for unknown ids.
- **`/media` routes** (se-core, token as `?t=`): `/media/assets/{id}/original`, `/thumbnails` (JSON), `/thumbnails/sheet`, `/thumbnails/poster`, `/peaks/{256|1024|4096}`, `/mezzanine`, `/proxy` (ready ones only); Range (206/416), `HEAD`, conditional requests, `Cache-Control: no-cache`; 404 for unknown assets and outputs not generated yet; CORS for reads from the renderer's origin (`file://` packaged, the Vite dev server in development: the engine reads media with `fetch`). Later: `/media/renders/{job}` (upload of rendered chunks).

## 8. Rendering engine (`@se/engine`)

ADR-0002 (web engine shared by preview and export), ADR-0022 (how prompt 12 built it), ADR-0023 (playback and audio, prompt 13).

```
sequence + assets + time ──► evaluate ──► FrameGraph ──► FrameRenderer ──► Compositor.render(graph, pictures) ──► canvas
                                                              │  ▲
                                         chooseVariant + URL  ▼  │ leases
                                                    DecoderPool (Mediabunny + WebCodecs, LRU picture cache)
```

- **Entry points.** `@se/engine` (root) is pure and dependency-free: `animation/` (easings, Bézier solver, number/vector/OKLab colour interpolation) and `evaluate/`; `@se/engine/playback` (transport, JKL, frame pacing, `PlaybackController`) is pure too. GPU, decoding and sound parts are subpaths that load PixiJS and Mediabunny: `@se/engine/preview` (the viewer engine, and `StillRenderer`: offscreen JPEG stills through the same `FrameRenderer` and compositor, for AI agents' `get_frames` — ADR-0028) and `@se/engine/audio` (the playback sound), both imported lazily by the renderer, `@se/engine/composite`, `@se/engine/decode`; plus `capabilities` and `sources` (prompts 08–09).
- **`evaluate(sequence, assets, time) → FrameGraph`** is pure: on every visible video track (bottom first) the item covering `time` (binary search, half-open ranges) becomes a layer — `video` (asset, source time `sourceStart + (time − start) × speed` rounded to the flick, natural box), `image`, `shape`, `text`, `caption` (prompt 23: the page of lines holding the active word, each word with its colour, pop scale and reveal opacity, styled by the sequence's `captionStyles`, the default preset without them); templates draw nothing until prompt 24. Transforms are resolved at the item-relative time in domain units (ADR-0015); clip fades (`fadeCurve` along their shape) multiply the opacity; invisible layers (opacity 0, zero scale) are dropped so their media is never decoded. During a **cross dissolve** (ADR-0024) the track gives both items — the outgoing one past its end, the incoming one before its start, weighted `1 − p` and `p` — tagged with their `DissolveRole`; prefetch includes what they play beyond their edges. A picture's **natural box** comes from the asset's upright display size (rotation applied) and the clip's reframe mode (`reframedPicture`, ADR-0026): fitted into the frame (`fit`), the frame itself with a **crop** of the picture around the focus (`fill`, `tracked`; the compositor frames its texture on the crop, mapped back through rotation and mirroring by `codedCrop`), or the natural size in design pixels (`custom`; images fit, fill or keep their natural size): it never depends on the decoded file, so a proxy keeps the framing. Platform **safe areas** are engine data (`@se/engine/safe-areas`: one JSON file per platform with source and date) that the viewer draws as an overlay, never rendered into a frame. Shared interpolation vectors: `docs/test-vectors/interpolation.json`.
- **Decoding** (`DecoderPool`): one Mediabunny source per (asset, variant) over se-core's `/media` URLs (`UrlSource`, Range requests); `chooseVariant` picks the ready proxy when the viewer's option is on (even for an offline original), otherwise `playbackSource` (original if decodable, else mezzanine); stills load as `ImageBitmap`. Media time 0 is the first frame. The renderer asks for the frame at `frameLookupTime`: with the asset's frame rate, the frame index is computed exactly in flicks and the lookup aims at the middle of that frame (robust to millisecond-rounded container timestamps). Pictures live in an LRU cache keyed by source and URL (byte budget, count limit: hardware decoders stall when too many frames are held); callers borrow them through **leases**, and the cache closes a picture only when it is evicted and unleased. Decodes of one source run one at a time; a request aborted before it starts is skipped (fast scrub). Every picture goes through a `FrameTracker` (open `VideoFrame`/`ImageBitmap` counter, the dev leak check).
- **Compositing** (`Compositor { resize, setBackground, render, destroy }`, `PixiCompositor` on PixiJS v8: WebGL 2, WebGPU optional, `HTMLCanvasElement` or `OffscreenCanvas`; `pixi.js/unsafe-eval` keeps it inside the CSP). It draws in sequence pixels scaled to the canvas, reuses one scene node per item, uploads `VideoFrame`s as textures (display rotation and mirroring applied) and keeps no picture beyond `render`, so leases are released right after. A dissolve pair is drawn into an isolated group (alpha filter at 1), the incoming side added (premultiplied `add`), then the group over what lies below: `(1 − p)·(A over below) + p·(B over below)`, never a dip.
- **`FrameRenderer`** (the path export will reuse): evaluate, lease cached pictures, draw synchronously when all are cached; otherwise report `loading`, keep the previous image, decode, then draw unless a newer request superseded it. Media that cannot be drawn is reported per asset (`preparing`, `missing`, `mezzanineFailed`, `decodeFailed`). **`PreviewEngine`** wraps it for the viewer: scene, time, proxy, background and size setters coalesced into one draw per animation frame, a state (`idle`/`loading`/`ready`/`error` + issues), `stats()` and `settled()`.
- **Playback** (prompt 13, ADR-0023): `PlaybackController` (`@se/engine/playback`, pure) derives the sequence time from the **audio clock** (`audioContextClock`: the context time of the sample being heard) and a transport anchor — no accumulation, so no drift — and reports on each display refresh the frame that will be on screen at the next one (late frames are skipped and counted). It handles play/pause, stop (back to the start of the run), JKL (1×, 2×, 4× each way; K held + J/L steps), loops over the in/out span and frame steps; a playhead moved elsewhere is a seek. In playback mode the `PreviewEngine` draws in the caller's refresh, never supersedes a frame still decoding, and keeps the decoder pool's **sequential readers** (`FrameReader`, one Mediabunny decoder per clip running through the file; reverse read in chunks) ahead of the playhead with an adaptive window, a read-ahead budget in frames and bytes, and passed frames closed at once (held frames stall hardware decoders).
- **Audio** (`@se/engine/audio`, ADR-0023, ADR-0024): `AudioEngine` = graph (voice → clip gain envelope → track gain with mute/solo/volume → track pan → master → speakers, 10 ms smoothing; each voice's gain follows `gainEnvelope`: volume curve in dB × fades × constant-power cross-fade weight, one value when constant, else points every 10 ms ramped with `linearRampToValueAtTime`; clips joined by a sound cross-fade play from `playStart` to `playEnd`) + `AudioChunkPool` (one-second chunks decoded with Mediabunny, contiguous across buffers and chunks even when the container rounds timestamps; original or mezzanine, never the proxy) + `AudioScheduler` (every 25 ms, the pieces starting within the next 200 ms, cut at chunk boundaries, clip edges and loop points, start times from the anchor; chunks decoded 2 s ahead) + audio scrub (80 ms snippets on paused moves) + the master `AudioMeter` (AudioWorklet peak/RMS, bundled with `?worker&url`). Sound plays at real-time forward speed only.
```
display refresh ─► PlaybackController ──(frame start)──► playhead store ─► PreviewEngine (playback mode) ─► readers / cache ─► compositor
        ▲                │ transport (anchor, rate, loop span)
   audio clock ◄──── AudioEngine: AudioScheduler (25 ms ticks, 200 ms ahead) ─► chunks ─► voices ─► gains ─► master ─► meter worklet
```
- Planned: `ExportWorker` (prompt 27: same evaluate + compositor on `OffscreenCanvas`, `VideoEncoder`/`AudioEncoder`, Mediabunny muxer; ffmpeg through se-core for ProRes/DNxHR), `MotionTemplateRuntime` (§10).
- **Graphics** (prompt 15, ADR-0025): text layers carry their whole style in sequence pixels (font request, colour, letter spacing, line height, alignment, wrapping width, outline, shadow, plate, reveal, letter case applied); shape layers an outline and a `line` geometry (capsule), covering shapes the frame's box; images their fit; every layer a blur radius (a Pixi blur filter, only while visible). The Pixi text node measures the whole text once without outline or shadow and draws it line by line at aligned places, so a reveal, an outline or a shadow never moves it. `Compositor.render` returns each layer's box; `FrameRenderer` returns a `FrameLayout` and the fonts missing from the catalogue, and `PreviewEngine` publishes the layout after every drawn frame (`getLayout`, `subscribeLayout`) for the viewer's transform handles.
- **Captions** (prompt 23, ADR-0033): `captionLayerStyle` (pure: active word = last started, `captionPage`, colours for `wordHighlight`/`highlightSpoken`/emphasis, `popPerWord` scale bump over 0.18 s, `revealPerWord` fade over 0.08 s, plates per line for `boxPerLine`) and `CaptionNode` (one Pixi text per word placed from `captionLayout`, a layout measured on the plain words so animation never moves a word; one shared `TextStyle` per colour; plates). Predefined styles: `@se/engine/caption-styles`. Golden frames: karaoke, TikTok pop, box per line, reveal.
- **Fonts** (`@se/engine/fonts`): `FontRegistry` registers the catalogue's faces (`listFonts` + `/media/fonts/{id}` URLs) as `FontFace`s and loads a family's faces the first time a frame's text uses it; `FrameRenderer` waits for them (`FONT_LOAD_TIMEOUT_MS`, 4 s) before drawing, so preview and export draw the same glyphs; families the catalogue lacks fall back to Inter and are reported (`PreviewState.missingFonts`).
- **Golden frames** (`packages/engine/test/golden/`, Vitest browser mode in headless Chromium via Playwright, project `engine-browser`): committed fixtures from `scripts/generate-golden-fixtures.mjs` (videos encoding their frame number in binary, a proxy, a rotated copy, VP9, colour bars, an alpha disc) and reference PNGs compared with a tolerance; WebGL runs on SwiftShader and decoding prefers software so references match on every OS. `SE_UPDATE_GOLDEN=1` rewrites them.

## 9. AI architecture

### 9.1 Ports

Declared in `se_app::ports::agents` (ADR-0003, ADR-0027); DTOs in `se_protocol` (`ai.rs`, `tasks.rs`).

```rust
#[async_trait]
pub trait AgentProvider: Send + Sync {
    fn info(&self) -> ProviderInfo;                 // id, name, command, login command, help URL
    fn capabilities(&self) -> ProviderCapabilities; // resume, image input, models, effort levels
    fn set_executable(&self, path: Option<PathBuf>); // the path chosen in the settings
    async fn detect(&self) -> Result<ProviderInstallation, AgentError>; // path, version
    async fn auth_status(&self) -> Result<AuthStatus, AgentError>;      // LoggedIn | LoggedOut | Unknown
    async fn start(&self, spec: AgentRunSpec) -> Result<AgentRun, AgentError>;
}

pub struct AgentRunSpec {
    pub prompt: String,
    pub system_instructions: String,
    pub mcp: McpEndpoint,              // url + the run's bearer token (ADR-0028)
    pub model: Option<String>,
    pub effort: Option<EffortLevel>,
    pub session: AgentSession,         // Start(new uuid) | Resume(uuid)
    pub working_dir: PathBuf,          // empty, task-specific directory
}

pub struct AgentRun {
    pub events: mpsc::Receiver<AgentEvent>, // ends with one Completed | Failed, then closes
    pub cancel: CancelHandle,               // kills the process tree (also when dropped)
}

pub enum AgentEvent {
    SessionStarted { session_id, model },
    AssistantText { text },
    ToolCallStarted { call_id, tool, input_summary },
    ToolCallFinished { call_id, tool, ok, output_summary },
    Usage { input_tokens, output_tokens, cost_usd, duration_ms },
    Completed { summary },
    Failed { failure: AgentFailureKind, message }, // NotInstalled, NotAuthenticated, RateLimited, Crashed, Cancelled, ProtocolError, Interrupted
}
```

`AgentWorkspaces` (`prepare(task_id)` → empty folder, `release(task_id)`) gives each task's runs one folder; `TaskStore` (`save_task`, `load_tasks`, `append_task_event`, `load_task_events`) keeps tasks in `project.db`. Only `crates/se-agents/src/providers/<provider>/` knows a concrete CLI. Everything else uses the traits; the UI shows what `ProviderInfo` says (it never names a CLI).

### 9.2 Claude Code provider

`se_agents::providers::claude_code::ClaudeCodeProvider`; validated flags and observations in its `CLAUDE.md`.
- Binary resolution: user override (« Réglages → IA ») → `which` (honours PATHEXT on Windows, where `claude` is usually a `.cmd` shim) → the usual install folders of the OS (desktop apps do not inherit the shell's PATH).
- Detection: `claude --version`. Auth: `claude auth status --json` → `loggedIn` and the account kind (`subscriptionType`, else `authMethod`); e-mail and organisation are never read.
- Invocation (validated against Claude Code 2.1.317 and 2.1.319, logged in with a subscription):
  `claude -p --output-format stream-json --verbose --tools "" --permission-prompts none --setting-sources "" --restricted --disable-slash-commands --strict-mcp-config --append-system-prompt-file <workdir>/.simpleeffect-instructions.md [--mcp-config <json> --allowedTools "mcp__simpleeffect__*"] [--model m] [--effort e] (--session-id <uuid> | --resume <uuid>)`, in the task's working folder, the **prompt on stdin** (arguments with line breaks cannot reach a Windows `.cmd` shim).
- MCP config passed inline (prompt 18): `{"mcpServers":{"simpleeffect":{"type":"http","url":"http://127.0.0.1:<port>/mcp","headers":{"Authorization":"Bearer <task token>"}}}}`, redacted from logs.
- stream-json lines are parsed into `AgentEvent` (`system`/`init` → session, `assistant` → text and tool calls, `user` → tool results, `result` → usage then completed or failed); unknown line types are logged and ignored (forward compatible). Failures are classified from the result text or stderr (usage limit, login, lost session, crash, unreadable output).
- Cancellation kills the whole process tree (`process::CliProcess`: job object on Windows, process group on Unix).
- Tests use recorded stream-json fixtures (`tests/fixtures/claude_code/`) and a fake `claude` script; the scheduler's tests use se-app's `ReplayProvider`.

### 9.3 Tasks

- `Task { id, projectId, prompt, scope: TaskScope { sequenceId, range?, tracks? }, provider, model, effort, status, agentSessionId, createdAt, updatedAt, runs: [TaskRun { runKind: initial | retry | followUp, prompt, status, queuedAt, startedAt, finishedAt }], changesetId, usage }`; events apart (`TaskEventRecord`). A follow-up may replace the scope (same sequence); `usage` adds up the runs' `usage` events (prompt 20).
- Status: `Queued → Running → Completed | Failed { failure, message } | Cancelled` (the latest run's). Follow-ups resume the task's session in a new run; retries rerun the last prompt (resuming only once an earlier run completed).
- `TaskService` (se-app, the `TaskScheduler`): one FIFO queue for every open project, at most `maxConcurrentTasks` runs (setting 1–8, default 3), cancellation of queued or running runs (process tree killed), persistence of tasks and event logs in `project.db` (`taskUpdated`, `taskEventAppended`), tasks left queued or running by a stopped core failed as `interrupted` when their project reopens, tasks cancelled when their project closes. It also keeps the AI status (checked at startup, on demand and after failed runs; `aiStatusChanged`) and the AI settings, and refuses tasks while the CLI is known missing or logged out (`AI_UNAVAILABLE`).
- Each run: mark running → mint its MCP token (`TaskTokenRegistry`, bound to `AgentContext { project_id, task_id, run, scope }`) → prepare the working folder (`<app data>/agent-workdirs/<task id>/`) → `start` → record and publish each event (the session id is kept on the task) → release the folder → revoke the token → final status. Agent instructions are composed from `crates/se-app/assets/agent-instructions/*.md` (role, time and scope, reading the project, editing rules, motion basics, audio editing), the scope (timecodes, track names) and a summary of the project.
- Changesets (prompt 19, ADR-0029): the first committed edit of a task opens its changeset (the actor records it; `TaskService::attach_changeset` sets `changesetId`, `taskUpdated`). `TaskService::revert(projectId, taskId)` (protocol `revertTask`) refuses active tasks, then asks the actor to revert; when something was undone `changesetId` becomes `null` and a later follow-up opens a new changeset. `preview_revert` (protocol `previewTaskRevert`, prompt 20) has the actor plan and commit the revert on a copy and report it; `changes` (`getTaskChanges`) lists the open changeset's surviving items.

### 9.4 MCP server (`se-mcp`)

Decisions: ADR-0003, ADR-0028. The contract with agents (conventions, every tool's input, output and errors): `docs/ai-tools.md`.

- `rmcp` streamable HTTP service (`se_mcp::mcp_service`, sessions in memory) mounted at `/mcp` by se-core (`mcp_routes.rs`), outside the session-token guard and behind its own: loopback `Host` (403) and `Authorization: Bearer <run token>` resolved by `se_app::TaskTokenRegistry` (401 for unknown or revoked tokens). Tokens are minted when a run starts and revoked when it ends; the handler resolves the token again on every call (`AgentContext`, `ACCESS_REVOKED`).
- `McpServer` (`ServerHandler`: `tools/list`, `tools/call`) over a registry of `McpTool`s, one per file under `crates/se-mcp/src/tools/`: input type (`serde` + `schemars`, unknown fields refused), handler calling se-app use cases, tests (handlers over `MediaTestbed`, schema snapshots with `insta`), a section in `docs/ai-tools.md`.
- Read tools (prompt 18): `get_project_overview`, `get_timeline`, `get_item`, `list_assets`, `get_frames` (composited sequence frames drawn by the renderer, returned as images), `get_source_frames` (asset frames via ffmpeg), `get_audio_analysis`, `get_scene_changes` (asset in source time, or a sequence range brought back through each clip's source window and speed), `list_animation_presets`, `get_transcript` and `search_transcript` (prompt 21: words of a sequence range in sequence time through `TranscriptionService::sequence_words`, paginated; full-text search mapped to where the sequence plays each phrase), and stubs answering `available: false` until their prompts: `get_brand_kit` (25), `list_templates` (24).
- `get_frames`: `se_app::FrameRenderService` publishes `renderFramesRequested` through the `RendererLink` port (se-core's event bus; refused at once without an `/rpc` connection), the renderer (`useAgentFrameRequests`, in `AppLayout`) draws with `@se/engine`'s `StillRenderer` (the viewer's `FrameRenderer` + compositor, offscreen) and uploads JPEGs to `PUT /media/renders/{id}/{index}`; the service answers in time order or fails after 30 s.
- Times in tool I/O are seconds or timecode strings in, `{"timecode": "HH:MM:SS.mmm", "seconds": n}` out (`se_domain::format_clock`), converted to flicks at the boundary (`se_mcp::time_io`). Outputs are compact JSON with ids and revisions, long lists paginated; errors are `isError` results with stable codes and actionable messages.
- Write tools (prompt 19, ADR-0029): `add_text`, `add_shape`, `add_image`, `update_item`, `set_keyframes`, `apply_animation_preset`, `set_volume`, `move_item`, `trim_item`, `split_at`, `delete_items`, `ripple_delete_range`, `insert_gap`, `add_transition`, `add_marker`, and `apply_batch` (several actions as one transaction, `"as"`/`"$name"` references); `get_task_changes` reads the task's changeset. Smart cuts (prompt 22, ADR-0032): `remove_words`, `remove_fillers`, `remove_silences` build a `se_app::smart_cuts::CutRequest`, read the media facts (`SmartCutService::media`: transcripts and level envelopes) and call `AgentEditService::apply_cuts`, which resolves the cut on the current document limited to the task's range and checks the **cut scope rule** (removed time inside the range, edited tracks among the task's tracks; what follows may move); not batchable. Each implements `WriteTool::plan` (its input as `se_app::agent_edits::AgentEdit`s against a `WriteScene`: the document, the task's sequence, the batch's names) and is run by `writes::run` → `AgentEditService::apply`. Captions (prompt 23, ADR-0033): `create_captions` (words read through `CaptionService::spoken_words`, then `AgentEdit::AddCaptions`; not batchable), `update_caption_style` (preset and/or fields, `style_id` in place or `item_ids` copy on write), `edit_caption_text`, `emphasize_caption_words` (batchable). Later prompts add `create_motion_template`, `instantiate_template` (24).
- `se_app::AgentEditService` (`agent_edits/`): plans the edits in order on a draft (`EditPlan`; graphics on the task's free tracks, link-aware moves/trims/splits/deletes, ripple edits on the concerned tracks, transitions shortened to the cut's room, markers, presets), applies the whole transaction to a draft and checks the scope on every touched entity **before and after** (items entirely inside the range and on the task's tracks, markers in the range, transitions joining items in scope, no track header or sequence change, tracks added only with every track allowed, never assets), then commits it with `apply_ai_on` (replanned when the project moved meanwhile). Refusals: `OUT_OF_SCOPE` with the scope, the faulty item and times, and what to do (`the edit would leave text « Late » … on V3, 00:00:08.000–00:00:10.000 outside this task's scope (00:00:01.000–00:00:09.000 of « Main edit », every track): keep every change between …`); domain refusals become `REVISION_CONFLICT`, `LOCKED_TRACK`, `*_NOT_FOUND` or `VALIDATION_ERROR` at the MCP boundary.
- Agent instructions (role, time formats and scope, reading the project, editing rules, motion basics, audio editing, smart cuts, captions) live in `crates/se-app/assets/agent-instructions/*.md`, embedded at compile time and composed per run. They are provider-agnostic.

## 10. Motion templates

- A template = `{ id, name, version, params_schema (JSON Schema subset), source (TypeScript module), default_duration, thumbnail }`.
- The module exports `render(ctx: TemplateRenderContext, time: number, params: Params): void`, drawing with a restricted 2D API (subset of Canvas2D + helpers: text layout, easing, shapes, images from allowed assets).
- Executed in a dedicated Worker with no network (CSP), no protocol client, no DOM; frames returned as `ImageBitmap`. Time budget per frame; errors are captured and reported to the agent so it can fix its code.
- The inspector auto-generates controls from `params_schema` (string, number with range, color, boolean, enum, font, asset).
- Libraries: project templates (in `project.json`) and global templates + brand kits (app data dir).

## 11. Persistence (`se-store`)

```
<Name>.seproj/
├─ project.json       canonical document (schemaVersion, revision), pretty JSON, atomic write (tmp + fsync + rename)
├─ project.json.bak.1 previous snapshots (.bak.1 newest … .bak.3)
├─ project.db         SQLite (WAL, synchronous=FULL): op_journal, tasks, task_events, changesets + changeset_entries, transcripts + transcript_words + transcript_segments + transcript_search (FTS5), analyses
├─ .lock              held with an OS file lock while the project is open (never deleted)
├─ cache/             regenerable derived files: media/<cache key>/ (thumbnails, peaks), later mezzanines, proxies, overlay renders
└─ exports/
```

- Implemented by `se-store` (`FsProjectRepository`, `FsProjectStore`, `SqliteAppLibrary`); decisions in ADR-0017. The folder name comes from the project name (characters Windows refuses become `-`); renaming a project changes its document, not its folder; duplicates get `<Name> 2.seproj`… next to the source.
- Migrations: `schemaVersion` + ordered pure functions `migrate_vN_to_vN+1(serde_json::Value)` (`se-store/src/document_migrations.rs`, v1 is the base format), covered by `tests/fixtures/project-v1.json`. SQLite schemas: `PRAGMA user_version` + append-only SQL migrations. Journal rows carry the `Transaction` JSON and a `format_version`. Anything newer than this build is refused (`SCHEMA_TOO_NEW`).
- Crash recovery: on open, load `project.json` then replay journal entries with `revision > snapshot.revision` (`se_app::recovery::replay`); the recovered state is saved as the new snapshot.
- Snapshots are written 2 s after the last commit and on close (also when the core shuts down); the journal is truncated after a successful snapshot.
- Trash: the OS trash through the `trash` crate (NSFileManager on macOS, no automation prompt); never a permanent deletion.
- Document schema version 2 (prompt 08): assets carry a `fingerprint` and a per-stream `decodability` (`migrate_v1_to_v2`). Version 3 (prompt 09): assets carry `analysis` summaries (`migrate_v2_to_v3`; `Asset.analysis` defaults to empty so older journal entries replay). Versions 4–5 (prompts 14–15): transitions, fade shapes, text styles. Version 6 (prompt 16): filling clips carry a `focus` (`migrate_v5_to_v6` centres it). Version 7 (prompt 21): `ProjectSettings.transcription` (language `fr`, empty vocabulary) and `Asset.analysis.transcript` (`migrate_v6_to_v7`).
- `project.db` schema version 5 (prompt 21, ADR-0031): version 1's empty transcript tables are replaced by `transcripts` (`asset_id`, `content_key`, `language`, `model`, `backend`, `created_at`, `format_version`), `transcript_words` (asset, index, text, start and end in flicks, confidence, segment) and `transcript_segments` (asset, index, start, end, text), cascading from `transcripts`, and the FTS5 table `transcript_search` (`asset_id`, `segment` unindexed, `text`; tokenizer `unicode61 remove_diacritics 2`). Saving a transcript replaces the asset's rows and index entries in one SQLite transaction; rows of a newer `format_version` read as absent. Read and written through `TranscriptStore` (`FsTranscriptStore`).
- `project.db` schema version 4 (prompt 19, ADR-0029): `changesets` (`id`, `task_id`, `created_at`, `reverted_at`) and `changeset_entries` (`changeset_id`, `sequence`, `transaction_id`, `format_version`, payload: a `ChangesetEntry` JSON — transaction, inverse, revisions left). An AI commit writes its journal row and its changeset entry in one SQLite transaction; a revert marks the changeset reverted in the transaction journaling the revert. Open changesets are loaded with the project; reverted ones stay.
- `project.db` schema version 3 (prompt 17): `tasks` rows hold a `Task` JSON (and its status kind), `task_events` rows an `AgentEvent` JSON with its `run` and `sequence`; both carry a `format_version` (newer rows are skipped). Read and written through `TaskStore` (`FsTaskStore`, sharing the store's connection).
- `project.db` schema version 2 (prompt 09): `analyses` rows record the `content_key` analysed and their payload `format_version` (`AnalysisResult` JSON); read and written through `AnalysisStore` (`FsAnalysisStore`, sharing the store's connection).
- App data dir (`SE_DATA_DIR`, i.e. Electron's `userData`): `library.db` (recent projects, settings such as `projects.lastParentDir`, the `media.*` keys and the `ai.*` keys: `ai.provider`, `ai.executablePath`, `ai.defaultModel`, `ai.defaultEffort`, `ai.maxConcurrentTasks`), `agent-workdirs/` (empty working folders of running agents, cleared at startup), `templates/`, `brand-kits/`, `models/whisper/` (transcription models, prompt 21), `runtimes/` (the GPU pack of whisper.cpp and its resumable download), `logs/`. New projects are proposed in `<Documents>/SimpleEffect`.

## 12. Export (`se-export`)

- **Video render**: orchestrated by se-core, executed by the engine's `ExportWorker`; presets (YouTube 16:9, Shorts/Reels/TikTok 9:16, square, custom).
- **Premiere package** (`<name>_premiere/`): `sequence.xml` (FCP7 xmeml), `sequence.otio`, `overlays/*.mov` (ProRes 4444 with alpha, rendered by the engine and encoded by ffmpeg), `captions.srt`, `README.txt` (import steps). Media references point to original files.
- **SRT** (prompt 23, ADR-0033): `write_srt` (UTF-8 with BOM, CRLF, `HH:MM:SS,mmm` to the nearest millisecond) and `parse_srt` (tolerant: BOM, LF/CR, dot milliseconds, missing numbers, cue settings, markup), `FsSubtitleFiles` implementing se-app's `SubtitleFiles` (atomic writes, Latin-1 fallback); cues come from `se_app::captions::subtitles::cues_of` (every caption of the sequence, lines of the style's length).
- Writers are pure functions from the domain model to text, snapshot-tested with insta.

## 13. Security model

- se-core listens on 127.0.0.1 only, on a random port, with a random 256-bit token (`?t=` query parameter, compared in constant time; 401 otherwise); rejects requests whose `Host` is not `127.0.0.1:<port>` or `localhost:<port>` (403, defeats DNS rebinding). The token is never logged.
- Per-run MCP tokens (256 bits) are minted when a run starts and revoked when it ends; `/mcp` checks the loopback `Host` and the bearer token (401 for unknown or revoked ones) and never sees the app's session token; tokens are never logged (ADR-0028).
- The renderer's frame uploads (`PUT /media/renders/…`) carry the session token like every `/media` route; CORS is answered for that route only (the renderer's origin differs from the core's).
- Agents run with `--tools ""` (no built-in tools), in an empty working directory, isolated from the user's own CLI configuration (no settings files, hooks, skills or other MCP servers: ADR-0027); prompts go on stdin.
- Renderer is sandboxed, with a CSP that only allows scripts from the app bundle and connections to `127.0.0.1`; motion templates run in an isolated Worker.
- The app never reads, stores or forwards AI CLI credentials: it asks the CLI (`claude auth status --json`) and reads only whether it is logged in and the account kind.
- The renderer can open web pages only through the bridge's `openExternal`, which the main process limits to `https:` URLs (installation guides).
- The only network access of se-core is the download of transcription models, the voice model and the GPU pack (ADR-0031): started by an explicit command after the user confirmed size and source, from pinned `https://` addresses (Hugging Face, GitHub releases), size and SHA-256 checked before the file is renamed into place (rustls with the system's verifier, se-transcribe only). Audio never leaves the machine.

## 14. Cross-platform notes

- Paths: always `PathBuf`/`path` APIs, never string concatenation; store original absolute paths plus a project-relative hint for relinking.
- Process spawning goes through one helper per crate that handles Windows specifics (`.cmd` shims, no console window, process-tree kill): `se_media::process::Sidecar`, `se_agents::process::CliProcess`. Text with line breaks never goes in arguments of a possible `.cmd` shim (stdin or a file instead).
- Desktop apps do not inherit the shell's PATH (macOS gives them `/usr/bin:/bin:/usr/sbin:/sbin`): CLIs installed by the user are also searched in their usual install folders.
- GPU: Chromium uses NVDEC/NVENC, VideoToolbox or VA-API depending on the OS; the engine always checks capabilities at runtime.
- Whisper backends (ADR-0031): CPU builds bundled on Windows and Linux; Metal compiled into the macOS build; CUDA on Windows through the GPU pack (`<app data>/runtimes/whisper-cuda-12.4/`, downloaded with consent); Linux CUDA or other GPUs through the user's own build (« Dossier de whisper.cpp »). GPU detection: `nvidia-smi` (Windows, Linux), the chip name on Apple Silicon. A failed GPU run is redone on the CPU (`-ng`). Windows builds need the Visual C++ runtime.

## 15. Extension points (recipes)

**Add an AI provider**
1. Create `crates/se-agents/src/providers/<name>/` implementing `AgentProvider`.
2. Map the CLI's streaming output to `AgentEvent`; add recorded fixtures and replay tests.
3. Describe it (`ProviderInfo`, `ProviderCapabilities`) and register it in se-core (`services.rs`, `TaskServiceDeps.providers`); the settings list its models and effort levels.
No change to MCP tools or instructions is needed.

**Add an MCP tool**
1. Add `crates/se-mcp/src/tools/<tool_name>.rs` (schema, handler → se-app use case, scope check for writes).
2. Register it in the tool router; add a unit test and a schema snapshot.
3. Document it in `docs/ai-tools.md`; mention it in the relevant agent instruction file if behaviour guidance is needed.

**Add an operation**
1. In se-domain, add `operation/<name>.rs` with the payload struct and its `Edit` impl (record the inverse and the touched entities; reuse `placement`, `reshape`, `run_steps`), then the variant in `Operation` and its name in `operation_payloads!` (recipe: `crates/se-domain/CLAUDE.md`).
2. Unit-test it with `operation::testing::apply_ok` (checks validity and the exact inverse); add it to the generator of `crates/se-domain/tests/operations/` so the property tests cover it.
3. Expose it through a se-app use case, then a protocol command and/or an MCP tool.

**Add an item kind**
1. Domain type + validation + operations in se-domain.
2. `evaluate` support and compositor drawing in @se/engine (+ golden frame).
3. Inspector panel in the renderer; FCP7/OTIO export behaviour in se-export.

**Add an export format** — implement the `Exporter` port in se-export with snapshot tests; register in se-core.

**Add a job kind** — add the variant, implement it in the relevant adapter, register priority/concurrency in `JobQueue`.

**Add a UI component** — in `packages/ui/src/<layer>/<component>/` with variants (CVA), a story, and a test; export from the package index.
