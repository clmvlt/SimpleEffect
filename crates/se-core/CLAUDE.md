# se-core

## Purpose
The core binary: composition root and local server. It binds `127.0.0.1` on a random port, prints the handshake, serves `/rpc` (WebSocket), `/media` (files with Range, frame uploads) and `/mcp` (AI agents, se-mcp), and wires adapters to `se-app` (`ProjectService` over `se-store`, `MediaService` + `JobQueue` over `se-media`, `TaskService` over `se-agents`, `FrameRenderService` over the event bus). It contains wiring only, never domain logic.

## Public API
The process contract (consumed by `apps/desktop/electron/main/core-supervisor.ts`):
- **stdout**: exactly one line, `{"port":<u16>,"token":"<64 hex>","version":"<semver>"}` (`se_protocol::CoreHandshake`). Nothing else is ever written to stdout.
- **Environment**: `SE_DATA_DIR` = app data folder (set by the supervisor to Electron's `userData`; default `<OS data dir>/SimpleEffect`). `SE_LOG` = `tracing` filter (default `info`, e.g. `SE_LOG=se_core=debug`).
- **Data**: `<SE_DATA_DIR>/library.db` (recent projects, settings), `<SE_DATA_DIR>/agent-workdirs/` (working folders of AI runs, leftovers removed at startup), `<SE_DATA_DIR>/models/whisper/` (transcription models), `<SE_DATA_DIR>/runtimes/` (the GPU pack of whisper.cpp). New projects are proposed in `<Documents>/SimpleEffect` (`<SE_DATA_DIR>/projects` without a documents folder).
- **Logs**: daily files `<SE_DATA_DIR>/logs/se-core.<YYYY-MM-DD>.log` (7 kept), plus stderr (plain text when not a terminal) in debug builds or when the log folder is unusable.
- **stdin**: kept open by the parent and never read for data; end-of-file means "shut down".
- **HTTP**: every route requires `Host: 127.0.0.1:<port>` or `localhost:<port>` (else 403) and `?t=<token>` (else 401) — except `/mcp`, which requires the bearer token of a running AI task instead.
- `GET /media/assets/{id}/original|thumbnails|thumbnails/sheet|thumbnails/poster|peaks/{level}|mezzanine|proxy` (`media_routes.rs`; mezzanine and proxy once ready): files of an asset of any open project, through tower-http's `ServeFile` (Range 206/416, `HEAD`, conditional requests), explicit content types for derived files, `Cache-Control: no-cache`; 400 for a malformed id, 404 for unknown assets, unknown levels and outputs not generated yet. CORS for reads (`media_cors.rs`: `GET`/`HEAD`, the `Range` header, `Content-Range`/`Content-Length`/`Accept-Ranges`/`ETag` exposed, origin mirrored): the renderer's page is `file://` when packaged and the Vite dev server in development, and the engine reads media with `fetch` — without it nothing decoded in `pnpm dev`.
- `PUT /media/renders/{request_id}/{index}` (`render_routes.rs`, ADR-0028): the renderer uploads one JPEG per frame an agent asked for (`renderFramesRequested`); 204, 400 (not a JPEG, index out of range), 404 (nothing waits for that request); body ≤ 8 MiB; CORS (`PUT`, `Content-Type`, origin mirrored) for this route only, since the renderer's page is another origin.
- `/mcp` (`mcp_routes.rs`, ADR-0028): se-mcp's streamable HTTP MCP service for AI agents, behind its own guard: loopback `Host` (403) and `Authorization: Bearer <token>` of a running run (`TaskTokenRegistry`; 401 with `WWW-Authenticate: Bearer` otherwise). Tokens are never logged.
- `GET /media/fonts/{id}` (`font_routes.rs`): a face of the font catalogue (`listFonts`): bundled faces from memory, installed ones through `ServeFile`; `font/woff2`, `font/ttf`, `font/otf` or `font/collection`, `Cache-Control: no-cache`; 404 for unknown ids.
- **Sidecars**: ffmpeg/ffprobe are searched next to the se-core binary (packaged), then in `resources/bin/<platform>/` of an ancestor folder (development), then on the `PATH`; the library setting `media.toolsDir` comes first (read at startup, changed at once by `updateMediaSettings`). Hardware encoders are detected in the background right after startup.
- `GET /rpc` (WebSocket): `se-protocol` messages. The core greets each connection with `coreReady`, answers requests (each in its own task), pushes events, and on shutdown saves and closes every open project (`projectClosed`), then sends `coreShuttingDown` and a close frame.

## Modules
| Module | Role |
|---|---|
| `main.rs` | Entry: resolve paths, init logging, tie child processes to the core, run the server |
| `platform.rs` | OS specifics: `tie_children_to_this_process()` (Windows: the core joins a kill-on-close job, so no sidecar outlives it, even one killed while process-wrap holds it suspended) |
| `app_paths.rs` | `AppPaths { data_dir, logs_dir }` from `SE_DATA_DIR` or the OS data dir; `library_db()`, `agent_workdirs()`, `whisper_models()`, `runtimes()`, `default_projects_dir()` |
| `system_ports.rs` | `SystemClock`, `UuidV7Ids`, `RandomTokens` (`TokenSource`: 256-bit CSPRNG hex), `EventSink` and `RendererLink` for `EventBus` (connected while an `/rpc` socket listens; requests published as `renderFramesRequested`) |
| `logging.rs` | `tracing` + `tracing-appender` (daily rolling file, non-blocking; hold the `LogGuard`) |
| `server.rs` | Bind, handshake, routes (`/rpc` + `/media` behind the access guard, `/mcp` behind its own), graceful shutdown (closes projects, publishes `coreShuttingDown`, then drains within `DRAIN_TIMEOUT`) |
| `services.rs` | Wiring of the use cases: the font catalogue (`se_fonts::LocalFontCatalog`, installed fonts read in a background task at startup), `ProjectService` (se-store, observed by the media service), `MediaService` (se-media prober, files, `FfmpegJobRunner` as runner and frame extractor, `FfmpegTools` (warmed up in a background task), the library for settings, `JobQueue` publishing to the event bus), `TaskService` (`ClaudeCodeProvider`, `FsAgentWorkspaces` after `remove_leftovers`, `McpAccess { url: http://127.0.0.1:<port>/mcp, tokens }`, the stored AI settings applied by `start`, the provider checked in a background task at startup) observing projects like the media service, the `TaskTokenRegistry` (`RandomTokens`), the `FrameRenderService` over the event bus and the `AgentEditService` (over `ProjectService` and `TaskService`; MCP write tools use it through `McpDeps.edits`); the `SmartCutService` (prompt 22, over projects, media, transcription and the library; RPC handlers and MCP tools share it); the `CaptionService` (prompt 23, over projects, transcription and se-export's `FsSubtitleFiles`; shared by RPC and MCP); one `JobQueue` shared by the media and transcription services; the `TranscriptionService` (prompt 21: `WhisperTranscriber` over a `WhisperLocator` for the current executable and the GPU pack under `runtimes()`, `HttpDownloader`, `FsModelFiles`, `FfmpegJobRunner` as speech extractor, models in `whisper_models()`, started in a background task) observing projects and the media service's imports. `main` installs rustls's ring provider (`se_transcribe::install_crypto_provider`) before anything builds an HTTP client |
| `media_routes.rs` | The `/media` router |
| `font_routes.rs` | The `/media/fonts/{id}` router over the `FontCatalog` (read CORS too) |
| `media_cors.rs` | `read_only_cors()`: the CORS layer of the read-only `/media` routes (assets, fonts) |
| `render_routes.rs` | The `PUT /media/renders/{request_id}/{index}` router over the `FrameRenderService` (CORS for this route) |
| `mcp_routes.rs` | `/mcp`: se-mcp's service behind the `Host` + task-token guard |
| `access_guard.rs` | Host + token middleware, `SessionToken` |
| `rpc.rs` | `RpcState` and the `/rpc` upgrade handler |
| `rpc/router.rs` | `RpcRouter`: frame → parse → async `dispatch` (one match arm per command) → encoded response; `router::testing::context()` builds a `HandlerContext` over in-memory ports for unit tests |
| `rpc/request.rs` | Frame parsing; `INVALID_REQUEST` vs `UNKNOWN_COMMAND` with actionable messages |
| `rpc/handlers.rs`, `rpc/handlers/<group>.rs` | `HandlerContext { version, paths, projects, media, tasks, fonts, renders, transcription, smart_cuts, captions }`; one file per command group: `system.rs` (`ping`, `getCoreInfo`), `projects.rs` (home screen commands), `editing.rs` (snapshot, sequences — create, rename, format, duplicate/version, remove —, undo/redo), `timeline.rs` (tracks, markers), `items.rs` (item edits of prompt 11), `sound.rs` (item patches, keyframes and transitions of prompt 14), `graphics.rs` (graphics, batched keyframes, presets and fonts of prompt 15), `media.rs` (import, relink, remove, jobs, decodability, decoder capabilities, derived copies, audio silences), `media_settings.rs` (settings, tools status, project cache), `tasks.rs` (submit, cancel, retry, follow-up, list, revert, events), `ai.rs` (AI status and settings), `frames.rs` (`reportFrameRenderFailure`), `transcription.rs` (prompt 21: status, preferences, downloads, model files, `transcribeAssets`, `getTranscript`, `searchTranscripts`, `setTranscriptionSettings`; dispatched by `RpcRouter::dispatch_transcription`, the main match still listing each command), `smart_cuts.rs` (prompt 22: settings, `removeWords`, `findFillers`, `previewSilenceCuts`, `removeSilences`; `RpcRouter::dispatch_smart_cuts`), `captions.rs` (prompt 23: generation, SRT import/export with absolute paths, text, emphasis, merge, split, shift, styles; `RpcRouter::dispatch_captions`). Paths must be absolute (`absolute_path`) |
| `rpc/event_bus.rs` | `EventBus` (bounded `broadcast`, 256 per connection); lag → `resyncRequired` |
| `rpc/connection.rs` | Per-socket loop (`biased` select: events, then finished responses, then incoming frames, then shutdown); one spawned task per request; tracing span per connection |
| `rpc/encode.rs` | Message serialisation with an `INTERNAL` fallback |
| `shutdown.rs`, `handshake.rs`, `error.rs` | stdin EOF / Ctrl+C, handshake line, fatal `CoreError` |

Adding a command: recipe in `crates/se-protocol/CLAUDE.md`. Publishing an event: `EventBus::publish` (clone the bus into whatever produces events).

## Rules and invariants
- `anyhow` only in `main`; everything else returns `CoreError` (`thiserror`) or `ProtocolError` (handlers).
- Never log the session token (`SessionToken` and `CoreHandshake` redact it in `Debug`; the access guard logs the path only, never the query).
- The process must never outlive its parent: stdin EOF and Ctrl+C trigger a graceful shutdown bounded by `DRAIN_TIMEOUT`. Its children must never outlive it: on Windows the kill-on-close job of `platform.rs` guarantees it even when the core is killed.
- Publishing an event never waits: a slow connection only delays itself and is told to resync when it overflows.
- Handlers are async and each request runs in its own task, so a slow command never delays events or other requests; requests of one client may complete in any order (ADR-0017).
- Handlers convert `se_app::AppError` with `?` (`From<AppError> for ProtocolError`); they hold no logic beyond argument checks.
- New routes go behind the access-guard middleware (router layer in `server.rs`); `/mcp` is the one exception, with its own guard (agents never see the session token).

## Allowed dependencies
Every workspace crate (composition root); nothing depends on `se-core`. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-core`
- Unit tests in-file (paths, access guard, request parsing, router, event bus, encoding, shutdown).
- `tests/core_process/` spawns the real binary (`CARGO_BIN_EXE_se-core`) with a temporary `SE_DATA_DIR` (`RunningCore::start_in(dir)` to restart on the same folder): handshake, 401/403, commands and error codes, `coreShuttingDown` delivered to two clients, shutdown on stdin close, log file contents (no token), projects (`projects.rs`: events of a commit, a commit surviving `kill`, the lock between two cores, the snapshot saved at shutdown) and media (`media.rs`: `/media` refuses a missing or wrong token; with ffmpeg installed, an import whose thumbnails and peaks are served with Range 206/416, and a renamed original flagged missing on reopening then relinked). `http_get` there is a raw HTTP/1.1 client.
- `mcp_routes.rs` unit tests serve `/mcp` over `MediaTestbed` on a loopback port and drive it with a real `rmcp` client (reqwest, no TLS): a minted token lists the 12 tools, reads the overview and gets a frame image (delivered to the fake renderer link), a revoked token is refused mid-session, an unknown token cannot connect; foreign `Host` → 403, missing token → 401 (`oneshot`). `render_routes.rs` tests uploads (JPEG check, CORS headers, late uploads) and the preflight.
- `media_routes.rs` unit tests drive the router with `tower::ServiceExt::oneshot` over `se_app::test_support::MediaTestbed` whose asset points at a real temporary file; `router::testing::context()` uses the same testbed.
- The media process test waits for the thumbnails and peaks jobs by kind: analyses run alongside and may finish first.
- A running `pnpm dev` (or any app using `target/debug/se-core.exe`) locks the binary on Windows: close it before `cargo test`. `call()` returns a response and the events pushed before it; `request()` and `next_frame()` skip the events the core publishes on its own (`aiStatusChanged` after the background provider check), which may arrive at any moment and made `coreShuttingDown` checks flaky on Windows CI.
- Manual run: `cargo run -p se-core` prints the handshake; press Ctrl+C or close stdin to stop. Logs land in `<OS data dir>/SimpleEffect/logs/` unless `SE_DATA_DIR` is set.
