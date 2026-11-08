//! Subtitle file port (prompt 23): reading and writing SRT (`.srt`) files, implemented by
//! se-export (`FsSubtitleFiles`).

use std::path::Path;

use async_trait::async_trait;
use se_domain::Flicks;

/// One subtitle of a file: when it shows (sequence time) and its text (lines separated by `\n`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubtitleCue {
    /// When it appears.
    pub start: Flicks,
    /// When it disappears (after `start`).
    pub end: Flicks,
    /// Its text, lines separated by `\n`, without markup.
    pub text: String,
}

/// Why a subtitle file could not be read or written. Messages name the file and the fix.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SubtitleError {
    /// The file could not be read.
    #[error("cannot read the subtitle file {path}: {reason}")]
    Read {
        /// The file.
        path: String,
        /// The system's reason.
        reason: String,
    },
    /// The file could not be written.
    #[error("cannot write the subtitle file {path}: {reason}; choose another folder")]
    Write {
        /// The file.
        path: String,
        /// The system's reason.
        reason: String,
    },
    /// The file is not SRT.
    #[error("{path} is not an SRT (.srt) file: {reason}")]
    Invalid {
        /// The file.
        path: String,
        /// What is wrong, with the line number.
        reason: String,
    },
}

/// Reads and writes SRT files.
#[async_trait]
pub trait SubtitleFiles: Send + Sync {
    /// Writes `cues` to `path` (replaced atomically when it exists).
    async fn write_srt(&self, path: &Path, cues: &[SubtitleCue]) -> Result<(), SubtitleError>;

    /// Reads the cues of `path`, in file order.
    async fn read_srt(&self, path: &Path) -> Result<Vec<SubtitleCue>, SubtitleError>;
}
