# 0037 — Automatic reframing: YuNet over a runtime-loaded ONNX Runtime, pure tracking, editable keyframes

- Status: Accepted
- Date: 2026-11-09

## Context
Prompt 26 turns a landscape edit into a portrait (9:16) version whose frame follows the subject
(a face, a person) with smooth moves, jumps cleanly at shot changes, and stays editable by hand.
ADR-0026 already gave clips a `tracked` reframe mode evaluated like `fill`, whose **focus** — a
point of the picture, independent of the frame's aspect — is an animatable property: only its
keyframes were missing. Open questions: which detector and inference library (licence, the three
platforms, packaging, GPU), how faces are sampled and stored, where the geometry lives and how it
stays testable without a model, how few keyframes a trajectory can be expressed with, how the
version flow and agents (whose scope never reached other sequences) use it, and how the
"local only, no telemetry" rule holds.

## Decision
- **Detector: YuNet** (`face_detection_yunet_2026may.onnx`, OpenCV Zoo, **MIT**, Shiqi Yu;
  dynamic input size; 229,738 bytes, SHA-256 `ebafce4e…f0f0`, pinned to opencv_zoo commit
  `26cc381e`). Copyleft models (YOLO-face under AGPL, SCRFD weights for non-commercial use) were
  excluded. The model is **embedded in se-vision** (`include_bytes!`, like se-fonts' bundled
  fonts; a test checks its SHA-256) rather than downloaded: 230 kB needs no consent dialog and
  never goes missing (the user's choice; the prompt proposed a consented download). Its decoding
  (scores `sqrt(cls × obj)`, centre/size from the cell, NMS 0.3, threshold 0.6, input padded to
  multiples of 32, BGR 0–255) reproduces OpenCV's `FaceDetectorYN` exactly (checked on a
  public-domain portrait against OpenCV 5.0).
- **Inference: ONNX Runtime 1.31 through `ort` 2.0.0-rc.15 in `load-dynamic` mode.** Nothing is
  downloaded or linked at build time: `pnpm fetch-binaries` installs Microsoft's official
  release builds (pinned by their GitHub SHA-256) into `resources/bin/<platform>/onnxruntime/`
  with their licence and third-party notices, electron-builder packages that folder, and
  se-vision loads the library the first time faces are detected (next to the binary, then
  `<ancestor>/resources/bin/<platform>/`). The environment is created **with telemetry
  disabled** (Microsoft's Windows builds enable ETW telemetry by default) and kept for every
  session. Execution providers: CoreML on macOS (part of Microsoft's macOS build), the CPU
  elsewhere — YuNet takes 2–5 ms per 640-pixel picture on a CPU, ffmpeg's decoding dominates, and
  DirectML builds stopped at 1.24 while CUDA builds weigh 400 MB. Microsoft publishes no Intel
  Mac build: there `getVisionStatus` says why and reframing is refused (`VISION_UNAVAILABLE`).
- **Sampling** (se-media, `FrameSampler` port): ffmpeg decodes the first picture stream with
  `setpts=PTS-STARTPTS,fps=5/1,scale=W:H:flags=area,format=bgr24` (upright: autorotation stays
  on; 5 pictures per second from the first frame, exact times `n × 200 ms`) into raw BGR on
  stdout, streamed through a small channel to the detector in batches while ffmpeg keeps decoding:
  pictures never touch the disk. The longest side is 640 px. A ready proxy is read instead of the
  original (same timing, much faster); hardware decoding (`-hwaccel auto`, software retry) is
  asked only for pictures larger than 1080p without a proxy — measured: on 1080p H.264 software
  decoding was 5× faster (0.5 s vs 2.7 s for 60 s; every decoded frame is copied back before
  `fps` drops it), on 4K HEVC Main 10 hardware gained 12 %.
- **Storage**: faces are an **analysis of the media's content** (`AnalysisKind::SubjectDetection`
  in `project.db`: model, interval, frames with normalised boxes and scores), run on request by
  a `SubjectDetection` job (one slot, `Normal` priority), once per content and model. The asset
  keeps no summary (no document change for it). Shot changes come from prompt 09's scene
  detection, asked ahead of other analyses when missing (`MediaService::request_scene_detection`).
- **Geometry is pure, in the domain** (`se_domain::tracking`, `track_subject`): shots cut at scene
  changes and at **subject cuts** — between two samples where every lasting face (≥ 3 samples)
  stops and only new lasting faces start, placed halfway (at most half an interval off): scene
  detection misses cuts between similar pictures (two cameras on one set scored 0.27 for a
  threshold of 0.3 in the real check), and the frame must not drift from one subject to the
  next — (samples within half an interval of a cut ignored), faces linked into tracks, the main
  subject chosen by persistence × size × centrality, **group framing** decided once per shot when
  the significant faces fit in the window together, the window of the engine's `fillCrop`, the
  focus putting the eyes on the **eye line** (upper third by default), gaps interpolated or held
  (a shot without faces stays centred), a **camera operator's dead zone** (hold while the subject
  moves inside it; follow while it moves; settle where it stops), **two rounds of forward and
  backward One-Euro filtering** (one round turns a step into two exponentials meeting at a
  corner; two give an S-shaped pan and cancel the lag), and a Ramer–Douglas–Peucker
  simplification over **cubic segments** — each keyframe's easing is a Bézier with
  `x1 = 1/3, x2 = 2/3`, whose progress is then a cubic Hermite polynomial of time with end slopes
  from the smoothed path, limited to `[0, 3]` so a segment never overshoots — within 0.8 % of the
  frame. Keyframes sit on the sequence's frames; the last one of a shot is a **hold** and the next
  shot starts on the cut's first frame. Measured on simulated tracks: a seated speaker → one
  static focus, a walk across the frame → ~6 keyframes, a lean in and out → ~7.
