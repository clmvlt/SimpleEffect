//! [`WhisperLocator`]: finds `whisper-cli` (`docs/architecture.md` §7, ADR-0031).
//!
//! Order: the folder chosen by the user (« Réglages → Transcription ») → the GPU pack when it is
//! installed and the GPU may be used → `whisper/` next to the running executable (packaged app:
//! `resources/bin/whisper/`) → `resources/bin/<platform>/whisper/` in an ancestor of that folder
//! (development: filled by `pnpm fetch-binaries`) → the `PATH`. Looked up on every run, so a
//! pack installed while the app runs is used at once.

use std::{
    path::{Path, PathBuf},
    sync::{Arc, PoisonError, RwLock},
};

use se_app::ports::TranscriberError;

use crate::platform::{PLATFORM_DIR, whisper_executable};

/// Where `whisper-cli` was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Located {
    /// The program.
    pub path: PathBuf,
    /// Whether it is the GPU pack's.
    pub gpu_pack: bool,
}

/// Where to look for `whisper-cli`. Clones share the user's folder.
#[derive(Debug, Clone, Default)]
pub struct WhisperLocator {
    override_dir: Arc<RwLock<Option<PathBuf>>>,
    /// Folder of the installed GPU pack (whether it exists is checked on every lookup).
    gpu_pack_dir: Option<PathBuf>,
    /// Folder of the running executable.
    exe_dir: Option<PathBuf>,
    /// Whether to search the `PATH` last (off in tests).
    search_path: bool,
}

impl WhisperLocator {
    /// A locator searching the given places.
    #[must_use]
    pub fn new(gpu_pack_dir: Option<PathBuf>, exe_dir: Option<PathBuf>, search_path: bool) -> Self {
        Self {
            override_dir: Arc::default(),
            gpu_pack_dir,
            exe_dir,
            search_path,
        }
    }

    /// The locator of the running process, with the GPU pack at `gpu_pack_dir`.
    #[must_use]
    pub fn for_current_exe(gpu_pack_dir: Option<PathBuf>) -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf));
        Self::new(gpu_pack_dir, exe_dir, true)
    }

    /// Searches `dir` first from now on (every clone), or no user folder when `None`.
    pub fn set_override_dir(&self, dir: Option<PathBuf>) {
        *self
            .override_dir
            .write()
            .unwrap_or_else(PoisonError::into_inner) = dir;
    }

    fn override_dir(&self) -> Option<PathBuf> {
        self.override_dir
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// The GPU pack's program, if installed.
    #[must_use]
    pub fn gpu_pack_program(&self) -> Option<PathBuf> {
        let program = self.gpu_pack_dir.as_ref()?.join(whisper_executable());
        program.is_file().then_some(program)
    }

    /// The candidates in search order (the `PATH` excluded), each with whether it is the pack's.
    #[must_use]
    pub fn candidates(&self, prefer_gpu: bool) -> Vec<Located> {
        let file = whisper_executable();
        let mut candidates = Vec::new();
        let mut push = |path: PathBuf, gpu_pack: bool| candidates.push(Located { path, gpu_pack });
        if let Some(dir) = self.override_dir() {
            push(dir.join(file), false);
        }
        if prefer_gpu && let Some(dir) = &self.gpu_pack_dir {
            push(dir.join(file), true);
        }
        if let Some(dir) = &self.exe_dir {
            push(dir.join("whisper").join(file), false);
            for ancestor in dir.ancestors() {
                let bundled = ancestor.join("resources").join("bin").join(PLATFORM_DIR);
                push(bundled.join("whisper").join(file), false);
            }
        }
        candidates
    }

    /// The program to run.
    ///
    /// # Errors
    /// [`TranscriberError::WhisperMissing`] naming where it looked.
    pub fn locate(&self, prefer_gpu: bool) -> Result<Located, TranscriberError> {
        let candidates = self.candidates(prefer_gpu);
        if let Some(found) = candidates.iter().find(|candidate| candidate.path.is_file()) {
            return Ok(found.clone());
        }
        if self.search_path
            && let Ok(path) = which::which("whisper-cli")
        {
            return Ok(Located {
                path,
                gpu_pack: false,
            });
        }
        let mut searched: Vec<String> = candidates
            .iter()
            .take(4)
            .map(|candidate| candidate.path.display().to_string())
            .collect();
        if self.search_path {
            searched.push("the PATH".to_owned());
        }
        Err(TranscriberError::WhisperMissing {
            searched: searched.join(", "),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn searches_the_users_folder_the_gpu_pack_then_the_bundled_builds() {
        let root = tempfile::tempdir().unwrap();
        let exe_dir = root.path().join("app").join("target").join("debug");
        let bundled = root
            .path()
            .join("app")
            .join("resources")
            .join("bin")
            .join(PLATFORM_DIR)
            .join("whisper");
        let pack = root.path().join("runtimes").join("whisper-cuda");
        for dir in [&exe_dir, &bundled, &pack] {
            std::fs::create_dir_all(dir).unwrap();
        }
        let locator = WhisperLocator::new(Some(pack.clone()), Some(exe_dir), false);
        assert!(matches!(
            locator.locate(true),
            Err(TranscriberError::WhisperMissing { .. })
        ));
        std::fs::write(bundled.join(whisper_executable()), b"").unwrap();
        assert_eq!(
            locator.locate(true).unwrap().path,
            bundled.join(whisper_executable())
        );
        std::fs::write(pack.join(whisper_executable()), b"").unwrap();
        let found = locator.locate(true).unwrap();
        assert!(found.gpu_pack && found.path.starts_with(&pack));
        assert!(
            !locator.locate(false).unwrap().gpu_pack,
            "no pack when the GPU is off"
        );
        let own = root.path().join("mine");
        std::fs::create_dir_all(&own).unwrap();
        std::fs::write(own.join(whisper_executable()), b"").unwrap();
        locator.set_override_dir(Some(own.clone()));
        assert!(locator.clone().locate(true).unwrap().path.starts_with(&own));
    }
}
