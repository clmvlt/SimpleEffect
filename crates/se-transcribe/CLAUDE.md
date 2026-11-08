# se-transcribe

## Purpose
Transcription adapter (prompt 21, ADR-0007, ADR-0031): runs the `whisper.cpp` command-line program (`whisper-cli`) as a sidecar and turns its output into word-level `se_domain::Transcript`s; manages the models folder; downloads the files the user agreed to (models, voice model, the Windows GPU pack) over HTTPS — **the only crate of the app that reaches the network**. It implements se-app's `Transcriber`, `FileDownloader` and `ModelFiles` ports; it never edits documents (the transcription service stores transcripts and records summaries).

## Public API
| Item | Role |
|---|---|
| `WhisperTranscriber::new(locator, runtimes_dir, downloader, threads)` | `Transcriber`: `set_whisper_dir` (user folder), `environment()` (program found or why not, GPU name asked once — `nvidia-smi`, or the Apple chip —, expected backend, GPU pack offer and whether installed), `transcribe(request, progress)` (creates the run folder, runs whisper-cli, a failed GPU run is redone with `-ng`, reads the JSON, removes the folder), `install_gpu_runtime` / `remove_gpu_runtime`; `gpu_pack_dir(runtimes_dir)` |
| `WhisperLocator` | Finds `whisper-cli` on every run: user folder → GPU pack (when installed and the GPU is allowed) → `whisper/` next to the executable (packaged) → `<ancestor>/resources/bin/<platform>/whisper/` (development) → `PATH`; `TranscriberError::WhisperMissing` naming where it looked |
| `HttpDownloader` | `FileDownloader`: reqwest + rustls (ring provider, system verifier), resumes `<file>.part` with `Range`, checks size and SHA-256, renames; a mismatch deletes the part |
| `install_crypto_provider()` | Installs rustls's ring provider as the process default (idempotent). Every reqwest client of the workspace needs it (rustls without a built-in provider): se-core calls it at startup, tests building clients call it first |
| `FsModelFiles` | `ModelFiles`: lists complete `.bin` files, SHA-256, `is_whisper_model` (ggml magic `lmgg` + a vocabulary ≥ 50,000 — the voice model is not a transcription model), copies through `.part`, removes |
| `whisper_args::arguments(request, use_gpu, threads)` | Pure: `-m -f -l (lang|auto) -oj -ojf -of <work>/transcript -pp -t N`, `-dtw <preset> -nfa` for catalogue models, `--vad -vm <model>`, `--prompt` (a few hesitations in the transcript's language when `keep_hesitations` — whisper imitates the prompt, so it writes « euh » instead of cleaning it away, prompt 22 — then the vocabulary), `-ng` |
| `whisper_json::parse` | Pure: whisper-cli's full JSON → segments and tokens (bytes, heuristic times, probability, DTW time in ms) |
| `whisper_log::{progress, backend, backend_of}` | Pure: progress lines (`progress = 42%`) and the backend (`whisper_backend_init_gpu: using CUDA0 backend`) |
| `voice_spans::{parse_voice_spans, to_original, within_speech}` | Pure: the speech spans of voice detection (`vad_segment_info` log lines), processed → original time (piecewise linear), bounding a word by its span |
| `words::build_transcript(output, spans, language, model)` | Pure: tokens → words → segments (rules below), times in flicks, sorted and non-overlapping |
| `platform` | `PLATFORM_DIR`, `whisper_executable()`, `isolate` (job object + `CREATE_NO_WINDOW` / process group), `missing_library` (Windows `0xC0000135`/`0xC000007B`, Linux loader message) and its advice, `gpu_pack()` (Windows x64: CUDA 12.4 asset, SHA-256, the 17 files kept), `bundled_backend()`, `gpu_query()`, `archive_tool()` (System32 `tar.exe`) |

## Words and timings (ADR-0031)
- Joining: a token starting with a space starts a word; others continue it (`" 20" + "26"`, `" 19" + "," + "99"`); punctuation-only tokens join the previous word keeping their leading space (`pensez ?`), except opening marks (`«`, `(`, `¿`…) which start the next word (`« ` keeps its space); after an apostrophe the next token continues the word even after a space (`qu'il`); special tokens (`[_BEG_]`, `[_TT_…]`, `<|…|>`) are dropped. Token texts are raw bytes (a token may split a UTF-8 character), decoded per word.
- Timing with DTW: a word starts at the DTW time of the previous text token (whisper.cpp places a token's DTW time where the alignment moves to the next token: its end) — the first word of the audio at its segment's first token —, and ends at its last token's DTW time. Without DTW (custom models): whisper's heuristic token times.
- With voice detection, token and DTW times are in *processed* time (speech glued together): mapped back through the spans of the log, then each word is clamped into the span it overlaps most (its end otherwise runs into the following pause). Segment offsets of the JSON are already original time.
- Confidence: mean probability of the word's non-punctuation tokens. Words are then made monotonic (start ≥ previous start, end ≤ next start).

## Rules and invariants
- Every process goes through `process::run_logged` / `first_line` (isolated, no stdin, stdout drained, stderr read line by line); dropping the future kills the tree (job cancellation).
- Network only in `download.rs`, only for a `DownloadSpec` a use case built after the user's consent; addresses and SHA-256 come from se-app's catalogue or `platform::gpu_pack`. Files appear complete or not at all (`.part` + rename; the GPU pack unpacks into `<pack>.part/` then renames).
- OS specifics live in `platform.rs` only. Timestamps become `Flicks` here (`words.rs`).
- whisper.cpp's Windows builds need the Visual C++ runtime: a missing DLL becomes `TranscriberError::CannotStart` with the advice to install it.

## Allowed dependencies
`se-app`, `se-domain`, `se-protocol` (`TranscriptionBackend`), `tokio` (process, fs), `process-wrap`, `which`, `serde`/`serde_json`, `reqwest` (rustls, no provider) + `rustls` (ring), `sha2`, `hex`, `async-trait`, `tracing`, `windows` (creation flags). Never another adapter crate. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-transcribe`
- Unit tests in-file: arguments, JSON with a split UTF-8 character, progress and backend lines, voice spans (parse, mapping, bounding), joining rules (French apostrophes, numbers, prices, opening marks), DTW timing bounded by speech, the locator's order, downloads (a complete `.part` installs without network, a wrong one is deleted), model files, missing libraries.
- `tests/fixtures.rs`: recorded whisper.cpp 1.9.5 output (`tests/fixtures/whisper/`: Windows' Hortense voice reading French with apostrophes, numbers, a price and punctuation — `large-v3-turbo` on CUDA and `base` on the CPU with DTW + voice detection, `base` without either): the exact words of the turbo run, French spacing, words inside speech, the first word at the speech onset. Re-record with the command in the file's doc comment, then replace local paths.
- `tests/whisper.rs`: a real run of the bundled whisper-cli on `tests/fixtures/whisper/bonjour-fr.flac` with `target/test-models/whisper/ggml-base.bin` (+ the voice model) or `SE_WHISPER_TEST_MODELS`; skipped with a message when either is missing (CI).
- Timing accuracy was measured (not a test) on synthetic phrases with known pauses: starts ±30 ms, ends +110 ms with DTW + voice detection (ADR-0031).
