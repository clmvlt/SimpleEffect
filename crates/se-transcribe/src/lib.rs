//! # se-transcribe
//!
//! Transcription adapter backed by the `whisper.cpp` command-line program (`whisper-cli`), run as
//! a sidecar (ADR-0007, ADR-0031). It implements se-app's `Transcriber` ([`WhisperTranscriber`]),
//! `FileDownloader` ([`HttpDownloader`], the only network access of the app, used for files the
//! user agreed to download) and `ModelFiles` ([`FsModelFiles`]) ports.
//!
//! Allowed workspace dependencies: `se-app`, `se-domain`, `se-protocol` (DTOs); never another
//! adapter crate.

mod download;
mod gpu_runtime;
mod hashing;
mod locator;
mod model_files;
pub mod platform;
mod process;
mod transcriber;
pub mod voice_spans;
pub mod whisper_args;
pub mod whisper_json;
pub mod whisper_log;
pub mod words;

pub use download::{HttpDownloader, install_crypto_provider, part_path};
pub use locator::{Located, WhisperLocator};
pub use model_files::FsModelFiles;
pub use transcriber::WhisperTranscriber;
