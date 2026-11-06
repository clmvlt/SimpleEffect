//! # se-export
//!
//! Export adapter: FCP7 XML and OTIO timelines for Premiere Pro, SRT subtitles, and
//! orchestration of final renders (see `docs/adr/0008-premiere-export.md`).
//!
//! It implements the `Exporter` port of `se-app`. Writers are pure functions from the document
//! model to bytes. Allowed workspace dependencies: `se-app`, `se-domain`; never another adapter crate.
//!
//! Implemented by roadmap prompts 27 and 28.
