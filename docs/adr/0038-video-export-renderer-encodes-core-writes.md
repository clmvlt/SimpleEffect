# 0038 — Video export: the renderer draws and encodes, the core orchestrates and writes

- Status: Accepted
- Date: 2026-11-10

## Context
Prompt 27 adds the final export: MP4 (H.264, HEVC, AV1), WebM (VP9, AV1), masters (ProRes 422 HQ,
ProRes 4444, DNxHR HQ in `.mov`) and sound-only files (WAV, AAC in `.m4a`), in the background,
cancellable, with progress, a time left and a check of the written file. Golden rule 7 requires
the export to use the preview's code path (`@se/engine`: `evaluate` + `FrameRenderer` + the PixiJS
compositor + the Web Audio graph), so pictures must be drawn in the renderer. Open questions:
who owns the queue, the file and the cancellation; how the renderer reaches a sequence that may
change (or close) during a long export; how frames leave the renderer (WebCodecs bitstream or raw
pixels); how sound is mixed exactly like playback without a real-time clock; which codecs need
ffmpeg; what happens when the window closes, the machine sleeps or an encoder is missing.

## Decision
- **The core orchestrates, the renderer renders.** `startExport` (se-app `ExportService`) checks
  the settings (container ↔ codecs table, even sizes 16–7680 px, at most 7680 × 4320, bitrates
  100 kb/s–500 Mb/s for pictures and 32–512 kb/s for sound, an absolute path with the
  container's extension), **snapshots** the sequence (`exports::scene`: the project's entities as
  they are, the span, the assets' files), registers the export in memory (`ExportRegistry`,
  `exportUpdated` on every change) and queues one `JobKind::Export` job (one at a time, never
  retried). When it runs, the job opens the output (se-media `FsExportOutputs`) and publishes
  `exportRenderRequested { snapshot, settings, start, end, pipeline }`. The renderer draws that
  snapshot, never its mirror: edits made meanwhile never reach the file. Exports are refused
  without an editor window (`RENDERER_UNAVAILABLE`); the job fails when the window closes, after
  90 s without any upload or progress, or when the project closes (its exports are cancelled).
- **Uploads, not IPC.** The renderer writes through `PUT /media/exports/{id}/data|audio?position=N`
  (bytes at positions: muxers patch headers), `…/frames` (one RGBA frame of a master) and
  `…/thumbnail` (a 320 px JPEG for the list, at most 2 MiB); each answers once written, so the
  encoder feels the disk's pace. It reads media from `…/assets/{asset}/original|mezzanine` (the
  snapshot's files, never the proxy). The job owns the output: uploads reach it through an inbox
  (one writer, in order). `reportExportProgress`, `finishExportRender`, `failExportRender` close
  the loop.
- **Three pipelines** (`settings::pipeline_of`): `muxed` — WebCodecs encodes, Mediabunny muxes,
  the bytes are the file; `ffmpegFrames` — ProRes/DNxHR, which WebCodecs cannot encode: the
  renderer sends the sound as a WAV first, then raw RGBA frames that ffmpeg encodes as they
  arrive (full-range RGB converted to limited-range BT.709, tagged); `aacRemux` — AAC when this
  machine's WebCodecs has no AAC encoder (some Linux Chromium builds): the renderer muxes the
  picture alone and sends a WAV; ffmpeg encodes the AAC and joins both at the end (the dialog
  probes `AudioEncoder` and says so). Everything is written under partial names next to the final
  file (`.part`, `.video.part`, `.audio.part`) and renamed once complete; dropping an unfinished
  output (cancel, failure) stops ffmpeg and removes every partial file. ffprobe then reads the
  file: duration, codecs and size different from what was asked become the export's warning.
