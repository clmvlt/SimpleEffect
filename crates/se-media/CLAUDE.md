# se-media

## Purpose
Media adapter: probing (`ffprobe`), fingerprints, thumbnails, waveform peaks, mezzanines, proxies, audio analyses, scene detection and frame extraction (`ffmpeg`), run as sidecar processes (ADR-0007, ADR-0018, ADR-0019). It implements the `MediaProber`, `MediaFiles`, `MediaJobRunner`, `FrameExtractor` and `MediaTools` ports of se-app; scheduling, priorities and retries belong to se-app's `JobQueue`, and deciding *what* to generate (mezzanine policy, recipes) to its `MediaService` and `media_policy`. This crate turns a `TranscodePlan` into ffmpeg arguments and picks the encoder.

## Public API
| Item | Role |
|---|---|
| `FfprobeProber::new(ToolLocator)` | `MediaProber`: `ffprobe -v error -print_format json -show_format -show_streams` → `ProbedMedia { kind, info }` |
| `parse_probe_output(json)` | The pure parser behind it (`probe/media_info.rs`): kind (image demuxers `*_pipe`/`image2`/`gif` → image; a picture stream that is not cover art → video; sound only → audio; `tty`/`ansi` refused), codec, profile, level, tag, pixel format, bit depth, chroma, size, frame rate (average rate if representable in flicks, else the base rate when within 0.2 %, else `None`), clockwise rotation (display matrix, else legacy `rotate` tag) |
| `FsMediaFiles` | `MediaFiles`: `is_file`, `fingerprint`, `directory_size`, `remove_directory` (going on past files in use) — blocking work on the blocking pool |
| `fingerprint::fingerprint_file(path)` | Size + modification time (Unix ms) + xxh3-64 of the first and last MiB (`SAMPLE_BYTES`) |
| `FfmpegTools::new(ToolLocator)` | `MediaTools`: `set_tools_dir` (updates the shared locator), `status()` (paths, version, hardware encoders); `encoder_support(ffmpeg)` (cached detection), `warm_up()` (detection in the background at startup). Cheap to clone |
| `FfmpegJobRunner::new(FfmpegTools)` | `MediaJobRunner`: `outputs_ready(kind, dir)` (cache kinds only, `false` for others), `run(spec, progress)` → `MediaJobOutput` for every `MediaTask` (`Thumbnails`, `WaveformPeaks`, `Transcode(plan)`, `AudioAnalysis`, `SceneDetection`); `FrameExtractor`: `extract_frame(request)` |
| `ToolLocator` / `Tool` | Finds `ffmpeg`/`ffprobe` on every use: the user's folder (library setting `media.toolsDir`, shared by every clone, changed with `set_override_dir`) → the executable's folder (packaged `resources/bin/`) → `<ancestor>/resources/bin/<platform>/` (development) → `PATH`; `MediaError::ToolsMissing` naming where it looked. `ToolLocator::new(override, exe_dir, search_path)` for tests |
| `encoders` | `HardwareEncoder` (`Nvenc`, `VideoToolbox`, `Vaapi`), `EncoderSupport { h264, hevc_main10 }`, `EncoderCache` (one detection per ffmpeg path: a 5-frame test encode per candidate of `platform::HARDWARE_ENCODER_CANDIDATES`) |
| `transcode_args` | Pure: `video_encoder_args(encoding, hardware)` (software CRF / NVENC `-cq` / VideoToolbox `-q:v` / VAAPI `-qp` + upload filter), `scale_filter(short_side)`, `transcode_args(spec, plan, hardware, output)` |
| `analysis` | Pure: `AudioAnalyzer` (one pass: `LoudnessMeter` over `ebur128` + `SilenceDetector`), `parse_scene_changes(stderr)` (`showinfo` lines), `frame_time`, `amplitude_db` |
| `scene_filter(threshold)` | The scene detection filter graph |
| `process::Sidecar` | The only way to spawn a tool (see Rules) |
| `cache_layout` | File names of a cache folder, `outputs_ready`, `write_atomically` (temporary name + rename) |
| `peaks` | `PEAK_LEVELS` (256, 1024, 4096), `PeakAccumulator`, `Peaks { coarsen, encode, decode }`, `all_levels` |
| `thumbnail_plan::ThumbnailPlan` | Pure sizing/sampling of a thumbnails job |
| `PLATFORM_DIR` | `win32-x64`, `darwin-arm64`, `darwin-x64`, `linux-x64` |

