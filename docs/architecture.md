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
- **Local only.** No runtime network access except the AI CLI process itself and explicit, user-approved model downloads.
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
│  ├─ se-agents/             AI CLI providers (claude_code first), stream parsing
│  ├─ se-mcp/                MCP server: tool definitions → use cases
│  ├─ se-export/             FCP7 XML, OTIO, SRT, render orchestration
│  ├─ se-core/               binary: composition root, axum server
│  └─ se-codegen/            dev tool: exports Rust types to TypeScript (`pnpm codegen`)
├─ packages/
│  ├─ protocol/              @se/protocol: generated TS types + typed RPC client
│  ├─ engine/                @se/engine: evaluation, decode, composite, audio, export
│  └─ ui/                    @se/ui: design system (shadcn-style, editor components)
├─ scripts/                  Node scripts behind pnpm commands (check, dev, build, package, set-version, codegen…)
└─ resources/bin/<platform>/ ffmpeg, ffprobe, whisper-cli (gitignored, fetched)
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
    ├── se-agents  │
    ├── se-export  │
    └── se-mcp ────┘
          ▲
       se-core (composition root: depends on everything, nothing depends on it)
```

| Crate | May depend on | Must NOT |
|---|---|---|
| `se-domain` | serde, thiserror, uuid, ts-rs (derive only) | do IO, use async/tokio, read the clock, spawn anything |
| `se-protocol` | se-domain, serde, serde_json, ts-rs | contain logic |
| `se-app` | se-domain, se-protocol, tokio, async-trait, tracing | touch `std::fs`, `std::process`, network, concrete adapters |
| adapters | se-app, se-domain, their IO libraries | depend on another adapter crate |
| `se-mcp` | se-app, se-domain, rmcp, schemars | mutate documents except through se-app use cases |
| `se-core` | everything | contain domain logic |
| `se-codegen` | se-protocol (later se-domain), ts-rs | be depended on, ship in the app |

Adapters implement **ports** declared in `se-app::ports`: `ProjectRepository`, `AppLibraryRepository`, `MediaProber`, `MediaJobRunner`, `Transcriber`, `SubjectDetector`, `AgentProvider`, `Exporter`, `EventSink`, `Clock`, `IdGenerator`.
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

### 4.3 Features

`projects` (home, create/open) · `media-bin` · `timeline` · `viewer` · `inspector` · `ai` (prompt composer, tasks panel, scope overlays) · `transcript` · `captions` · `templates` (library, brand kit) · `export` · `settings`.

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
- `preload/`: exposes a minimal typed API via `contextBridge` as `window.simpleEffect` (`getCoreConnection()`; later `getPathForFile(file)`, `openDialog(...)`). Built as CommonJS because sandboxed preloads cannot be ES modules.
- `shared/`: the bridge contract (types and IPC channel names) bundled into main, preload and renderer.
- Security: `contextIsolation: true`, `nodeIntegration: false`, `sandbox: true`, no pop-ups or navigation, strict CSP written as a meta tag at build time (`tooling/`): production connects only to `127.0.0.1`, scripts come only from the app bundle; inline `<style>` elements are allowed because Radix and sonner insert them (ADR-0013).

**se-core process contract**: stdout carries exactly one handshake line; stdin stays open and its end-of-file means "shut down" (so the core dies with the app, even if the app crashes); `SE_DATA_DIR` (set by the supervisor to `app.getPath('userData')`) is the app data folder, and logs go to daily files in `<SE_DATA_DIR>/logs/` (7 kept, filter `SE_LOG`) plus stderr in debug builds. The packaged binary lives in `resources/bin/`, the development one in `target/debug/`.

## 5. Core concepts

### 5.1 Time

- `Flicks(i64)`: 705,600,000 flicks per second. Exact for all common frame rates (incl. NTSC 1001 rates) and sample rates.
- `FrameRate { numerator, denominator }` (e.g. 30000/1001).
- `TimeRange { start: Flicks, duration: Flicks }`.
- `Timecode` parsing accepts `51s`, `51.5s`, `00:51`, `1:02.500`, `00:00:51:12` (frames, needs a frame rate). Formatting depends on the sequence frame rate.
- In TypeScript flicks are a branded `number` (`type Flicks = number & { __brand: 'Flicks' }`). Exact up to ±2^53 (~147 days); the domain rejects larger values.
- Conversions happen only at boundaries: UI display, MCP tool input/output, export writers.

### 5.2 Identifiers

UUIDv7 strings wrapped in newtypes (`ProjectId`, `SequenceId`, `TrackId`, `ItemId`, `AssetId`, `TaskId`, `ChangesetId`, `TemplateId`). Generated through the `IdGenerator` port (deterministic in tests).

### 5.3 Document model (summary — source of truth is `se-domain`)

```
Project { id, name, schema_version, revision, settings, assets, sequences, templates, brand_kit }
Asset { id, original_path, kind: Video|Audio|Image, probe: MediaInfo, derived: DerivedMedia, analysis_refs }
Sequence { id, name, format: Format { width, height, frame_rate, sample_rate }, tracks, markers }
Track { id, kind: Video|Audio, name, muted, solo, locked, hidden, volume, items }
Item { id, revision, origin: Origin, timeline_range, kind: ItemKind }
ItemKind = MediaClip { asset_id, source_start, speed, transform, opacity, volume, fades, reframe }
         | Text { content, style, transform, animations }
         | Shape { geometry, fill, stroke, transform }
         | Image { asset_id, transform }
         | MotionTemplate { template_id, params }
         | Caption { words, style_ref }
