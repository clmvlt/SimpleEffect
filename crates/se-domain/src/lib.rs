//! # se-domain
//!
//! The pure document model of SimpleEffect: time in flicks, identifiers, projects,
//! sequences, tracks, items, keyframes and their validation.
//!
//! This crate is the innermost layer of the architecture (see `docs/architecture.md` §3 and §5):
//! every other crate may depend on it, and it depends on no workspace crate.
//!
//! ## Rules
//! - Pure: no IO, no async, no clock, no randomness, no process spawning.
//! - Allowed third-party dependencies: `serde`, `thiserror`, `uuid`, `ts-rs` (derive only).

pub mod time;

pub use time::{
    Flicks, FrameRate, RoundingMode, TimeError, TimeRange, TimecodeError, TimecodeErrorReason,
    format_timecode, parse_timecode,
};
