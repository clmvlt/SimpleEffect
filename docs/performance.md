# Performance and robustness

Budgets SimpleEffect must hold on real projects, how to measure them, and the latest results
(prompt 29). Keep this file in sync with `apps/desktop/src/features/performance/services/budgets.ts`
(the in-app measure checks the same numbers) and with the benchmarks of `crates/se-devtools`.

## Budgets

| Area | Budget | Checked by |
|---|---|---|
| Core round trip (`/rpc`, 95th percentile) | < 20 ms | In-app measure |
| Scrub: playhead moved → frame drawn (95th percentile) | < 100 ms | In-app measure |
| Main-thread frame interval while the viewer works (95th percentile) | < 20 ms (≥ 50 fps) | In-app measure |
| Visible hitches (frame interval > 50 ms) during a measure | 0 | In-app measure |
| Dropped frames, 1080p30 or 4K with proxies, at 1× | 0 | In-app measure |
| Timeline interaction (zoom, scroll) on a 1 h / 500-item project (95th percentile frame) | < 16 ms | `measure-app.mjs --scenario timeline` |
| Commit of one edit on a 500-item project (core side) | < 5 ms | `pnpm bench` (`commit_move`) |
| Opening a project with 200 journal entries to replay | < 1 s | `pnpm bench` (`open_with_200_entries`) |
| Export speed, 1080p H.264 | ≥ 1× real time | `measure-app.mjs --scenario 4k` |
| Unsaved changes after a crash | ≤ 30 s or 200 commits (snapshot bound); nothing committed is lost (journal) | se-app / se-core tests |
| Five AI tasks + the user editing at once | every edit lands, none starves, each task revertible | `crates/se-app/tests/load.rs` |

## Results (dev machine)

Windows 11, Ryzen + RTX 3080, release `se-core`, production renderer bundle, 2026-11-10.

### Viewer and timeline (built app, `scripts/perf/measure-app.mjs`)

| Scenario | Core p95 | Scrub p95 | Frame p95 | Hitches | Dropped frames |
|---|---|---|---|---|---|
| 4K/30, H.264 + HEVC 10-bit on V1/V2, originals | 1.5 ms | 504–573 ms ✗ | 13.9 ms | 18 ✗ | 9–60 ✗ |
| Same, proxies (default since prompt 29) | 0.3 ms | 31 ms | < 16 ms | 0 | 0 |
| 1 h timeline, 500 items (generated), 1080p25 | 0.2 ms | 16.9 ms | 7.1 ms | 0 | 0 |

Timeline zoom and scroll on the 1 h project: 420 frames, p95 7.1 ms, worst 41.7 ms, none over 50 ms.
After the runs: 2 open `VideoFrame`s (the decoder pool's cache), JavaScript heap 36–58 MB.

**Before / after.** The main bottleneck was reading 4K originals: each scrub step decodes from the
previous keyframe (GOP of 60 frames, ~570 ms), and playback dropped frames. The app already made
proxies for UHD media but the viewer read them only when « Lire les proxys » was switched on. The
toggle is now on by default (exports always read originals): scrub p95 573 ms → 31 ms, dropped
frames 9 → 0. Reading originals without proxies stays over budget: see « Known gaps ».

### Exports (built app, 30 s 4K/30 project, two video tracks)

| Output | Time | Speed |
|---|---|---|
| 1080p H.264 | 8.1 s | 3.7× real time |
| 2160p H.264 | 9.7 s | 3.1× real time |

### Core (`pnpm bench`, criterion, release)

| Benchmark | 500 items | 2000 items |
|---|---|---|
| `commit_move` (validate + apply one move) | 1.37 ms | 5.78 ms |
| `patch` (patch sent to clients) | 33 µs | — |
| `snapshot` | 25 µs | — |
| `write_project_json` / `read_project_json` | 0.90 ms / 3.1 ms | — |
| `append_journal` (SQLite, `synchronous = FULL`) | 10.5 ms | — |
| `save_snapshot` | 20 ms | — |
| `replay_200` (journal entries) | 273 ms | — |
| `open_with_200_entries` | 295 ms | — |

`append_journal` is dominated by Windows' `fsync`: `synchronous = FULL` is kept on purpose (a
committed edit survives a power cut). It runs on the actor after the patch is broadcast, so it
does not delay what the user sees.

### AI load (`crates/se-app/tests/load.rs`)

Five tasks making 20 edits each while the user moves a title 20 times: every edit lands, the
revision counts every commit, the saved snapshot matches, and each task reverts alone. The test
found a real bug, fixed here: optimistic AI edits raced each other on revisions and failed after
four replans. AI edits now plan and commit one at a time (`AgentEditService` serial lock); replans
only absorb user and system commits.

## Robustness scenarios