- **The export Worker** (`@se/engine/export`): a module Worker builds the preview's renderer on an
  `OffscreenCanvas` at the output size (its own decoder pool, the font catalogue, a template
  runtime when the scene has motion templates) and draws every frame in order in **playback
  mode** (sequential readers pointed two seconds ahead, like forward playback; a frame waits for
  its pictures: an export never skips one). Video encoding prefers hardware
  (`hardwareAcceleration: 'prefer-hardware'` when WebCodecs can, software otherwise); quality
  levels are Mediabunny's `Quality` (0.25/0.5/0.75/1), fixed bitrates are passed as such.
  Cancelling terminates the Worker (decoders, GPU context and uploads with it).
- **Sound mixed in blocks offline.** Workers have no `OfflineAudioContext`, so the page mixes:
  `ExportAudioMixer` renders 30 s blocks at 48 kHz stereo through the preview's `AudioGraph`
  (same gain envelopes, fades, cross-fades, mute/solo, pan) from the pieces `planPieces` gives for
  that stretch, each block starting from a short lead-in so pieces running across a boundary join
  on the sample; decoded chunks are shared by every block. The Worker asks for each block and
  encodes it. Tests check that a block mix equals a one-pass mix and lasts exactly the span.
- **The dialog** (desktop `features/export`): built-in presets (YouTube 1080p and 4K in H.264,
  1080p HEVC, Shorts/Reels/TikTok 9:16, square, ProRes 422 HQ master, WAV, AAC `.m4a`; AAC at
  192 kb/s, the highest rate Windows' encoder accepts), saved presets and custom settings. The
  output **keeps the sequence's aspect**: a preset fixes the shorter side; a 9:16 or 1:1 preset on
  a sequence of another shape warns and offers « Créer une version… » (ADR-0026) instead of
  letterboxing or cropping. The range is the whole sequence (to its content end) or its in/out
  span. The destination is a folder and a file name template (`{project}`, `{sequence}`,
  `{preset}`, `{date}`, `{time}`, `{width}`, `{height}`), joined by the main process (path rules
  of the platform; an existing file is replaced only once confirmed). The dialog estimates the
  size and lists warnings: empty range, missing encoder (blocking), missing or preparing media,
  fonts the catalogue lacks, aspect, software encoding, AAC by ffmpeg. Folder, template and saved
  presets are `library.db` settings (`export.*`).
- **While exports run** the main process keeps the computer from suspending the app
  (`powerSaveBlocker('prevent-app-suspension')`) and asks once before the window closes or the
  app quits; each end is announced by a toast and, when the window is not focused, a system
  notification whose click reveals the file.

## Consequences
- Preview and export cannot drift: same evaluation, decoding, compositing, fonts, templates and
  mix. A golden test renders an export and compares its decoded frames with preview stills.
- An export needs an open editor window (the renderer). Drawing and encoding run in the Worker,
  not on the page's timers, which Chromium throttles in hidden windows.
- Masters cost a full RGBA frame per upload (≈ 8 MB at 1080p): loopback HTTP keeps up, and
  ffmpeg encodes while frames arrive, so no intermediate file holds raw pixels.
- Exports live in memory: a restarted core forgets the list (the files stay). Partial files never
  survive a cancellation or a failure.
- Loudness normalisation and a software retry after a hardware encoder failure are not done
  (`docs/backlog.md`).

## Alternatives considered
- **Render in se-core with ffmpeg filters**: a second renderer that would differ from the preview
  (text, captions, templates, blending) — golden rule 7.
- **Raw frames for every codec, encoded by ffmpeg**: simpler, but 8 MB per frame through loopback
  for long H.264 exports and no hardware encoder; WebCodecs keeps those in the GPU.
- **Mux in the core from WebCodecs packets**: duplicates Mediabunny's muxers in Rust; writing
  bytes at positions keeps the core format-agnostic.
- **Real-time mix with an `AudioContext`**: as slow as the sequence and timing-dependent; a
  single `OfflineAudioContext` for the whole span holds everything in memory.
- **Draw from the live mirror**: an edit during the export would change frames halfway.
- **Letterbox or crop to the preset's aspect**: hides a framing decision; reframed versions are
  the editor's tool for that (ADR-0026, ADR-0037).