## Cache folder (`<project>/cache/media/<cache key>/`)
The folder is chosen by se-app (`media_paths::media_cache_dir`, key = `ContentFingerprint::cache_key`); this crate owns its files. Every file is written under `<name>.part` and renamed, so a file that exists is complete.

| File | Content |
|---|---|
| `thumbnails.jpg` | The sheet: tiles of 90 px high (width from the display aspect, even), `columns` per row (≤ 8), row-major; the last row may be incomplete |
| `poster.jpg` | One picture up to 360 px high (never enlarged), at 10 % of the duration |
| `thumbnails.json` | `se_protocol::ThumbnailSheet { tileWidth, tileHeight, columns, rows, times: [Flicks], posterWidth, posterHeight }` — written last: its presence marks the job complete |
| `peaks-256.sepk`, `peaks-1024.sepk`, `peaks-4096.sepk` | Waveform peaks, one file per level (`SEPK` below); written coarsest first |
| `mezzanine.mp4` / `.m4a` / `.png` | The mezzanine (name chosen by se-app, recorded in the document); written as `mezzanine.part.<ext>` then renamed |
| `proxy-540.mp4`, `proxy-720.mp4` | The proxy at the resolution it was made with |
| `frames/<flicks>-<width>.jpg` | Extracted frames (`FrameExtractor`), written as `<name>.part.jpg` then renamed |

### `SEPK` format (version 1)
All integers little-endian.

| Offset | Size | Field |
|---|---|---|
| 0 | 4 | magic `SEPK` |
| 4 | 2 | format version (`u16`) = 1 |
| 6 | 2 | channels (`u16`, 1 or 2) |
| 8 | 4 | sample rate of the source (`u32`) |
| 12 | 4 | samples per peak (`u32`: 256, 1024 or 4096) |
| 16 | 4 | peak count per channel (`u32`) |
| 20 | 12 | reserved (zero) |
| 32 | 4 × count × channels | for each block, for each channel: min (`i16`), max (`i16`); ±32767 = full scale |

The body starts at a 2-byte boundary so the renderer maps it onto an `Int16Array` (`@se/protocol` `decodeWaveformPeaks`). A coarser level merges 4 blocks of the finer one (the last block may cover fewer samples).

## Jobs
- **Thumbnails** (`jobs/thumbnails.rs`): three ffmpeg runs in `thumbnails.tmp/` (removed at the end): (1) tiles `tile-%03d.jpg` with `select='isnan(prev_selected_t)+gte(t-prev_selected_t,INTERVAL)'`, `scale=W:90`, `metadata=mode=add` + `metadata=mode=print:file=tiles.txt` (the real source time of each tile), `-skip_frame nokey` beyond 30 s, progress from `-progress pipe:1`; (2) the sheet with the `tile` filter; (3) the poster with an input seek. At most 40 tiles, one per whole second for shorter clips; images get one tile. When the times file does not match the tiles, evenly spaced times are written.
- **Waveform peaks** (`jobs/waveform.rs`): `ffmpeg -i IN -map 0:a:0 -ac ≤2 -c:a pcm_f32le -f f32le pipe:1` (`jobs/pcm.rs`: `PcmFormat`, `spawn_decoder`, `PcmReader`), accumulated while reading (finest level), then coarsened; progress = samples read / expected samples (stdout carries the PCM).
- **Mezzanine / proxy** (`jobs/transcode.rs`, arguments in `jobs/transcode_args.rs`): one run with `-noautorotate` (coded size and display matrix kept), `-progress pipe:1`, the plan's maps (`0:v:0`, `0:a:0`), codecs and `-movflags +faststart`, into `<stem>.part.<ext>`, renamed when done. The hardware encoder of the target codec is tried first; on failure the job runs again in software (libx264 `-preset fast`, libx265 `-preset fast` with `log-level=error`).
- **Audio analysis** (`jobs/audio_analysis.rs`): the PCM decoder of the peaks; chunks go through a channel to `AudioAnalyzer` on the blocking pool (EBU R128 integrated, range, true and sample peaks, short-term every 0.5 s; silences of 20 ms RMS windows). A frame split across two chunks is completed by the next one.
- **Scene detection** (`jobs/scene_detection.rs`): `-v info -filter_complex "[0:v:0]scale=320:-2,split[scenes][all];[scenes]select='gt(scene,T)',showinfo,nullsink;[all]null[out]" -map [out] -f null -`; `showinfo` lines are read from stderr (`Sidecar::run_with_lines_keeping_stderr`), `-progress` from stdout.
- **Frames** (`frames.rs`): `-ss T -i IN -map 0:v:0 -frames:v 1 -vf scale='min(W,iw)':-2 -q:v 3` (upright: autorotate stays on); past the last frame, `-sseof -1 … -update 1` keeps the last one.