Animatable<T> = Static(T) | Keyframed(Vec<Keyframe<T>>)
Keyframe<T> { time: Flicks (item-relative), value: T, easing: Easing }
Easing = Linear | Hold | CubicBezier(x1, y1, x2, y2) | Preset(name)
Origin = User | AiTask(TaskId)
```

Invariants (validated in `se-domain`, property-tested):
- items on the same track never overlap;
- all times ≥ 0 and within the flicks safe range;
- a media clip's source range stays inside its asset duration;
- keyframes are sorted and inside the item duration;
- every referenced id exists.

### 5.4 Operations, transactions, history, changesets

- `Operation` is an enum of intent-level edits: `AddItem`, `RemoveItem`, `MoveItem`, `TrimItem`, `SplitItem`, `RippleDeleteRange`, `InsertGap`, `SetItemProperty`, `SetKeyframes`, `AddTrack`, `RemoveTrack`, `UpdateTrack`, `AddSequence`, `UpdateSequenceFormat`, `AddAsset`, `UpdateAsset`, `AddTemplate`, `UpdateTemplate`, `UpdateBrandKit`, …
- `apply(project, op) -> Result<(Project, InverseOps), DomainError>` is pure.
- A `Transaction { id, actor: Actor, label, ops }` is atomic: all ops apply or none.
- `Actor = User | AiTask(TaskId)`.
- **User history**: undo/redo stack of the user's transactions only.
- **Changeset**: all transactions of one AI task. Applied live (ADR-0005). "Revert task" computes the inverse of the changeset against the current state; items modified by someone else since then are reported as conflicts and skipped. A revert is itself a user transaction (so it can be undone).
- Every item carries a `revision`; write operations from agents may include `expected_revision` for optimistic concurrency.

### 5.5 ProjectActor (single writer)

- One tokio task per open project, owning the `Project`.
- Inbox messages: `Apply(Transaction)`, `Undo`, `Redo`, `RevertChangeset(id)`, `Snapshot` (query).
- On commit: validate → apply → append to journal (SQLite) → bump `revision` → emit `ProjectPatched` event → schedule debounced snapshot save.
- Readers get an `Arc<Project>` snapshot; no locks are held across awaits.

### 5.6 UI synchronisation

- On open, the renderer receives a full `ProjectSnapshot`.
- Each commit emits `ProjectPatched { revision, upserts, deletes }` with **whole entities** (items, tracks, sequences, assets…) replaced by id. The renderer never re-implements operation logic.
- Drags and trims use local ephemeral state in the timeline feature; the command is sent on drop, and the ephemeral state clears when the patch arrives.
- If a patch revision gap is detected, the client requests a fresh snapshot.

## 6. Protocol (`se-protocol` → `@se/protocol`)

Conventions and their rationale: ADR-0010 and ADR-0012.

- Transport: one WebSocket at `/rpc`. Messages are JSON, tagged by `type`:
  - `{"type":"request","id":…,"command":{"kind":…}}` → `{"type":"response","id":…,"result":{"kind":…}}` or `{"type":"response","id":…,"error":{code,message,details?}}` (`id` is `null` when the request could not be read).
  - `{"type":"event","event":{"kind":…}}` for server push. Every connection first receives `coreReady`; `coreShuttingDown` precedes the close on shutdown; `resyncRequired` tells a connection that fell behind that it missed events (later: project patches, job progress, task events).
- `Command`, `CommandResult`, `CoreEvent` are Rust enums (`#[serde(tag = "kind", rename_all = "camelCase")]`); every `Command` variant has a `CommandResult` variant of the same name, so `ResultFor<K>` is derived in TypeScript without a table.
- `pnpm codegen` runs the `se-codegen` binary (ts-rs) into `packages/protocol/src/generated/`; `pnpm codegen --check` (part of `pnpm check`) fails when the committed files are stale; `tsc -b` checks they compile.
- se-core side (`crates/se-core/src/rpc/`): `RpcRouter` parses a frame (`INVALID_REQUEST` / `UNKNOWN_COMMAND` with actionable messages), dispatches to one handler per command group (`rpc/handlers/`), encodes the response. `EventBus` (bounded `broadcast`) fans events out; a slow connection never blocks the others.
- Client side: `CoreClient` (`@se/protocol`) correlates ids, times requests out (30 s default), reconnects with a capped exponential backoff (re-resolving the endpoint each time), queues requests while disconnected and fails in-flight ones with `CONNECTION_LOST`. The renderer creates one client in `src/shared/core/` and exposes it through React context and hooks.
- Errors carry a stable `code` (`ErrorCode`, e.g. `UNKNOWN_COMMAND`, later `PROJECT_NOT_FOUND`, `REVISION_CONFLICT`, `OUT_OF_SCOPE`) and a human/agent-readable message. Codes raised by clients (`TIMEOUT`, `CONNECTION_LOST`, `CLIENT_CLOSED`) are part of the same enum.

