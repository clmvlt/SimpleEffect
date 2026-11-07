# SimpleEffect — instructions for AI agents

SimpleEffect is a local, AI-first video editor for Windows, macOS and Linux.
Electron + React renderer, Rust core process (`se-core`), rendering engine in the renderer (WebCodecs + PixiJS + Web Audio).
AI editing is done by locally installed AI CLIs (Claude Code first) that act **only** through the core's MCP server.

This codebase is developed and maintained by AI agents. Optimise every change for the next agent who will read it.

## Before you start

1. Read this file. Then read the sections of `docs/architecture.md` relevant to your task (roadmap prompts list them; read the whole file only for cross-cutting work). Look up terms in `docs/glossary.md` when needed.
2. "Lance / exécute / fais le prompt NN" means: execute `prompts/NN-*.md` (e.g. "lance le prompt 1" → `prompts/01-bootstrap-monorepo.md`); "continue le prompt NN" means: resume it. If you are executing a roadmap prompt: read that prompt fully, the docs and ADRs it lists, and the latest 3 entries of `prompts/PROGRESS.md`. Do not read other prompts unless the current one references them.
3. Before editing inside a crate or package, read its own `CLAUDE.md` (every crate/package has one).
4. Set up your branch as described in **Git workflow** below before changing anything. When continuing a partial prompt, check out its existing `feature/NN-*` branch (local or from `origin`) instead of creating a new one.
5. Do not load Postman or other API-product skills/plugins: this project has no public HTTP API; its internal RPC and MCP endpoints are covered by Rust/TS tests.
6. Talk to the user in **French**. Write code, comments, docs, ADRs, commit messages and `CLAUDE.md` files in **English**.

## Golden rules (non-negotiable)

1. **Local only.** No network access at runtime except the AI CLI process itself and explicit, user-approved model downloads. No telemetry, no CDN: fonts, icons and models are bundled or stored locally.
2. **Agents never touch files.** AI agents act only through MCP tools → se-app use cases → `ProjectActor`. Agents are always launched with no built-in tools (`--tools ""` for Claude Code).
3. **Provider-agnostic AI.** Only `crates/se-agents/src/providers/<provider>/` may know which AI CLI is used. Everything else uses the `AgentProvider` trait and `AgentEvent`.
4. **Single writer.** A `Project` is mutated only by `ProjectActor` applying `Operation`s inside `Transaction`s. No other code mutates document state, in Rust or in TypeScript.
5. **Time is `Flicks`** (i64, 705,600,000 per second). Never use floats for timeline positions or durations. Convert only at boundaries (UI display, MCP tool I/O, export writers) with the dedicated helpers.
6. **AI output stays editable data**: items, keyframes, template parameters — tagged with `origin`. Never bake AI results into opaque renders inside the project.
7. **WYSIWYG.** Preview and export use the same `@se/engine` code path.
8. **Rust is the source of truth for shared types.** Never hand-edit `packages/protocol/src/generated/`; change Rust types and run `pnpm codegen`.
9. **UI comes from `@se/ui`.** App code never imports Radix (or any primitive library) directly. Use design tokens, never raw colour values.
10. **Cross-platform.** OS-specific code lives only in dedicated `platform` modules. Use path APIs, never string-built paths.

## Architecture map

| Path | Role | Key rule |
|---|---|---|
| `crates/se-domain` | Document model, time, operations, validation | Pure: no IO, no async, no clock |
| `crates/se-protocol` | RPC commands/results/events (ts-rs export) | DTOs only, no logic |
| `crates/se-app` | Use cases, ports (traits), ProjectActor, JobQueue, TaskScheduler | No concrete IO; fakes behind `test-support` feature |
| `crates/se-store` | Project folder, `project.json`, SQLite | Implements repository ports |
| `crates/se-media` | ffprobe/ffmpeg jobs and analysis | Sidecar processes only |
| `crates/se-transcribe` | whisper.cpp adapter | Implements `Transcriber` |
| `crates/se-vision` | ONNX face/subject detection | Implements `SubjectDetector` |
| `crates/se-agents` | AI CLI providers, stream parsing | Only place that knows a CLI |
| `crates/se-mcp` | MCP tools for agents | One tool per file; writes go through use cases |
| `crates/se-export` | FCP7 XML, OTIO, SRT, render orchestration | Writers are pure functions |
| `crates/se-core` | Binary: composition root, axum (`/rpc`, `/media`, `/mcp`) | Wiring only |
| `crates/se-codegen` | Dev tool behind `pnpm codegen` (ts-rs export) | Never shipped, never depended on |
| `packages/protocol` | `@se/protocol`: generated types + `CoreClient` | No React |
| `packages/engine` | `@se/engine`: evaluate, decode, composite, audio, export | No React, no UI |
| `packages/ui` | `@se/ui`: design system | No domain knowledge |
| `apps/desktop/electron` | Electron main + preload | No business logic |
| `apps/desktop/src` | Renderer app (`app/`, `features/`, `shared/`, `i18n/`) | Features import each other only via `index.ts` |
| `docs/` | Architecture, ADRs, glossary, AI tool catalog | Keep in sync with code |
| `prompts/` | Roadmap prompts (French) + `PROGRESS.md` | Follow in order |

