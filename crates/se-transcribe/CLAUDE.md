# se-transcribe

## Purpose
Transcription adapter backed by the `whisper.cpp` command-line sidecar. It implements the `Transcriber` port of `se-app`; it does not edit documents (transcripts become domain data through use cases).

Currently an empty skeleton, implemented by prompt 21.

## Public API
None yet. Planned: a `Transcriber` implementation producing word-level timestamps.

## Rules and invariants
- Runs locally only; models are downloaded only after explicit user approval.
- Timestamps are converted to `Flicks` at this boundary.
- Backends: CUDA, Metal or CPU depending on the platform (`docs/architecture.md` §14).

## Allowed dependencies
`se-app`, `se-domain`, `tokio` (process). Never another adapter crate. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-transcribe`
- Output parsing is tested on recorded whisper.cpp output.
