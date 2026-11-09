# se-vision

## Purpose
Vision adapter (prompt 26, ADR-0037): finds faces in sampled pictures with the **YuNet** model over **ONNX Runtime**, for automatic reframing. It implements se-app's `SubjectDetector` port; turning faces into reframing keyframes is the domain's job (`se_domain::tracking`), sampling the pictures se-media's (`FrameSampler`).

## Public API
| Item | Role |
|---|---|
| `YuNetDetector::new(locator, threads)` | `SubjectDetector`: `model()` = `yunet-2026may`, `status()` → `VisionStatus` (loads the runtime and the session the first time; `reason` when unavailable), `detect(frames)` → the faces of each frame, most confident first, boxes normalised in the frame (one session reused behind a mutex, on the blocking pool, one picture at a time). Cheap to clone |
| `RuntimeLocator::{new, for_current_exe, candidates, locate}` | Where the ONNX Runtime library is looked for: `<exe dir>/onnxruntime/<lib>` (packaged `resources/bin/`), then `<ancestor>/resources/bin/<platform>/onnxruntime/<lib>` (development); the error names the places searched, or why the system has no build |
| `LoadedRuntime { environment, description, path }` | The process-wide runtime: loaded once by `runtime::load` (`ort::init_from` + `with_telemetry(false)`; a failure is retried next time), its `Environment` kept for every session |
| `yunet` | Pure: `MODEL_BYTES` (embedded), `MODEL_NAME`, `MODEL_SHA256`, `STRIDES`, `SCORE_THRESHOLD` (0.6), `NMS_THRESHOLD` (0.3), `MAX_FACES` (20), `padded`, `input_tensor(frame)` (BGR planes 0–255, zero-padded right and below to multiples of 32), `decode(outputs, padded_width, width, height)` (scores `sqrt(cls × obj)`, boxes from the cell + offsets, `exp` sizes, NMS, normalised, clamped, rounded to 4 decimals) |
| `platform` | `PLATFORM_DIR`, `LIBRARY_FILE` (`onnxruntime.dll`, `libonnxruntime.dylib`, `libonnxruntime.so`), `unsupported_system()` (Intel Macs), `execution_providers()` (CoreML on macOS, none elsewhere), `accelerator()` |

## Model
- `models/face_detection_yunet_2026may.onnx`: YuNet of the OpenCV Zoo (commit `26cc381e4d2594bb9f47a26eb8fd96c94a13660d`), **MIT** (`models/LICENSE-yunet`, Shiqi Yu), 229,738 bytes, SHA-256 `ebafce4e3c118d6554634be5c27ab333b4c047a9a8c3faf1d7cf93101c22f0f0` (checked by a test), dynamic `height`/`width`. Input `input` `[1, 3, H, W]` float BGR 0–255 (H, W multiples of 32); outputs `cls_8/16/32`, `obj_*` `[1, cells, 1]`, `bbox_*` `[1, cells, 4]`, `kps_*` (unused). Detects faces of about 10–300 px: pictures are sampled 640 px on their longest side.
- Embedded with `include_bytes!` (like se-fonts' fonts): nothing to download, nothing to install. Never replace it with a copyleft model (AGPL YOLO-face, non-commercial SCRFD weights).
- To change it: replace the file and `MODEL_NAME` (stored with results: analyses of another model run again), `MODEL_SHA256`, the decoding if the outputs differ, the expected box of `tests/detection.rs`, and ADR-0037.

## ONNX Runtime
- `ort` 2.0.0-rc.15, features `load-dynamic`, `api-31`, `std`, `tracing` (+ `coreml` on macOS); nothing is downloaded or linked at build time.
- The library comes from `pnpm fetch-binaries`: Microsoft's official **1.31.0** release builds (`scripts/binaries.manifest.json`, SHA-256 pinned), into `resources/bin/<platform>/onnxruntime/` with `LICENSE` and `ThirdPartyNotices.txt`; electron-builder packages the folder next to se-core (`resources/bin/onnxruntime/`). Intel Macs have no build: `status()` says why and se-app refuses with `VISION_UNAVAILABLE`.
- To upgrade: bump `ort` (its README names the ONNX Runtime it targets) and the manifest's three downloads together; `ort` refuses an older library at load time.

## Rules and invariants
- **Telemetry stays off**: the environment is built with `with_telemetry(false)` and every session uses it (`Session::builder(&environment)`); never create a session from another environment.
- No ort call before `runtime::load` succeeded (ort would try to load a library on its own and panic).
- Inference is CPU heavy: only on the blocking pool (`spawn_blocking`); one session, one picture at a time.
- No network, no files: the model is in the binary, pictures arrive in memory and faces leave as values.
- OS specifics live in `platform.rs` only.

## Allowed dependencies
`se-app`, `se-domain`, `se-protocol` (`VisionStatus`), `ort`, `tokio`, `async-trait`, `tracing`. Dev: `jpeg-decoder`, `sha2`, `hex`. Never another adapter crate. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-vision`
- Unit tests in-file: the embedded model's SHA-256, padding, the BGR tensor, decoding of cells as OpenCV does, NMS and low scores, normalisation and clamping, the locator's order and its message, the runtime's description from its build information (the release version, else the build's commit: Microsoft's builds say `git-branch=HEAD`).
- `tests/detection.rs`: the real detector on `tests/fixtures/portrait.jpg` (NASA's 1969 portrait of Neil Armstrong, public domain, 480 × 600) — one face within 0.006 of OpenCV 5.0's `FaceDetectorYN` box `[257.01, 115.40, 76.97, 101.56]` px, score 0.9414; a grey picture has none; a malformed frame is refused with its size. Skipped with a message when the library is absent (CI fetches it).
- Reference decoding outside Rust: OpenCV's `cv2.FaceDetectorYN.create(model, "", (w, h), 0.6, 0.3, 5000)` on the same picture.
