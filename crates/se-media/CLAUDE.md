# se-media

## Purpose
Media adapter: probing (`ffprobe`), fingerprints, thumbnails and waveform peaks (`ffmpeg`), run as sidecar processes (ADR-0007, ADR-0018). It implements the `MediaProber`, `MediaFiles` and `MediaJobRunner` ports of se-app; scheduling, priorities and retries belong to se-app's `JobQueue`, and deciding *what* to generate to its `MediaService`. Prompt 09 adds mezzanines, proxies and analyses.

## Public API
| Item | Role |
|---|---|
| `FfprobeProber::new(ToolLocator)` | `MediaProber`: `ffprobe -v error -print_format json -show_format -show_streams` → `ProbedMedia { kind, info }` |
| `parse_probe_output(json)` | The pure parser behind it (`probe/media_info.rs`): kind (image demuxers `*_pipe`/`image2`/`gif` → image; a picture stream that is not cover art → video; sound only → audio; `tty`/`ansi` refused), codec, profile, level, tag, pixel format, bit depth, chroma, size, frame rate (average rate if representable in flicks, else the base rate when within 0.2 %, else `None`), clockwise rotation (display matrix, else legacy `rotate` tag) |
| `FsMediaFiles` | `MediaFiles`: `is_file`, `fingerprint` (on the blocking pool) |
| `fingerprint::fingerprint_file(path)` | Size + modification time (Unix ms) + xxh3-64 of the first and last MiB (`SAMPLE_BYTES`) |
| `FfmpegJobRunner::new(ToolLocator)` | `MediaJobRunner`: `outputs_ready(kind, dir)`, `run(spec, progress)` for `Thumbnails` and `WaveformPeaks` |
| `ToolLocator` / `Tool` | Finds `ffmpeg`/`ffprobe` on every use: `override_dir` (library setting `media.toolsDir`) → the executable's folder (packaged `resources/bin/`) → `<ancestor>/resources/bin/<platform>/` (development) → `PATH`; `MediaError::ToolsMissing` naming where it looked |
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
- **Waveform peaks** (`jobs/waveform.rs`): `ffmpeg -i IN -map 0:a:0 -ac ≤2 -c:a pcm_f32le -f f32le pipe:1`, accumulated while reading (finest level), then coarsened; progress = samples read / expected samples (stdout carries the PCM).

## Rules and invariants
- Sidecar processes only: never link FFmpeg libraries.
- **Every process goes through `process::Sidecar`**: no stdin, stdout and stderr piped (stderr collected for error messages), Windows job object + `CREATE_NO_WINDOW` / Unix process group (`platform.rs`). Dropping a `Sidecar` that has not exited kills the whole tree: a job is cancelled by dropping its future, so never detach a process from its `Sidecar`.
- Outputs appear atomically (`write_atomically`, renames); a cancelled job leaves no file that looks complete. `outputs_ready` lists the files that must exist (`completion_files`).
- OS specifics live in `platform.rs` only (executable suffix, process isolation, platform folder name).
- Binaries come from `resources/bin/<platform>/` (`pnpm fetch-binaries`, pinned in `scripts/binaries.manifest.json`), never committed. Paths given to ffmpeg filters are relative to the run's working folder (`metadata` file, tile pattern) so no Windows path ever needs filter escaping.
- Error messages quote the file and ffmpeg's last stderr lines; they reach users and AI agents.

## Allowed dependencies
`se-app`, `se-domain`, `se-protocol` (DTOs: `JobKind`, `ThumbnailSheet`), `tokio` (process, fs, io), `process-wrap`, `serde`/`serde_json`, `xxhash-rust`, `which`, `windows` (creation flags, Windows only). Never another adapter crate. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-media`
- Unit tests in-file: ffprobe mapping (rotation, frame rates, pixel formats, refusals), peaks (accumulation, coarsening, `SEPK` round trip), thumbnail plans, progress lines, fingerprints, cache layout, tool search order.
- `tests/probe_fixtures.rs`: recorded ffprobe output of files generated by ffmpeg 9.0.2 (`tests/fixtures/ffprobe/`: H.264 + AAC, HEVC Main 10 rotated, ProRes 422 HQ + PCM, WAV, PNG). Re-record with `ffprobe -v error -print_format json -show_format -show_streams` and strip `format.filename` to the base name.
- `tests/with_ffmpeg.rs`: real ffmpeg on media generated with `testsrc2`/`sine` (thumbnails + peaks of a video, an image, a text file refused, cancelling a 4K job by dropping it). Skipped with a message when `resources/bin/<platform>/` has no ffmpeg.
