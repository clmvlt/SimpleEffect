# @se/engine

## Purpose
The rendering engine shared by preview and export (WYSIWYG, `docs/adr/0002-web-rendering-engine.md`, `docs/adr/0022-rendering-engine-evaluation-decoding-compositing.md`): document evaluation at a time, WebCodecs decoding (via Mediabunny), PixiJS compositing, later the Web Audio graph and the export worker. It has no UI.

Implemented progressively: capabilities (prompt 08), playback sources (09), evaluation/decoding/compositing/preview (12), playback and audio (13), export (27).

## Public API (entry points in `package.json` `exports`)
| Entry | What | Loads |
|---|---|---|
| `@se/engine` (`src/index.ts`) | `animation/` + `evaluate/` + `FLICKS_PER_SECOND` | nothing (pure) |
| `@se/engine/animation` | `easedProgress(easing, p)`, `cubicBezierProgress`, `EASING_PRESET_CURVES` (CSS ease-in/out/in-out), `valueAt(animatable, localTime, interpolator, fallback)`, `keyframesValueAt`, `interpolateNumber`/`interpolateVec2`/`interpolateColor` (OKLab, premultiplied alpha), `parseColor`/`formatColor`/`rgbaToOklab`/`oklabToRgba`/`mixColors` | — |
| `@se/engine/evaluate` | `evaluate(sequence, assets, time) → FrameGraph`; `Evaluable{Sequence,Track,Item}` (structural: a domain `Sequence` or mirror entities fit); layer types; `designScale`, `assetDisplaySize`, `reframedBox`, `shapeBox`, `clipSourceTime`, `fadeFactor`, `resolveTransform` | — |
| `@se/engine/composite` | `Compositor` interface, `createPixiCompositor({ canvas, width, height, renderer?, background?, preserveDrawingBuffer? })`, pure `layerPlacement`/`pictureFit`/`outputScale` | PixiJS |
| `@se/engine/decode` | `DecoderPool`, `PictureLease`, `FrameTracker`/`pictureTracker`/`openPictureCounts`, `chooseVariant`, `MediaLocator`, `MediabunnyVideoSource`, `fetchImageBitmap`, `DecodeError` | Mediabunny |
| `@se/engine/preview` | `createPreviewEngine(options) → PreviewEngine` (viewer engine), `createEngineWith(compositor, pool, locator, schedule?)`, `FrameRenderer`, `PreviewState`, `MediaIssue` | PixiJS + Mediabunny: **import lazily** from UI code |
| `@se/engine/capabilities` | `videoCodecString`/`audioCodecString`, `checkDecodability(kind, MediaInfo, support)`, `checkDecoderCapabilities(support)` (prompts 08–09) | — |
| `@se/engine/sources` | `playbackSource(asset)` → `original` / `mezzanine` / `preparing` / `unavailable`, `isPlayable` | — |

## Internal architecture
```
src/
├─ time/flicks.ts          FLICKS_PER_SECOND, roundToFlicks
├─ animation/              easing.ts, cubic-bezier.ts, color.ts (OKLab), interpolate.ts
├─ evaluate/               evaluate.ts (tracks → layers), item-layer.ts (per item kind),
│                          transform.ts, timing.ts (source time, fades), geometry.ts (boxes), frame-graph.ts (types)
├─ composite/              compositor.ts (interface, Picture), layout.ts (pure geometry),
│                          pixi/pixi-compositor.ts, pixi/layer-nodes.ts (PictureNode, ShapeNode, TextNode)
├─ decode/                 decoder-pool.ts, frame-cache.ts (LRU + leases), decode-queue.ts, frame-tracker.ts,
│                          frame-time.ts (frameLookupTime), media-variant.ts, video-source.ts, image-loader.ts
├─ preview/                frame-renderer.ts (evaluate → pictures → compositor), engine.ts (PreviewEngine),
│                          preview-engine.ts (factory wiring Pixi + pool), media-issue.ts
├─ capabilities/, sources/ prompts 08–09
└─ testing/                document builders (document.ts) and fake decoding (fake-decoding.ts) for tests
test/golden/               golden-frame tests (browser), fixtures/ (committed media), references/ (PNGs)
```

