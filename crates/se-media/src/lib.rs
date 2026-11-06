//! # se-media
//!
//! Media adapter: probes files and runs media jobs (thumbnails, waveforms, mezzanines, proxies,
//! analyses) through the `ffprobe` and `ffmpeg` sidecar binaries
//! (see `docs/adr/0007-media-pipeline-sidecars-mezzanine.md`).
//!
//! It implements the `MediaProber` and `MediaJobRunner` ports of `se-app`.
//! Allowed workspace dependencies: `se-app`, `se-domain`; never another adapter crate.
//! Sidecars are spawned processes only: no FFmpeg linking.
//!
//! Implemented by roadmap prompts 08 and 09.
