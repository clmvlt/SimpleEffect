# @se/engine

## Purpose
The rendering engine shared by preview and export (WYSIWYG, `docs/adr/0002-web-rendering-engine.md`): document evaluation at a time, WebCodecs decoding (via Mediabunny), PixiJS compositing, the Web Audio graph and the export worker. It has no UI.

Implemented progressively: capabilities (prompt 08), evaluation/decoding/compositing (12), playback and audio (13), export (27).

## Public API
- `@se/engine/capabilities` (`src/capabilities/`): `videoCodecString(VideoStreamInfo)` / `audioCodecString(AudioStreamInfo)` (WebCodecs codec strings: `avc1.PPCCLL`, `hvc1|hev1.P.C.L<level>.B0`, `vp09.PP.LL.DD`, `av01.P.LLM.DD`, `vp8`; `mp4a.40.x`, `opus`, `flac`, `mp3`, `vorbis`, `ac-3`, `ec-3`, little-endian PCM; `null` for ProRes, DNxHR…; a missing level is guessed from the size), `checkDecodability(kind, MediaInfo, support = globalThis)` → `{ video, audio }` (`StreamDecodability`; never rejects: refusals, thrown errors and missing APIs are `undecodable`), `CodecSupport` (injectable `VideoDecoder`/`AudioDecoder`/`ImageDecoder`).
- `@se/engine` (`src/index.ts`) exports nothing yet. Planned: `evaluate`, decoder pool, compositor, audio engine, export worker entry points.

## Rules and invariants
- No React, no `@se/ui`; no network calls except to se-core (`/media`).
- Preview and export use the same code path.
- Heavy work runs in Workers; always `close()` `VideoFrame`s (the decoder pool owns their lifetime).
- Always check `isConfigSupported()` before using a codec; fall back to mezzanines.

## Allowed dependencies
`@se/protocol` (types). See `docs/architecture.md` §4.1.

## How to test
- `pnpm vitest run --project engine` (or `pnpm test` at the root). `capabilities.test.ts` checks codec strings and drives `checkDecodability` with fake decoders.
- Unit tests as `*.test.ts`; golden-frame tests in Vitest browser mode from prompt 12.
