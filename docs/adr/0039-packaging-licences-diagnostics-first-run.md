# 0039 — Packaging, licence notices, diagnostics archives and the first-run wizard

- Status: Accepted
- Date: 2026-11-10

## Context
Prompt 30 turns SimpleEffect into an installable application on Windows, macOS and Linux. It must
ship se-core and the sidecars (ffmpeg, ffprobe, whisper.cpp, ONNX Runtime) per platform, show the
licences of everything it bundles, let users send a diagnostic without leaking project content or
credentials, and guide a first launch (AI CLI installed and logged in, binaries, GPU and codecs,
optional models, projects folder). Constraints: no network access at runtime besides consented
downloads (golden rule 1), no paid signing or notarisation (personal use), no automatic updates,
release commands of `GUIDE-GIT.md` kept exact.

## Decision
- **electron-builder** (`apps/desktop/electron-builder.yml`): Windows NSIS x64, per-user, folder
  choice allowed, app data kept at uninstall; macOS dmg for arm64 (`macos-latest`) and x64
  (`macos-15-intel`, without ONNX Runtime), **ad-hoc signed** (`identity: '-'`, no hardened
  runtime: Apple Silicon refuses unsigned code, a Developer ID costs money); Linux AppImage + deb.
  Installer names are stable and platform-explicit:
  `SimpleEffect-<version>-windows-x64-setup.exe`, `-macos-arm64.dmg`, `-macos-x64.dmg`,
  `-linux-x86_64.AppImage`, `-linux-amd64.deb`. The icon is `build-resources/icon.png` (1024 px,
  source `icon.svg`); electron-builder derives every platform's format from it.
- **Resources layout**: `resources/bin/` holds se-core and the sidecars (unchanged: se-core looks
  next to itself first); `resources/LICENSES/` holds the repository's `LICENSES/` folder plus the
  generated notices. The main process resolves both with pure functions
  (`core-binary-path.ts`, `resource-paths.ts`) tested against each platform's packaged layout.
- **Licence notices are generated at build time**, from what actually ships: a Vite plugin
  (`tooling/bundled-packages-plugin.ts`) lists the npm packages of each bundle (main, preload,
  renderer, workers) from Rollup's module ids; `scripts/generate-notices.mjs` adds the crates
  linked into se-core (`cargo metadata --filter-platform <host>`, normal dependencies only, no
  proc-macros) and the hand-kept `LICENSES/components.json` (FFmpeg, whisper.cpp, ONNX Runtime,
  models, Electron, fonts), reads each package's licence files and writes
  `third-party-notices.json` (identical texts stored once, `formatVersion` 1) and
  `THIRD-PARTY-NOTICES.txt`. `pnpm build` runs it; « À propos → Licences » reads the JSON through
  the bridge (`getThirdPartyNotices`, validated by the main process).
- **Diagnostics are written by the core** (`exportDiagnostics`, se-app `DiagnosticsService`): the
  renderer sends what only it knows (versions, the WebGL GPU name, WebCodecs support, locale); the
  core adds its info, the AI, media, transcription and vision statuses and settings, and its
  newest log files (7 at most, 2 MiB each), then masks everything with one `Redactor`: the home
  folder (any spelling) becomes `~`, media and project file names become FNV-1a hashes, tokens,
  bearer values and long secrets become `<redacted>`, e-mail addresses `<email>`. No project
  content is read. The archive is a zip with `README.txt`, `report.json` and `logs/`.
- **First-run wizard** (`features/onboarding`): a step dialog — Bienvenue, Assistant IA, Binaires
  & GPU, Modèles (facultatif), Dossier des projets — opened once per machine (a versioned key in
  `uiPreferencesStorage()`, injected through the route dependencies so tests choose), reopened
  from « À propos ». It reuses existing commands and the settings' models list (downloads keep
  their consent); the projects folder is saved by the new `setProjectsFolder` command (the
  library's `projects.lastParentDir`). The face detection model is embedded, so only Whisper
  models are offered.
- **« À propos »** (`features/about`): versions (app, se-core, engine, Electron, Chromium, the AI
  CLI), « Copier », « Exporter un diagnostic », « Assistant de bienvenue », the licences and the
  performance measure (moved from « Réglages », prompt 29). The status bar's version opens it.
- **Release notes** come from `scripts/release-notes.mjs`: feat/fix/perf Conventional Commits
  since the previous tag, grouped under French headings, then the installation notes of every
  platform (SmartScreen, Gatekeeper `xattr -dr com.apple.quarantine`, AppImage/deb).

## Consequences
- Easier: the notices always match the shipped code; a diagnostic can be attached to a bug report
  without reviewing it by hand; new users find a missing or logged-out CLI before their first task.
- Harder: `pnpm dev` has no notices until a `pnpm build` (the screen says so); a new sidecar or
  bundled asset needs an entry in `LICENSES/components.json`; anything new written to the logs
  must stay maskable (paths through the home folder, no raw credentials).
- Users see SmartScreen once on Windows and must clear the quarantine on macOS until the app is
  signed by a certificate.

## Alternatives considered
- **`license-checker` / `cargo-about`**: list every installed dependency (dev tools included) or add
  a binary tool to install in CI; the bundle-based list is exact and has no extra dependency.
- **Notices written by hand**: drift as dependencies change.
- **Diagnostics zipped by the main process**: it would duplicate the core's knowledge of statuses
  and logs, and the masking would have to live in TypeScript too.
- **A first-run page instead of a dialog**: routes are for documents (home, editor); a dialog can be
  reopened anywhere and skipped at any step.
- **Unsigned macOS builds**: refused outright on Apple Silicon; ad-hoc signing is free.
