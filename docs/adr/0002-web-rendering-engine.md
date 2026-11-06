# 0002 — Web rendering engine shared by preview and export

- Status: Accepted
- Date: 2026-11-06

## Context
The editor needs frame-accurate real-time preview of multiple video/audio tracks with animated text and motion graphics, and a final export that matches the preview exactly. AI agents generate motion design, a domain where LLMs are strongest with 2D web graphics.

## Decision
- The rendering engine `@se/engine` runs in the Electron renderer (TypeScript):
  - `evaluate(sequence, time) -> FrameGraph` is a pure function (keyframes, easing, transforms, z-order).
  - Decoding uses **WebCodecs** through **Mediabunny** (MPL-2.0) with a VideoFrame LRU cache and prefetching.
  - Compositing uses **PixiJS v8** (WebGL2, WebGPU optional) behind a `Compositor` interface.
  - Audio uses the **Web Audio API**; the `AudioContext` clock is the master clock.
- **Export runs the same code** offline in a Web Worker with `OffscreenCanvas`, encodes with `VideoEncoder`/`AudioEncoder` (hardware-accelerated) and muxes with Mediabunny. Outputs that WebCodecs cannot encode (ProRes, DNxHR) send raw frames to ffmpeg through se-core.

## Consequences
- WYSIWYG by construction; one engine to test (golden-frame tests in Chromium).
- Codecs Chromium cannot decode (ProRes, DNxHR, some 10-bit 4:2:2) need mezzanine files (ADR-0007).
- Export can run in the background while the user keeps editing.
- PixiJS is hidden behind an interface so it can be replaced.

## Alternatives considered
- **Native Rust renderer (wgpu + ffmpeg)**: fastest and supports all codecs, but displaying a native surface inside the web UI is fragile and the text/animation stack would be built from scratch. Rejected for v1.
- **Remotion**: built for programmatic video rendered through headless Chrome screenshots; not an interactive real-time NLE; company licence beyond 3 people. Rejected.
- **MLT / GStreamer**: proven NLE engines but heavy C integration and same display problem. Rejected.
- **HTML `<video>` elements as textures**: not frame-accurate for scrubbing and export. Rejected.
