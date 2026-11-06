//! # se-transcribe
//!
//! Transcription adapter backed by the `whisper.cpp` command-line sidecar.
//!
//! It implements the `Transcriber` port of `se-app`.
//! Allowed workspace dependencies: `se-app`, `se-domain`; never another adapter crate.
//!
//! Implemented by roadmap prompt 21.
