//! # se-devtools
//!
//! Development tools (never shipped): the large test projects of `pnpm generate-test-project`
//! (`project`, `media`) and the core benchmarks of `pnpm bench` (`benches/core.rs`, prompt 29),
//! which build their documents with the same generator.

pub mod media;
pub mod project;

use std::{io, path::Path};

/// Why the tool stopped.
#[derive(Debug, thiserror::Error)]
pub enum DevError {
    /// Bad command-line arguments.
    #[error(
        "{0}\\nusage: generate-test-project [--out <folder>] [--items <count>] [--name <name>] [--seed <n>] [--minutes <m>]"
    )]
    Usage(String),
    /// A file operation failed.
    #[error("{path}: {message}")]
    Io {
        /// The file or folder.
        path: String,
        /// The OS error.
        message: String,
    },
    /// ffmpeg could not make a test file.
    #[error("ffmpeg could not generate {file}; run `pnpm fetch-binaries` and try again")]
    Ffmpeg {
        /// The file name.
        file: String,
    },
    /// A media or storage port refused something.
    #[error("{0}")]
    Invalid(String),
}

impl DevError {
    /// An IO failure on `path`.
    #[must_use]
    pub fn io(path: &Path, error: &io::Error) -> Self {
        Self::Io {
            path: path.display().to_string(),
            message: error.to_string(),
        }
    }
}
