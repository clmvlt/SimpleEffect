//! # se-domain
//!
//! The pure document model of SimpleEffect: time in flicks, identifiers, projects,
//! sequences, tracks, items, keyframes, operations and their validation.
//!
//! This crate is the innermost layer of the architecture (see `docs/architecture.md` §3 and §5):
//! every other crate may depend on it, and it depends on no workspace crate.
//!
//! ## Rules
//! - Pure: no IO, no async, no clock, no randomness, no process spawning.
//! - Allowed third-party dependencies: `serde`, `thiserror`, `uuid`, `ts-rs` (derive only).
//!
//! The model itself is introduced by roadmap prompts 05 (time and document model)
//! and 06 (operations, history and changesets).
