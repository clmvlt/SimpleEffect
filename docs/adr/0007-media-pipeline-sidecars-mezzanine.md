# 0007 — Media pipeline with sidecar binaries, mezzanines and proxies

- Status: Accepted
- Date: 2026-11-06

## Context
The user edits phone/screen footage **and professional camera footage** (ProRes, DNxHR, 10-bit 4:2:2). Chromium's WebCodecs decodes H.264/HEVC/VP9/AV1 (depending on GPU/OS) but not ProRes or DNxHR. AI features need analysis data (waveforms, silences, loudness, scene changes, transcripts, faces).

## Decision
- ffmpeg, ffprobe and whisper.cpp run as **sidecar processes** driven by Rust adapters. Binaries live in `resources/bin/<platform>/` (fetched by `pnpm fetch-binaries` in development, bundled at packaging) with a user override path.
- At import, ffprobe produces metadata and the renderer reports whether WebCodecs can decode the asset on this machine (`isConfigSupported`).
- Non-decodable or heavy assets get, automatically:
  - a **mezzanine**: high-quality, WebCodecs-decodable file used for full-quality preview and export;
  - a **proxy**: low-resolution file for smooth playback.
- Exports to Premiere always reference **original** files.
- Proprietary RAW formats (BRAW, R3D, ARRIRAW) are out of scope initially (vendor SDKs required).
- A `JobQueue` in se-app runs jobs with priorities, per-kind concurrency, progress events and cancellation; outputs are cached by content hash.
- ONNX Runtime (`ort` crate) runs local vision models (face/subject detection) in-process.

## Consequences
- Simple builds on all OSes; a crashing ffmpeg never takes the app down; binaries are replaceable.
- Extra disk usage for mezzanines/proxies (configurable quality, cache cleanup tools).
- Export quality from mezzanines is visually lossless for delivery codecs; a future "master render" path through ffmpeg can use originals for 10-bit deliverables.

## Alternatives considered
- **ffmpeg-next / libav bindings**: complex cross-platform builds, crashes in-process. Rejected.
- **GStreamer**: powerful but heavy to ship and to maintain. Rejected.
- **Decoding originals in Rust and streaming raw frames to the renderer**: very high bandwidth (4K RGBA ≈ 33 MB/frame). Rejected for preview; possible later for master exports.