Data flow: `FrameRenderer.render({ scene, time, proxy })` → `evaluate` → for each picture layer `chooseVariant` + `MediaLocator` → `DecoderPool.peek*` (sync) or `videoFrame`/`image` (async, cancellable) → `Compositor.render(graph, pictures)` → leases released.

## Rules and invariants
- No React, no `@se/ui`; no network calls except to se-core (`/media`). Preview and export use the same code path (`FrameRenderer` + `Compositor`).
- `evaluate` and everything in `animation/` are pure (no clock, no IO, no globals). Time is flicks; seconds appear only at the decoder boundary (`frameLookupTime`).
- **Frame lifetimes.** The `DecoderPool` owns every `VideoFrame` and `ImageBitmap`: it adopts them into the `FrameTracker` when decoded and closes them through it. Everyone else **leases** (`PictureLease.release()` exactly once, extra calls ignored). The cache never closes a leased picture; removed ones close on their last release. `Compositor.render` must upload pictures before returning and keep no reference that it could read later (Pixi texture sources use `autoGarbageCollect: false` and are re-pointed on the next render). Pixi never closes resources. Destroying a pool closes everything not leased.
- Cache limits matter: hardware decoders stall when too many decoded frames are held (default 48 pictures, 256 MiB).
- Pictures are cached by source **and URL**; a changed URL (relinked file, regenerated proxy, restarted core) is new content.
- A picture's box comes from the asset (`assetDisplaySize`), never from the decoded frame: originals, mezzanines and proxies fill the same box.
- Media time 0 is the first frame (`max(0, first timestamp)`). Ask for frames with `frameLookupTime(sourceTime, assetFrameRate)` (mid-frame), never with raw seconds.
- Always check decodability before decoding (`track.canDecode()` in `MediabunnyVideoSource`; `capabilities/` for imports); fall back to mezzanines (`chooseVariant`).
- `pixi.js/unsafe-eval` stays imported first in `pixi-compositor.ts` (the renderer's CSP has no `unsafe-eval`).
- Error messages never contain media URLs (they carry the session token).
- Shared interpolation vectors (`docs/test-vectors/interpolation.json`) are the contract with a future Rust evaluator: change them only with the code.

## Allowed dependencies
`@se/protocol` (types), `pixi.js`, `mediabunny`. See `docs/architecture.md` §4.1 and §8.

## How to test
- Node unit tests: `pnpm vitest run --project engine` (`src/**/*.test.ts`: interpolation vectors, evaluate, layout, frame cache, decode queue, decoder pool with fake sources incl. a 3,000-request scrub leak test, frame renderer and engine with a recording compositor).
- Golden frames: `pnpm vitest run --project engine-browser` (`test/**/*.browser.test.ts`, headless Chromium through Playwright, config `vitest.browser.config.ts`). Needs Chromium once: `pnpm exec playwright install chromium`. WebGL is forced onto SwiftShader and test decoders prefer software decoding so references match on every OS.
  - `frame-accuracy.browser.test.ts`: reads back the frame number encoded in the fixtures (29.97 fps H.264 with B-frames, 25 fps VP9, proxy, rotated phone clip, offsets and speeds).
  - `scenes.browser.test.ts`: compares scenes with `test/golden/references/<name>.png` (`expectGolden`, tolerance 16 levels / 0.5 % of pixels); a mismatch writes `<name>.actual.png` and `<name>.diff.png` next to it (gitignored).
  - `decoder-leaks.browser.test.ts`: intensive scrub with cancellations on real WebCodecs, zero open frames after destroy.
  - Rewrite references after an intended visual change: `SE_UPDATE_GOLDEN=1 pnpm vitest run --project engine-browser`, then **look at every changed PNG** before committing.
  - Fixtures are committed; regenerate them with `node scripts/generate-golden-fixtures.mjs` (needs `pnpm fetch-binaries`), then rewrite the references.
- Browser-mode file commands (`commands.readFile/writeFile`) resolve paths from the package root (`./test/golden/…`), not from the test file.