| Scenario | Behaviour | Covered by |
|---|---|---|
| `se-core` killed while editing | The supervisor restarts it, the renderer reconnects and reopens the project; the journal replays every commit; a toast says how many changes were recovered | `se-core/tests/core_process/projects.rs` (`a_commit_survives_killing_the_core`), `use-recovery-notice.ts` |
| `project.json` corrupted | Set aside as `project.json.corrupted`, the newest valid `.bak.N` restored, the journal since that backup replayed; a lasting toast says what was restored and lost | `se-store/tests/project_folders.rs`, `se-app/tests/project_service.rs`, `se-core` process test |
| Saving fails (disk full, permissions) | Edits stay in memory and the journal; `projectSaveStateChanged` shows « Enregistrement impossible » with the unsaved count in the status bar until a save succeeds; disk-full errors say so | `se-app/tests/project_actor.rs`, `se-store` `io_errors.rs` |
| Disk full during an export | The export fails with « disque plein », partial files removed | `se-media` `part_file.rs` |
| Missing binaries / models | `FFMPEG_UNAVAILABLE`, `VISION_UNAVAILABLE`, missing model with « Réglages » (earlier prompts) | se-media, se-vision, se-transcribe tests |
| AI CLI crashes or disconnects mid-task | The task fails with its reason; its edits stay and remain revertible | `se-app/tests/task_service.rs` |
| App closed during an export or AI task | Confirmation while exports run; on shutdown running tasks are marked **interrupted** (not cancelled) and exports cancelled with their partial files removed | `se-app/tests/task_service.rs`, `exports.rs` |
| Continuous editing | Snapshots wait at most 30 s or 200 commits, so the journal to replay stays bounded | `se-app/tests/project_actor.rs` |

## Measuring

### In the app

« Réglages → Performances → Mesurer les performances » times 30 core round trips, 20 scrub steps
over the open sequence, 3 s of playback at 1× and the main-thread frames meanwhile, then compares
them with the budgets. Open a project and show its sequence in the viewer first. The window must
be visible: a hidden or occluded window gets no animation frames, so the measure refuses to start
(or stops after a 10 s stalled seek) and says why.

### Core benchmarks

```bash
pnpm bench
```

`cargo bench -p se-devtools --bench core` (criterion; add `-- --quick` for a fast pass). Projects
come from the generator of `pnpm generate-test-project`.

### The built app over CDP

1. Synthetic media: `node scripts/perf/make-media.mjs` (4K/30 H.264 and HEVC 10-bit with the
   pinned ffmpeg); a long timeline: `pnpm generate-test-project --minutes 60`.
2. Build and start the production renderer with an isolated profile (bash):

   ```bash
   cargo build --release -p se-core
   cd apps/desktop && pnpm exec electron-vite build --outDir "$TEMP/se-perf/out"
   echo '{"type":"module","main":"main/index.js"}' > "$TEMP/se-perf/out/package.json"
   SE_CORE_PATH=../../target/release/se-core.exe pnpm exec electron "$TEMP/se-perf/out" --user-data-dir="$TEMP/se-perf/profile" --remote-debugging-port=9223 --disable-features=CalculateNativeWinOcclusion --disable-renderer-backgrounding --disable-background-timer-throttling
   ```

   The three flags keep a covered or background window drawing (Windows otherwise stops its
   frames). Move the window to a secondary screen without activating it so it does not get in
   the way (`SetWindowPos` with `SWP_NOZORDER | SWP_NOACTIVATE`).
3. `node scripts/perf/measure-app.mjs --scenario 4k` (originals, proxies, exports in 1080p and
   2160p) or `--scenario timeline --project "<folder>.seproj"`. Results print as JSON.

### Profiling

- **Renderer:** a Chromium trace over CDP (`Tracing.start` / `Tracing.end`, categories
  `devtools.timeline,v8.execute,disabled-by-default-devtools.timeline.frame`) or a CPU profile
  (`Profiler.start` / `Profiler.stop`), opened in Chrome's Performance panel. The dev build exposes
  `window.simpleEffectPreviewStats()` (open `VideoFrame`s, cached pictures) for leak checks.
- **Core logs:** every RPC command is timed; commands over 250 ms are logged at `info` as
  « slow command » with their kind, all of them at `debug` (`SE_LOG=se_core=debug`).
- **Core tasks:** `RUSTFLAGS="--cfg tokio_unstable" cargo run -p se-core --features tokio-console`,
  then `tokio-console`. Never in shipped builds.

## Known gaps and plans

- **4K originals without proxies** scrub at ~500–570 ms and drop frames at 1×: random access in
  long-GOP 4K is bound by the hardware decoder. Proxies are read by default; a plan is in
  `docs/backlog.md` (draw the nearest keyframe at once, then refine).
- **30-minute memory soak** not automated: the in-app measure reports open `VideoFrame`s and the
  JavaScript heap after each run (stable at 2 frames, 36–58 MB across the runs above); a scripted
  soak is in the backlog.
- **MCP tool latency** follows the RPC path (same use cases); it is not benchmarked separately.