## Rules and invariants
- Sidecar processes only: never link FFmpeg libraries.
- **Every process goes through `process::Sidecar`**: no stdin, stdout and stderr piped (stderr collected for error messages), Windows job object + `CREATE_NO_WINDOW` / Unix process group (`platform.rs`). Dropping a `Sidecar` that has not exited kills the whole tree: a job is cancelled by dropping its future, so never detach a process from its `Sidecar`.
- Outputs appear atomically (`write_atomically`, renames); a cancelled job leaves no file that looks complete. `outputs_ready` lists the files that must exist (`completion_files`).
- OS specifics live in `platform.rs` only (executable suffix, process isolation, platform folder name, hardware encoder candidates, VA-API device).
- Hardware encoders are trusted only after a test encode with the job's own arguments (`encoders.rs`); every hardware run has a software fallback.
- Derived copies keep the original's coded geometry and rotation; frames for viewers (extracted JPEGs) are upright.
- Binaries come from `resources/bin/<platform>/` (`pnpm fetch-binaries`, pinned in `scripts/binaries.manifest.json`), never committed. Paths given to ffmpeg filters are relative to the run's working folder (`metadata` file, tile pattern) so no Windows path ever needs filter escaping.
- Error messages quote the file and ffmpeg's last stderr lines; they reach users and AI agents.

## Allowed dependencies
`se-app`, `se-domain`, `se-protocol` (DTOs: `JobKind`, `ThumbnailSheet`, `MediaToolsStatus`, `DerivedKind`), `tokio` (process, fs, io), `process-wrap`, `serde`/`serde_json`, `xxhash-rust`, `which`, `ebur128` (loudness), `windows` (creation flags, Windows only). Never another adapter crate. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-media`
- Unit tests in-file: ffprobe mapping (rotation, frame rates, pixel formats, refusals), peaks (accumulation, coarsening, `SEPK` round trip), thumbnail plans, progress lines, fingerprints, cache layout, tool search order, transcode arguments, silence detection and EBU R128 loudness on synthetic sines (a −20 dBFS stereo 1 kHz sine is −20 LUFS), `showinfo` parsing (`tests/fixtures/showinfo/scenes.txt`, recorded with ffmpeg 9.0.2 on red → blue → `testsrc2` shots cut at 2 s and 4 s), folder size and removal.
- `tests/probe_fixtures.rs`: recorded ffprobe output of files generated by ffmpeg 9.0.2 (`tests/fixtures/ffprobe/`: H.264 + AAC, HEVC Main 10 rotated, ProRes 422 HQ + PCM, WAV, PNG). Re-record with `ffprobe -v error -print_format json -show_format -show_streams` and strip `format.filename` to the base name.
- `tests/ffmpeg/` (one test crate: `main.rs`, `support.rs`): real ffmpeg on media generated with `testsrc2`/`color`/`sine` — `previews.rs` (thumbnails + peaks of a video, an image, a text file refused, cancelling a 4K job by dropping it), `derived.rs` (ProRes 422 HQ → HEVC Main 10 mezzanine with AAC, probed; a rotated proxy at 960×540 keeping its rotation; frames at 1 s and at the very end), `analyses.rs` (a tone with a 1 s silence: silence at 2–3 s and loudness; cuts at 2 s and 4 s found). Skipped with a message when `resources/bin/<platform>/` has no ffmpeg. On a machine with an NVIDIA GPU the mezzanine test uses NVENC, elsewhere libx265.
