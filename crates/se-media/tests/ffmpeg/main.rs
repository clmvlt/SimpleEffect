//! The adapter against real ffmpeg and ffprobe: media are generated with `testsrc2`, `color` and
//! `sine`. Every test is skipped (with a message) when the binaries are not installed
//! (`pnpm fetch-binaries`).

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

mod analyses;
mod derived;
mod previews;
mod sampling;
mod support;