## 7. Media pipeline (`se-media`)

- Binaries: `ffmpeg`, `ffprobe` resolved from `resources/bin/<platform>/` (packaged) or a user override; fetched in development with `pnpm fetch-binaries`.
- Import: probe → `Asset` → renderer checks `VideoDecoder.isConfigSupported()` / `AudioDecoder.isConfigSupported()` and reports decodability → core schedules jobs.
- Job kinds: `Thumbnails`, `WaveformPeaks`, `AudioAnalysis` (silences, loudness), `SceneDetection`, `Mezzanine`, `Proxy`, `Transcription`, `SubjectDetection`, `OverlayRender`.
- `JobQueue` (se-app): priorities, concurrency per kind, progress events, cancellation, retry; outputs cached under `cache/` keyed by a fast content hash (size + mtime + xxhash of first/last MiB).
- Decode source selection: original if decodable; otherwise mezzanine; preview may switch to proxy.
- `/media` routes (token as `?t=` query for `<img>`/media elements, header otherwise):
  `/media/assets/{id}/original|mezzanine|proxy`, `/media/assets/{id}/thumbnails/{index}`, `/media/assets/{id}/peaks/{level}`, `/media/renders/{job}` (upload of rendered chunks).

## 8. Rendering engine (`@se/engine`)

```
evaluate(sequence, time) ─► FrameGraph ─► Compositor.render(frameGraph, frames) ─► canvas
                                   ▲
                 DecoderPool (Mediabunny + WebCodecs, LRU cache of VideoFrame, prefetch window)
```

- `evaluate` is pure: resolves visible items, interpolates keyframes, applies easing, computes transforms, z-order.
- `DecoderPool`: one decoder per (asset, source variant); frame cache with strict `VideoFrame.close()` discipline.
- `Compositor` interface; `PixiCompositor` is the implementation (WebGL2, WebGPU optional). Works on `HTMLCanvasElement` and `OffscreenCanvas`.
- `AudioEngine`: Web Audio graph (clip gain → track gain/pan → master), lookahead scheduler, `AudioContext` clock is the master clock, meters via AudioWorklet.
- `PlaybackController`: play/pause/seek/shuttle (JKL), loop range, dropped-frame accounting.
- `ExportWorker`: same `evaluate` + `PixiCompositor` on `OffscreenCanvas`, `VideoEncoder`/`AudioEncoder`, Mediabunny muxer, streams output to `/media/renders/{job}`; alternative path pipes raw frames to ffmpeg through se-core for ProRes/DNxHR.
- `MotionTemplateRuntime`: runs AI-authored template modules in a dedicated Worker (see §10).
- Golden-frame tests run in Vitest browser mode (Chromium) with pixel tolerance.

## 9. AI architecture

### 9.1 Ports

