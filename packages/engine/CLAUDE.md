# @se/engine

## Purpose
The rendering engine shared by preview and export (WYSIWYG, `docs/adr/0002-web-rendering-engine.md`): document evaluation at a time, WebCodecs decoding (via Mediabunny), PixiJS compositing, the Web Audio graph and the export worker. It has no UI.

Currently an empty skeleton, implemented by prompts 12 and 13 (export in 27).

## Public API
None yet (`src/index.ts` exports nothing). Planned: `evaluate`, decoder pool, compositor, audio engine, export worker entry points.

## Rules and invariants
- No React, no `@se/ui`; no network calls except to se-core (`/media`).
- Preview and export use the same code path.
- Heavy work runs in Workers; always `close()` `VideoFrame`s (the decoder pool owns their lifetime).
- Always check `isConfigSupported()` before using a codec; fall back to mezzanines.

## Allowed dependencies
`@se/protocol` (types). See `docs/architecture.md` §4.1.

## How to test
- `pnpm vitest run --project engine` (or `pnpm test` at the root).
- Unit tests as `*.test.ts`; golden-frame tests in Vitest browser mode from prompt 12.
