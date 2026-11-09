//! # se-media
//!
//! Media adapter: probes files and runs media jobs (thumbnails, waveform peaks, mezzanines,
//! proxies, audio analyses, scene detection) and frame extraction through the `ffprobe` and
//! `ffmpeg` sidecar binaries (`docs/adr/0007-media-pipeline-sidecars-mezzanine.md`, ADR-0018,
//! ADR-0019).
//!
//! It implements the `MediaProber` ([`FfprobeProber`]), `MediaFiles` ([`FsMediaFiles`]),
//! `MediaJobRunner` and `FrameExtractor` ([`FfmpegJobRunner`]) and `MediaTools`
//! ([`FfmpegTools`]) ports of `se-app`. Allowed workspace dependencies:
//! `se-app`, `se-domain`, `se-protocol` (DTOs); never another adapter crate. Sidecars are spawned
//! processes only (one helper: [`process::Sidecar`]); no FFmpeg linking.
//!
//! File formats of the cache folder (thumbnail sheet, `SEPK` peaks) are documented in
//! `crates/se-media/CLAUDE.md`.

pub mod analysis;
pub mod cache_layout;
pub mod encoders;
mod ffmpeg_tools;
mod files;
pub mod fingerprint;
pub mod frame_sampler;
mod frames;
mod jobs;
pub mod peaks;
mod platform;
mod probe;
pub mod process;
pub mod speech_audio;
mod tools;

pub use ffmpeg_tools::FfmpegTools;
pub use files::FsMediaFiles;
pub use jobs::{FfmpegJobRunner, parse_tile_times, scene_filter, thumbnail_plan, transcode_args};
pub use platform::PLATFORM_DIR;
pub use probe::{FfprobeProber, parse_probe_output};
pub use tools::{Tool, ToolLocator};