```rust
#[async_trait]
pub trait AgentProvider: Send + Sync {
    fn id(&self) -> ProviderId;
    fn capabilities(&self) -> ProviderCapabilities; // resume, image input, models, effort levels
    async fn detect(&self) -> Result<ProviderInstallation, AgentError>; // path, version
    async fn auth_status(&self) -> Result<AuthStatus, AgentError>;      // LoggedIn | LoggedOut | Unknown
    async fn start(&self, spec: AgentRunSpec) -> Result<AgentRun, AgentError>;
}

pub struct AgentRunSpec {
    pub prompt: String,
    pub system_instructions: String,
    pub mcp: McpEndpoint,              // url + per-task bearer token
    pub model: Option<String>,
    pub effort: Option<EffortLevel>,
    pub resume_session: Option<AgentSessionId>,
    pub working_dir: PathBuf,          // empty, task-specific directory
}

pub struct AgentRun {
    pub events: BoxStream<'static, AgentEvent>,
    pub cancel: CancelHandle,
}

pub enum AgentEvent {
    SessionStarted { session_id },
    AssistantText { text },
    ToolCallStarted { tool, input_summary },
    ToolCallFinished { tool, ok, output_summary },
    Usage { input_tokens, output_tokens, cost_usd },
    Completed { summary },
    Failed { kind: AgentFailureKind, message }, // NotInstalled, NotAuthenticated, RateLimited, Crashed, Cancelled, ProtocolError
}
```

Only `crates/se-agents/src/providers/<provider>/` knows a concrete CLI. Everything else uses the trait.

### 9.2 Claude Code provider

- Binary resolution: user override → `which` (honours PATHEXT on Windows, where `claude` is usually a `.cmd` shim).
- Detection: `claude --version`. Auth: `claude auth status --json` → `loggedIn`.
- Invocation (validated against Claude Code 2.1.312):
  `claude -p <prompt> --output-format stream-json --verbose --mcp-config <json> --strict-mcp-config --tools "" --allowedTools "mcp__simpleeffect__*" --permission-prompts none --append-system-prompt <instructions> [--model m] [--effort e] (--session-id <uuid> | --resume <uuid>)`.
- MCP config passed inline: `{"mcpServers":{"simpleeffect":{"type":"http","url":"http://127.0.0.1:<port>/mcp","headers":{"Authorization":"Bearer <task token>"}}}}`.
- stream-json lines are parsed into `AgentEvent`; unknown line types are logged and ignored (forward compatible).
- Cancellation kills the whole process tree.
- Tests use a replay provider fed with recorded stream-json fixtures.

### 9.3 Tasks

- `Task { id, prompt, scope: TaskScope { range: Option<TimeRange>, tracks: Option<Vec<TrackId>>, sequence_id }, provider, model, effort, status, agent_session_id, changeset_id, created_at, events }`.
- Status: `Queued → Running → Completed | Failed | Cancelled`; follow-ups (`resume`) create a new run in the same task.
- `TaskScheduler` (se-app): FIFO queue, semaphore (default 3 concurrent runs), persists tasks and event logs in `project.db`, emits `TaskEvent`s to the UI.

### 9.4 MCP server (`se-mcp`)

- `rmcp` streamable HTTP service mounted at `/mcp` on the core's axum router.
- The bearer token identifies the task → tool calls get an `AgentContext { task_id, scope, sequence_id }`.
- One tool per file under `crates/se-mcp/src/tools/`, each with: input schema (schemars), output type, handler calling a se-app use case, unit test, and an entry in `docs/ai-tools.md`.
- Read tools: `get_project_overview`, `get_timeline`, `get_item`, `list_assets`, `get_transcript`, `get_audio_analysis`, `get_frames` (composited sequence frames rendered by the renderer, returned as images), `get_source_frames` (raw asset frames via ffmpeg), `get_scene_changes`, `list_animation_presets`, `get_brand_kit`, `list_templates`.
- Write tools: one per intent (`add_text`, `update_item`, `delete_items`, `split_at`, `trim_item`, `move_item`, `ripple_delete_range`, `insert_gap`, `set_keyframes`, `apply_animation_preset`, `add_shape`, `add_image`, `set_volume`, `create_captions`, `create_motion_template`, `instantiate_template`, …) plus `apply_batch` for atomic multi-step edits.
- Times in tool I/O are human formats (seconds or timecode strings), converted to flicks at the boundary.
- Every write is checked against the task scope; errors are actionable (`OUT_OF_SCOPE: item ends at 00:00:55.000, task scope is 00:00:51.000–00:00:54.000`).
- Agent instructions (role, editing rules, time formats, scope rules, motion/caption/audio guides) live in `crates/se-app/assets/agent-instructions/*.md`, embedded at compile time and composed per task. They are provider-agnostic.

## 10. Motion templates

