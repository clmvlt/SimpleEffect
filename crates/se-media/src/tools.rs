//! [`ToolLocator`]: finds `ffmpeg` and `ffprobe` (`docs/architecture.md` §7).
//!
//! Order: the folder configured by the user → the folder of the running executable (packaged app:
//! `resources/bin/`, next to se-core) → `resources/bin/<platform>/` in an ancestor of that folder
//! (development: the repository, filled by `pnpm fetch-binaries`) → the `PATH`.

use std::{
    path::{Path, PathBuf},
    sync::{Arc, PoisonError, RwLock},
};

use se_app::ports::MediaError;

use crate::platform::{PLATFORM_DIR, executable_name};

/// A sidecar tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    /// Transcoding, thumbnails, PCM extraction.
    Ffmpeg,
    /// Stream metadata.
    Ffprobe,
}

impl Tool {
    /// Its executable name without extension.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Ffmpeg => "ffmpeg",
            Self::Ffprobe => "ffprobe",
        }
    }
}

/// Where to look for the tools. Looked up on every use, so binaries fetched while the app runs are
/// found without a restart. Clones share the user's folder: changing it in the settings affects
/// every component holding a clone.
#[derive(Debug, Clone, Default)]
pub struct ToolLocator {
    /// Folder chosen by the user (setting `media.toolsDir`), searched first.
    override_dir: Arc<RwLock<Option<PathBuf>>>,
    /// Folder of the running executable.
    exe_dir: Option<PathBuf>,
    /// Whether to search the `PATH` last (off in tests).
    search_path: bool,
}

impl ToolLocator {
    /// A locator searching `override_dir`, then `exe_dir` and its ancestors' `resources/bin`, then
    /// the `PATH` when `search_path`.
    #[must_use]
    pub fn new(override_dir: Option<PathBuf>, exe_dir: Option<PathBuf>, search_path: bool) -> Self {
        Self {
            override_dir: Arc::new(RwLock::new(override_dir)),
            exe_dir,
            search_path,
        }
    }

    /// The locator of the running process: the user's folder, the executable's folder, the
    /// `PATH`.
    #[must_use]
    pub fn for_current_exe(override_dir: Option<PathBuf>) -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf));
        Self::new(override_dir, exe_dir, true)
    }

    /// The folder chosen by the user, if any.
    #[must_use]
    pub fn override_dir(&self) -> Option<PathBuf> {
        // A poisoned lock still holds a valid path.
        self.override_dir
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Searches `dir` first from now on (every clone), or no user folder when `None`.
    pub fn set_override_dir(&self, dir: Option<PathBuf>) {
        *self
            .override_dir
            .write()
            .unwrap_or_else(PoisonError::into_inner) = dir;
    }

    /// The candidate files for `tool`, in search order (the `PATH` excluded).
    #[must_use]
    pub fn candidates(&self, tool: Tool) -> Vec<PathBuf> {
        let file = executable_name(tool.name());
        let mut candidates = Vec::new();
        if let Some(dir) = self.override_dir() {
            candidates.push(dir.join(&file));
        }
        if let Some(dir) = &self.exe_dir {
            candidates.push(dir.join(&file));
            for ancestor in dir.ancestors() {
                candidates.push(
                    ancestor
                        .join("resources")
                        .join("bin")
                        .join(PLATFORM_DIR)
                        .join(&file),
                );
            }
        }
        candidates
    }

    /// The path of `tool`.
    ///
    /// # Errors
    /// [`MediaError::ToolsMissing`] naming the places searched.
    pub fn locate(&self, tool: Tool) -> Result<PathBuf, MediaError> {
        if let Some(found) = self
            .candidates(tool)
            .into_iter()
            .find(|path| path.is_file())
        {
            return Ok(found);
        }
        if self.search_path
            && let Ok(found) = which::which(tool.name())
        {
            return Ok(found);
        }
        let mut searched: Vec<String> = Vec::new();
        if let Some(dir) = self.override_dir() {
            searched.push(dir.display().to_string());
        }
        searched.push(format!("resources/bin/{PLATFORM_DIR}"));
        if self.search_path {
            searched.push("PATH".to_owned());
        }
        Err(MediaError::ToolsMissing {
            tool: tool.name().to_owned(),
            searched: format!("searched {}", searched.join(", ")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_user_folder_comes_first_then_the_executable_folder_then_resources() {
        let locator = ToolLocator::new(
            Some(PathBuf::from("custom")),
            Some(Path::new("repo").join("target").join("debug")),
            false,
        );
        let candidates = locator.candidates(Tool::Ffmpeg);
        let file = executable_name("ffmpeg");
        assert_eq!(candidates[0], Path::new("custom").join(&file));
        assert_eq!(
            candidates[1],
            Path::new("repo").join("target").join("debug").join(&file)
        );
        assert!(
            candidates.contains(
                &Path::new("repo")
                    .join("resources")
                    .join("bin")
                    .join(PLATFORM_DIR)
                    .join(&file)
            )
        );
    }

    #[test]
    fn changing_the_user_folder_affects_every_clone() {
        let locator = ToolLocator::new(None, None, false);
        let clone = locator.clone();
        locator.set_override_dir(Some(PathBuf::from("chosen")));
        assert_eq!(clone.override_dir(), Some(PathBuf::from("chosen")));
        assert_eq!(
            clone.candidates(Tool::Ffmpeg)[0],
            Path::new("chosen").join(executable_name("ffmpeg"))
        );
    }

    #[test]
    fn a_missing_tool_names_where_it_was_searched() {
        let locator = ToolLocator::new(Some(PathBuf::from("nowhere-at-all")), None, false);
        let error = locator.locate(Tool::Ffprobe).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("ffprobe was not found"), "{message}");
        assert!(message.contains("nowhere-at-all"), "{message}");
        assert!(message.contains("pnpm fetch-binaries"), "{message}");
    }
}