- **Document**: `Reframe::Tracked { focus, tracking: TrackingSettings { smoothing, eyeLine,
  groupFraming, deadZone } }` (schema version 11; ranges checked as `VALUE_OUT_OF_RANGE`;
  migration and serde default give old tracked reframes the defaults). The settings let
  « Relancer » recompute with the same choices — instantly, since the faces are stored.
- **Use cases** (se-app `ReframeService`): `reframe_items` reframes at once (one user transaction
  « Auto reframe ») the clips whose media is analysed, queues the analyses of the others and
  reframes them when they end (one more transaction, then `autoReframeFinished`);
  `create_version` = `duplicateSequence` (« Create version ») then `reframe_items` on its video
  clips — two undo steps; `subject_detection` feeds the viewer; `analyses_for` lets agents wait.
- **Agents** (`auto_reframe`, MCP): clips of the scope (`AgentEdit::ReframeClips`) or, with
  `target_format`, a version of the task's sequence placed after it
  (`AgentEdit::AddReframedVersion`: the copy, filling clips and their tracked focus in one AI
  transaction). The scope rule gains **one exception**: a transaction may add a new sequence, and
  everything inside a sequence it creates is new content; no other sequence can be added, changed
  or removed, and reverting the task removes the version.
- **UI**: the inspector's « Cadrage » gets « Recadrer automatiquement » / « Relancer », the
  analysis's progress and the settings (applied at once to tracked clips); « Créer une version… »
  gets « Recadrage automatique (suivi du sujet) », ticked when faces can be detected; the viewer's
  reframing mode draws the focus trajectory, its keyframes and the faces detected under the
  playhead; dragging the picture keeps writing focus keyframes (ADR-0026).

## Consequences
- A 9:16 version of an interview is one dialog: the version appears at once, its clips start
  following their speaker as soon as the analysis ends; everything stays ordinary keyframes.
- se-vision is the only crate holding an inference library; the geometry is tested on simulated
  detections (unit tests per step, property tests: keyframes inside the item, focuses inside the
  picture, no overshoot, a hold before each cut, a valid document) and the real detector on one
  portrait when the library is present (CI fetches it).
- Releases carry ~17 MB (Windows), ~29 MB (Linux) or ~45 MB (macOS) more; Intel Macs cannot
  reframe automatically.
- Active-speaker detection (audio diarisation) stays in the backlog: a two-shot whose faces do not
  fit together follows the most persistent, largest, most central face.
- Analyses cover the whole media even when a clip uses a few seconds of it (one result per
  content, reusable by every clip); partial analyses are in the backlog.

## Alternatives considered
- **`ort` statically linked with pyke's binaries (DirectML on Windows)**: GPU on every Windows
  machine, but a third-party CDN at build time, ~30 MB more in se-core, DirectML.dll to package and
  a heavy link in every test binary — for a model that needs no GPU. Rejected.
- **`tract` (pure Rust)**: no native library at all, but leaves the stack's choice of ONNX Runtime
  and any acceleration. Rejected by the user.
- **A consented download of the model**: what the prompt proposed; a 230 kB MIT file is better
  shipped than downloaded. Rejected by the user.
- **Kalman filter**: needs a motion model and tuning per scene; One-Euro's one-knob behaviour maps
  to « Fluidité », and the forward-backward rounds remove its lag offline.
- **Linear segments** for keyframes: 25–40 % more keyframes for the same tolerance.
- **A dead zone that jumps to the subject**: turned a walk into a staircase of small pans;
  following while the subject moves fixed it.
- **Agents reframing only an existing portrait sequence**: would leave « make a vertical cut » to
  the user; the version exception keeps every existing sequence out of reach.
- **Running the face analysis at import**: costly for media never reframed; it runs on request
  and is kept.