- A template = `{ id, name, version, params_schema (JSON Schema subset), source (TypeScript module), default_duration, thumbnail }`.
- The module exports `render(ctx: TemplateRenderContext, time: number, params: Params): void`, drawing with a restricted 2D API (subset of Canvas2D + helpers: text layout, easing, shapes, images from allowed assets).
- Executed in a dedicated Worker with no network (CSP), no protocol client, no DOM; frames returned as `ImageBitmap`. Time budget per frame; errors are captured and reported to the agent so it can fix its code.
- The inspector auto-generates controls from `params_schema` (string, number with range, color, boolean, enum, font, asset).
- Libraries: project templates (in `project.json`) and global templates + brand kits (app data dir).

## 11. Persistence (`se-store`)

```
<Name>.seproj/
├─ project.json   canonical document (schema_version, revision), atomic write (tmp + fsync + rename)
├─ project.db     SQLite (WAL): op_journal, tasks, task_events, transcripts (FTS5), analyses
├─ cache/         regenerable derived files (mezzanines, proxies, thumbnails, peaks, overlay renders)
└─ exports/
```

- Migrations: `schema_version` + ordered pure functions `migrate_vN_to_vN+1(serde_json::Value)`; covered by fixture tests.
- Crash recovery: on open, load `project.json` then replay journal entries with `revision > snapshot.revision`.
- Snapshots are written on a debounce (e.g. 2 s after the last commit) and on close; the journal is truncated after a successful snapshot.
- App data dir (via `dirs`): `library.db` (recent projects, settings, provider config), `templates/`, `brand-kits/`, `models/`, `logs/`.

## 12. Export (`se-export`)

- **Video render**: orchestrated by se-core, executed by the engine's `ExportWorker`; presets (YouTube 16:9, Shorts/Reels/TikTok 9:16, square, custom).
- **Premiere package** (`<name>_premiere/`): `sequence.xml` (FCP7 xmeml), `sequence.otio`, `overlays/*.mov` (ProRes 4444 with alpha, rendered by the engine and encoded by ffmpeg), `captions.srt`, `README.txt` (import steps). Media references point to original files.
- Writers are pure functions from the domain model to text, snapshot-tested with insta.

## 13. Security model

- se-core listens on 127.0.0.1 only, on a random port, with a random 256-bit token (`?t=` query parameter, compared in constant time; 401 otherwise); rejects requests whose `Host` is not `127.0.0.1:<port>` or `localhost:<port>` (403, defeats DNS rebinding). The token is never logged.
- Per-task MCP tokens are minted at task start and revoked when the run ends.
- Agents run with `--tools ""` (no built-in tools), in an empty working directory.
- Renderer is sandboxed, with a CSP that only allows scripts from the app bundle and connections to `127.0.0.1`; motion templates run in an isolated Worker.
- The app never reads, stores or forwards AI CLI credentials.

## 14. Cross-platform notes

- Paths: always `PathBuf`/`path` APIs, never string concatenation; store original absolute paths plus a project-relative hint for relinking.
- Process spawning goes through one helper per crate that handles Windows specifics (`.cmd` shims, no console window, process-tree kill).
- GPU: Chromium uses NVDEC/NVENC, VideoToolbox or VA-API depending on the OS; the engine always checks capabilities at runtime.
- Whisper backends: CUDA (Windows/Linux NVIDIA), Metal (macOS), CPU fallback.

## 15. Extension points (recipes)

**Add an AI provider**
1. Create `crates/se-agents/src/providers/<name>/` implementing `AgentProvider`.
2. Map the CLI's streaming output to `AgentEvent`; add recorded fixtures and replay tests.
3. Register it in se-core's provider registry; expose it in settings.
No change to MCP tools or instructions is needed.

**Add an MCP tool**
1. Add `crates/se-mcp/src/tools/<tool_name>.rs` (schema, handler → se-app use case, scope check for writes).
2. Register it in the tool router; add a unit test and a schema snapshot.
3. Document it in `docs/ai-tools.md`; mention it in the relevant agent instruction file if behaviour guidance is needed.

**Add an operation**
1. Add the variant to `Operation` in se-domain with `apply` + inverse.
2. Add property tests (apply then inverse is identity; invariants hold).
3. Expose it through a se-app use case, then a protocol command and/or an MCP tool.

**Add an item kind**
1. Domain type + validation + operations in se-domain.
2. `evaluate` support and compositor drawing in @se/engine (+ golden frame).
3. Inspector panel in the renderer; FCP7/OTIO export behaviour in se-export.

**Add an export format** — implement the `Exporter` port in se-export with snapshot tests; register in se-core.

**Add a job kind** — add the variant, implement it in the relevant adapter, register priority/concurrency in `JobQueue`.

**Add a UI component** — in `packages/ui/src/<layer>/<component>/` with variants (CVA), a story, and a test; export from the package index.
