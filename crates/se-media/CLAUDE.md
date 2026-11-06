# se-media

## Purpose
Media adapter: probing (`ffprobe`), thumbnails, waveform peaks, mezzanines, proxies and analyses (`ffmpeg`), run as sidecar processes (`docs/adr/0007-media-pipeline-sidecars-mezzanine.md`). It implements `MediaProber` and `MediaJobRunner`; scheduling and priorities belong to the `JobQueue` in `se-app`.

Currently an empty skeleton, implemented by prompts 08 and 09.

## Public API
None yet. Planned: `MediaProber` and `MediaJobRunner` implementations.

## Rules and invariants
- Sidecar processes only: never link FFmpeg libraries.
- One spawn helper handles Windows specifics (no console window, process-tree kill on cancel) — `docs/architecture.md` §14.
- Binaries come from `resources/bin/<platform>/` (`pnpm fetch-binaries`), never committed.

## Allowed dependencies
`se-app`, `se-domain`, `tokio` (process), parsing libraries. Never another adapter crate. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-media`
- Parsers are unit-tested on recorded ffprobe/ffmpeg output; tests needing real binaries are marked and skipped when absent.
