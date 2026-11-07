//! [`ToolLocator`]: finds `ffmpeg` and `ffprobe` (`docs/architecture.md` §7).
//!
//! Order: the folder configured by the user → the folder of the running executable (packaged app:
//! `resources/bin/`, next to se-core) → `resources/bin/<platform>/` in an ancestor of that folder
//! (development: the repository, filled by `pnpm fetch-binaries`) → the `PATH`.

use std::path::{Path, PathBuf};

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
/// found without a restart.
#[derive(Debug, Clone, Default)]
pub struct ToolLocator {
    /// Folder chosen by the user (setting `media.toolsDir`), searched first.
    pub override_dir: Option<PathBuf>,
    /// Folder of the running executable.
    pub exe_dir: Option<PathBuf>,
    /// Whether to search the `PATH` last (off in tests).
    pub search_path: bool,
}

impl ToolLocator {
    /// The locator of the running process: no override, the executable's folder, the `PATH`.
    #[must_use]
    pub fn for_current_exe(override_dir: Option<PathBuf>) -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf));
        Self {
            override_dir,
            exe_dir,
            search_path: true,
        }
    }

    /// The candidate files for `tool`, in search order (the `PATH` excluded).
    #[must_use]
    pub fn candidates(&self, tool: Tool) -> Vec<PathBuf> {
        let file = executable_name(tool.name());
        let mut candidates = Vec::new();
        if let Some(dir) = &self.override_dir {
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
        if let Some(dir) = &self.override_dir {
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
        let locator = ToolLocator {
            override_dir: Some(PathBuf::from("custom")),
            exe_dir: Some(Path::new("repo").join("target").join("debug")),
            search_path: false,
        };
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
    fn a_missing_tool_names_where_it_was_searched() {
        let locator = ToolLocator {
            override_dir: Some(PathBuf::from("nowhere-at-all")),
            exe_dir: None,
            search_path: false,
        };
        let error = locator.locate(Tool::Ffprobe).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("ffprobe was not found"), "{message}");
        assert!(message.contains("nowhere-at-all"), "{message}");
        assert!(message.contains("pnpm fetch-binaries"), "{message}");
    }
}
