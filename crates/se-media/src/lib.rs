//! # se-media
//!
//! Media adapter: probes files and runs media jobs (thumbnails, waveform peaks; mezzanines,
//! proxies and analyses with prompt 09) through the `ffprobe` and `ffmpeg` sidecar binaries
//! (`docs/adr/0007-media-pipeline-sidecars-mezzanine.md`, ADR-0018).
//!
//! It implements the `MediaProber` ([`FfprobeProber`]), `MediaFiles` ([`FsMediaFiles`]) and
//! `MediaJobRunner` ([`FfmpegJobRunner`]) ports of `se-app`. Allowed workspace dependencies:
//! `se-app`, `se-domain`, `se-protocol` (DTOs); never another adapter crate. Sidecars are spawned
//! processes only (one helper: [`process::Sidecar`]); no FFmpeg linking.
//!
//! File formats of the cache folder (thumbnail sheet, `SEPK` peaks) are documented in
//! `crates/se-media/CLAUDE.md`.

pub mod cache_layout;
mod files;
pub mod fingerprint;
mod jobs;
pub mod peaks;
mod platform;
mod probe;
pub mod process;
mod tools;

pub use files::FsMediaFiles;
pub use jobs::{FfmpegJobRunner, parse_tile_times, thumbnail_plan};
pub use platform::PLATFORM_DIR;
pub use probe::{FfprobeProber, parse_probe_output};
pub use tools::{Tool, ToolLocator};
