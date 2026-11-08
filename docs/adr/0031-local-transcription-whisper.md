# 0031 — Local transcription with whisper.cpp: builds, consented downloads, word timings, storage

- Status: Accepted
- Date: 2026-11-08

## Context
Prompt 21 transcribes the speech of media locally, word by word, so the transcript panel follows
playback and AI agents can cut by what is said (prompts 22–23 build text-based editing and
captions on it). ADR-0007 already chose whisper.cpp as a sidecar. Open questions: which builds to
ship per platform (GPU included), how models reach the machine without breaking « local only »,
how to get word timings precise enough to cut on, where transcripts live, and how one asset's
words become a sequence's words.

## Decision
- **Builds** (whisper.cpp 1.9.5, `pnpm fetch-binaries`, `resources/bin/<platform>/whisper/`):
  Windows and Linux use the official CPU release builds (ggml backends loaded at runtime, every
  CPU variant shipped); macOS has no official command-line build, so it is **compiled from the
  pinned source tarball** with Metal embedded (cmake, in the script and in `release.yml`).
  The **CUDA build is not in the Windows installer** (685 MB download, ~1.2 GB installed): it is a
  *GPU pack* the user installs from « Réglages → Transcription » (official release asset, pinned
  SHA-256, unpacked with the system's bsdtar into `<app data>/runtimes/whisper-cuda-12.4/`). The
  official CUDA 11.8 build lacks cuBLAS and is unusable; no official Vulkan build exists. Linux
  CUDA and other GPUs: the user may point « Dossier de whisper.cpp » at their own build.
- **Downloads only by explicit consent**: the app downloads nothing by itself. Models (tiny, base,
  small, medium, large-v3-turbo from the official ggml repository on Hugging Face, pinned
  revision), the voice detection model (Silero, 0.9 MB, fetched with the first model) and the GPU
  pack are downloaded only when the user confirms a dialog stating size and source. Each download
  resumes a `.part` file, checks size and SHA-256, then renames. HTTPS goes through reqwest +
  rustls (ring provider, system verifier) in se-transcribe, the only crate reaching the network.
  Imported `.bin` files are recognised as catalogue models by SHA-256, else kept as custom models.
  Golden rule 1 now reads « explicit, user-approved model and GPU-pack downloads ».
- **Model choice**: automatic by default — large-v3-turbo with a GPU, small on 8+ threads, base
  otherwise — or the user's choice. Measured on the dev machine (RTX 3080): 12 min 10 s of French
  speech in 18.4 s with large-v3-turbo + DTW + voice detection (≈ 40× real time).
- **Word timings**: whisper-cli runs with full JSON output, **DTW alignment** (`-dtw <preset>`,
  flash attention off: ~20 % slower) and **voice detection** (`--vad`). whisper.cpp maps segment
  times back to the original audio but not token times, so se-transcribe reads the speech spans
  from whisper's log (`vad_segment_info`), maps token times back piecewise, takes a word's start
  where the alignment left the previous token and its end at its last token, then bounds the word
  by its speech span. Measured against phrase boundaries of synthetic French speech: starts within
  ~30 ms, ends within ~110 ms (raw token times were 300–800 ms off). Custom models (unknown
  alignment heads) fall back to whisper's heuristic times. A failed GPU run is redone on the CPU.
- **Tokens → words** (pure, tested on recorded output): a leading space starts a word,
  punctuation-only tokens join the previous word keeping French spacing (`pensez ?`), opening
  marks start the next, an apostrophe glues the next token (`qu'il`), special tokens are dropped;
  token texts are read as raw bytes (a token may split a UTF-8 character).
- **Storage**: transcripts are facts about content, like analyses (ADR-0019): `project.db` schema
  5 — `transcripts` (asset, content key, language, model, backend, version), `transcript_words`,
  `transcript_segments`, and the FTS5 index `transcript_search` (`unicode61`, accents removed) —,
  replaced as a whole by a new transcription. The asset keeps a `TranscriptSummary` (document
  schema 7, system transaction « Record transcript »); its `transcribedAt` tells the renderer to
  fetch again. Per-project language (default `fr`) and vocabulary (`--prompt`) live in
  `ProjectSettings.transcription`, changed by the undoable `SetTranscriptionSettings`.
- **One source → sequence correspondence**: `se_domain::sequence_transcript` (only sound clips
  count; a word belongs to the clip playing its middle; speed applied, clamped to the clip; one
  asset word at one time once), mirrored in TypeScript (`shared/transcript`) and tested on shared
  vectors (`docs/test-vectors/transcript-mapping.json`). The panel maps locally on every patch
  (instant after a cut); MCP tools and later prompts use the Rust function.
- **Jobs**: `JobKind::Transcription` in the shared `JobQueue` (one at a time), on request or after
  an import when « Transcrire les médias à l'import » is on (an `ImportListener` of the media
  service).

## Consequences
- The Windows installer stays small; GPU transcription on Windows needs one consented download.
  The Visual C++ runtime is required by whisper.cpp's Windows builds (an actionable error if
  missing; bundling it is in the backlog).
- macOS releases need cmake on the runner and compile for a few minutes; the macOS path was not
  run on the dev machine (Windows) for this prompt.
- Word timings depend on voice detection: without it (or with custom models) word bounds are
  looser; text-based editing (prompt 22) should snap cuts to silences.
- Every reqwest client of the process needs the ring provider installed first
  (`se_transcribe::install_crypto_provider`, called at startup and in tests).

## Alternatives considered
- **CUDA in the installer**: simplest, but +650 MB for every Windows user. Rejected (user choice).
- **Building Vulkan/CUDA ourselves in CI**: smaller and vendor-neutral (Vulkan), but needs SDKs in
  CI and on developer machines. Deferred.
- **whisper's heuristic token timestamps**: hundreds of milliseconds off; unusable for cuts.
- **Mapping in Rust only, served to the renderer**: one implementation, but a round trip after
  every edit; the mirror + shared vectors keeps the panel instant.
