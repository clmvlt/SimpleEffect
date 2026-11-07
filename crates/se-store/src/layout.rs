//! [`ProjectLayout`]: the files of a `.seproj` folder (ADR-0006, `docs/architecture.md` §11).

use std::path::{Path, PathBuf};

/// Extension of project folders.
pub const PROJECT_EXTENSION: &str = "seproj";

/// Number of previous snapshots kept as `project.json.bak.1` (newest) … `.bak.3`.
pub const BACKUP_COUNT: u32 = 3;

/// Paths inside one project folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectLayout {
    root: PathBuf,
}

impl ProjectLayout {
    /// The layout of the folder at `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The project folder.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// `project.json`: the canonical document.
    #[must_use]
    pub fn snapshot(&self) -> PathBuf {
        self.root.join("project.json")
    }

    /// `project.json.tmp`: written then renamed over `project.json`.
    #[must_use]
    pub fn snapshot_temp(&self) -> PathBuf {
        self.root.join("project.json.tmp")
    }

    /// `project.json.bak.<number>`: previous snapshots, 1 being the newest.
    #[must_use]
    pub fn backup(&self, number: u32) -> PathBuf {
        self.root.join(format!("project.json.bak.{number}"))
    }

    /// `project.db`: SQLite (journal, tasks, transcripts, analyses).
    #[must_use]
    pub fn database(&self) -> PathBuf {
        self.root.join("project.db")
    }

    /// `.lock`: held with an OS file lock while the project is open.
    #[must_use]
    pub fn lock(&self) -> PathBuf {
        self.root.join(".lock")
    }

    /// `cache/`: regenerable derived files.
    #[must_use]
    pub fn cache(&self) -> PathBuf {
        self.root.join("cache")
    }

    /// `exports/`: default export destination.
    #[must_use]
    pub fn exports(&self) -> PathBuf {
        self.root.join("exports")
    }
}
