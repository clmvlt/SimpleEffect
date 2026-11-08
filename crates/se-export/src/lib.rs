//! # se-export
//!
//! Export adapter: SRT subtitles (prompt 23), later FCP7 XML and OTIO timelines for Premiere
//! Pro and the orchestration of final renders (see `docs/adr/0008-premiere-export.md`).
//!
//! Writers are pure functions from data to text ([`write_srt`]); [`FsSubtitleFiles`] implements
//! se-app's `SubtitleFiles` port on the file system. Allowed workspace dependencies: `se-app`,
//! `se-domain`; never another adapter crate.
//!
//! TODO(prompt-28): FCP7 XML and OTIO writers (they reuse the SRT writer).

mod srt;
mod subtitle_files;

pub use srt::{SrtError, parse_srt, write_srt};
pub use subtitle_files::FsSubtitleFiles;