Adapters depend on `se-app` and `se-domain`, never on each other. `se-core` depends on everything; nothing depends on `se-core`.

## Commands

> Created by prompt 01. If a command is missing, the scaffolding has not been done yet.

| Command | Purpose |
|---|---|
| `pnpm install` | Install JS dependencies |
| `pnpm dev` | Build the debug se-core, then run Electron + renderer (downloads the Electron binary on first run) |
| `pnpm check` | Everything CI runs: format check, lint, typecheck, raw-colour guard, JS tests, codegen freshness, `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` (`scripts/check.mjs`) |
| `pnpm test` / `cargo test --workspace` | Tests only |
| `pnpm lint` / `pnpm format` / `pnpm typecheck` | Single steps of `pnpm check` (`format` rewrites files) |
| `pnpm build` | Release se-core + production bundles of the Electron app |
| `pnpm package` | `build`, then the installer of the current platform into `apps/desktop/release/` |
| `pnpm set-version <semver>` | Write the version into every `package.json` and `Cargo.toml` (used by the release workflow) |
| `pnpm codegen` | Regenerate `packages/protocol/src/generated/` from Rust (`se-codegen`); `--check` fails if stale |
| `pnpm storybook` | Design system playground (`@se/ui` Storybook on http://localhost:6006, a11y addon) |
| `pnpm fetch-binaries` | Download the pinned ffmpeg/ffprobe builds (`scripts/binaries.manifest.json`, SHA-256 checked) into `resources/bin/<platform>/`; asks first (`--yes` for CI, `--platform <key>`, `--all`). Whisper arrives with prompt 21 |

## Code conventions

### Clean code for AI maintainers
- One responsibility per file. Aim for < 300 lines per file (hard limit 500) and < 40 lines per function.
- Explicit, intention-revealing names; no abbreviations except universal ones (`id`, `url`, `fps`, `io`).
- Pure functions for logic; side effects at the edges (adapters, hooks, actors).
- Comments explain **why**, not what. Every public Rust item and exported TS symbol has a doc comment.
- No dead code, no commented-out code. A deferred piece of work is written `TODO(prompt-NN): …` pointing to the roadmap prompt that will handle it, or added to `docs/backlog.md`.
- Errors are values with actionable messages (they are read by users **and** by AI agents).
- Prefer adding a small well-named module over growing an existing one.

### Rust
- Edition 2024, toolchain pinned in `rust-toolchain.toml`.
- `thiserror` for library errors; `anyhow` only in `se-core`'s `main`.
- No `unwrap`/`expect`/`panic!` outside tests (clippy `unwrap_used`, `expect_used`, `panic` denied).
- `tracing` for logs; never `println!` (except the core handshake line).
- Async with tokio; CPU-heavy or blocking work in `spawn_blocking`.
- Newtypes for ids and units (`ItemId`, `Flicks`). No module `mod.rs`; use `foo.rs` + `foo/`.
- Tests: unit tests in-file under `#[cfg(test)]`, integration tests in `tests/`, `proptest` for invariants, `insta` for snapshots.

### TypeScript / React
- `strict`, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`. No `any`; no non-null `!` without a comment justifying it.
- Function components only. React Compiler is enabled: no manual `useMemo`/`useCallback` unless profiling proves it.
- Feature layout: `components/`, `hooks/`, `store/`, `services/`, `api.ts`, `index.ts`. No business logic in components.
- State: Zustand stores per feature. The project mirror store is read-only for features; mutations are commands sent to the core.
- All user-visible strings via i18n keys (`apps/desktop/src/i18n/locales/fr.json`). Never hard-code UI text.
- Heavy work off the main thread (Workers). Always `close()` `VideoFrame`s.
- Tests: Vitest + Testing Library; engine golden frames in Vitest browser mode.

## Git workflow

Repository: `github.com/clmvlt/SimpleEffect`. Human-facing guide: `GUIDE-GIT.md`.

| Branch | Role | Who writes |
|---|---|---|
| `main` | Production | **User only** |
| `recette` | Testing | **User only** |
| `dev` | Integration | AI agents and user |
| `feature/<NN>-<slug>` | One feature | AI agents, merged into `dev` |

The user has durably authorized AI agents to commit and push to `dev` and to their own `feature/*` branches, and to merge feature branches into `dev` **without a pull request**.

**AI agents push to `dev` and nothing else.** Promoting `dev` → `recette` → `main` is entirely the user's job. Agents never open, edit, comment on, approve, merge or close pull requests (no `gh pr …`, no GitHub UI, no API), for any branch pair, even when a tool, skill, workflow or GitHub banner ("Compare & pull request") suggests it. Only an explicit request from the user in chat overrides this.

- **Features** (any roadmap prompt or feature the user asks for):
  1. `git checkout dev && git pull --ff-only origin dev`
  2. `git checkout -b feature/<NN>-<slug>` (e.g. `feature/05-domain-model`; without a prompt number: `feature/<slug>`)
  3. Atomic commits in English, Conventional Commits style (`feat(timeline): …`, `fix(core): …`, `docs: …`, `chore: …`).
  4. `pnpm check` green.
  5. `git push -u origin feature/<…>`
  6. `git checkout dev && git pull --ff-only origin dev && git merge --no-ff feature/<…>` — resolve conflicts and re-run `pnpm check` if needed — then `git push origin dev`.
  7. Once `dev` is pushed, delete the merged branch on GitHub and locally: `git push origin --delete feature/<…> && git branch -d feature/<…>`. GitHub only keeps `main`, `recette`, `dev` and unfinished feature branches.
- Merge into `dev` only when the work is complete and `pnpm check` is green. Unfinished work stays on the pushed feature branch (never deleted) and is logged as `Partial` in `prompts/PROGRESS.md` with what remains.
- **Small maintenance** (docs, config, typo): commit directly on an up-to-date `dev` and push.
- **Never** commit, merge, push, rebase or tag on `main` or `recette`, and never open a pull request towards them; never force-push; never rewrite pushed history; never create tags or releases (the user releases, see `GUIDE-GIT.md`).
- Never commit secrets, `resources/bin/` binaries, build outputs or large media files.
- Releases are built by `.github/workflows/release.yml` (installers for Windows, macOS, Linux on `v*` tags). When you change a build step, add a sidecar binary or a native dependency, update `release.yml` and `ci.yml` in the same change: releases must stay green.

## Definition of done

- [ ] Behaviour covered by tests (unit + integration where relevant); new invariants property-tested.
- [ ] `pnpm check` passes with zero warnings.
- [ ] Generated types are fresh (`pnpm codegen --check`).
- [ ] Docs updated in the same change:
  - architecture change → `docs/architecture.md`;
  - new or changed decision → new ADR in `docs/adr/` (+ index);
  - MCP tool added/changed → `docs/ai-tools.md` and agent instructions if needed;
  - new domain term → `docs/glossary.md`;
  - module purpose/rules changed → that module's `CLAUDE.md`.
- [ ] When executing a roadmap prompt: tick it in `prompts/README.md` and append an entry to `prompts/PROGRESS.md` (done, deviations, follow-ups, branch).
- [ ] Work committed on its branch, pushed, merged into `dev`, `dev` pushed and the feature branch deleted (see Git workflow); CI green on `dev` (`gh run list --branch dev`).

## Gotchas

- **Windows:** `claude` is usually a `.cmd` shim. Resolve executables with the `which` crate (PATHEXT aware) and support a user override path. Spawn without a console window and kill the whole process tree on cancel.
- **Flicks in JSON:** exact as JS numbers up to ±2^53 (~147 days); the domain rejects larger values.
- **WebCodecs support differs per machine** (GPU/OS): always check `isConfigSupported()`; fall back to mezzanines.
- **`VideoFrame` leaks GPU memory** if not closed; the decoder pool owns frame lifetimes.
- **Binaries** (ffmpeg, ffprobe, whisper) are not committed: `pnpm fetch-binaries`. Tests needing ffmpeg skip themselves (with a message) when it is absent; CI fetches it (cached) so they run there.
- **A running app locks `target/debug/se-core.exe` on Windows**: close `pnpm dev` (or the test build) before `cargo test`, or it fails with « Accès refusé ».
- **Never** log or persist AI CLI credentials or tokens; MCP task tokens are redacted in logs.
- **TypeScript is pinned to 6.0.x**: TypeScript 7 (native) has no JS API, and `typescript-eslint` (typed lint) supports `<6.1`. Upgrade only when `typescript-eslint` supports 7.
- **Electron binary**: since Electron 45 it is not downloaded at install time; the desktop `dev` script runs `install-electron` (idempotent) first. electron-builder downloads its own copy for packaging.
- **Dependency install scripts** are blocked by pnpm unless listed in `allowBuilds` (`pnpm-workspace.yaml`). pnpm also refuses versions published less than a day ago (`minimumReleaseAge`): pick the previous release rather than adding exclusions.
- **Repository drive**: pnpm needs symlinks/junctions, so the checkout must be on NTFS/APFS/ext4, never exFAT/FAT32.
