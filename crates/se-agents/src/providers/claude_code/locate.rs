//! Finding the `claude` executable: the path chosen in the settings, else the PATH (on Windows a
//! real `claude.exe` first, then what PATHEXT finds — the `claude.cmd` shim of npm installs),
//! else the usual install folders.

use std::path::{Path, PathBuf};

use se_app::ports::AgentError;

use super::{
    identity::EXECUTABLE,
    platform::{PATH_NAMES, install_locations},
};

/// The executable to run.
///
/// # Errors
/// [`AgentError::NotInstalled`] naming where it looked and what to do.
pub(super) fn resolve(configured: Option<&Path>) -> Result<PathBuf, AgentError> {
    let home = directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf());
    let on_path = PATH_NAMES.iter().find_map(|name| which::which(name).ok());
    resolve_with(configured, on_path, home.as_deref())
}

/// The first of [`PATH_NAMES`] found in the folders of `paths` (a PATH value; Windows tests).
#[cfg(all(test, windows))]
fn on_path_in(paths: &std::ffi::OsStr, cwd: &Path) -> Option<PathBuf> {
    PATH_NAMES
        .iter()
        .find_map(|name| which::which_in(name, Some(paths), cwd).ok())
}

/// The rule of [`resolve`], with the PATH lookup and home folder given (tests).
pub(super) fn resolve_with(
    configured: Option<&Path>,
    on_path: Option<PathBuf>,
    home: Option<&Path>,
) -> Result<PathBuf, AgentError> {
    if let Some(path) = configured {
        return if path.is_file() {
            Ok(path.to_path_buf())
        } else {
            Err(AgentError::NotInstalled {
                message: format!(
                    "no Claude Code executable at {} (path set in Réglages → IA): correct it, or \
                     clear it to search the usual places",
                    path.display()
                ),
            })
        };
    }
    if let Some(path) = on_path {
        return Ok(path);
    }
    let candidates = home.map(install_locations).unwrap_or_default();
    if let Some(found) = candidates.iter().find(|candidate| candidate.is_file()) {
        return Ok(found.clone());
    }
    let searched: Vec<String> = candidates
        .iter()
        .map(|candidate| candidate.display().to_string())
        .collect();
    Err(AgentError::NotInstalled {
        message: format!(
            "Claude Code (`{EXECUTABLE}`) was not found on the PATH nor in {}: install it, or set \
             its path in Réglages → IA",
            if searched.is_empty() {
                "the usual install folders".to_owned()
            } else {
                searched.join(", ")
            }
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_configured_path_wins_and_must_exist() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("claude-custom");
        std::fs::write(&file, "").unwrap();
        let found = resolve_with(Some(&file), Some(PathBuf::from("/elsewhere")), None).unwrap();
        assert_eq!(found, file);
        let missing = resolve_with(Some(&dir.path().join("nope")), None, None).unwrap_err();
        assert!(matches!(missing, AgentError::NotInstalled { .. }));
        assert!(missing.to_string().contains("Réglages"));
    }

    #[test]
    fn the_path_comes_before_the_usual_folders() {
        let home = tempfile::tempdir().unwrap();
        let installed = install_locations(home.path())[0].clone();
        std::fs::create_dir_all(installed.parent().unwrap()).unwrap();
        std::fs::write(&installed, "").unwrap();
        let on_path = PathBuf::from("/usr/bin/claude");
        assert_eq!(
            resolve_with(None, Some(on_path.clone()), Some(home.path())).unwrap(),
            on_path
        );
        assert_eq!(
            resolve_with(None, None, Some(home.path())).unwrap(),
            installed
        );
    }

    #[cfg(windows)]
    #[test]
    fn a_real_executable_on_the_path_wins_over_a_script_wrapper_before_it() {
        let wrapper = tempfile::tempdir().unwrap();
        let native = tempfile::tempdir().unwrap();
        std::fs::write(wrapper.path().join("claude.cmd"), "@echo off").unwrap();
        std::fs::write(native.path().join("claude.exe"), "").unwrap();
        let paths = std::env::join_paths([wrapper.path(), native.path()]).unwrap();
        let found = on_path_in(&paths, wrapper.path()).unwrap();
        assert!(
            found.starts_with(native.path()) && found.ends_with("claude.exe"),
            "{}",
            found.display()
        );
        // Without a real executable, the wrapper is still found (npm installs).
        let only_wrapper = std::env::join_paths([wrapper.path()]).unwrap();
        let found = on_path_in(&only_wrapper, wrapper.path()).unwrap();
        assert!(
            found
                .to_string_lossy()
                .to_lowercase()
                .ends_with("claude.cmd")
        );
    }

    #[test]
    fn nothing_found_says_where_it_looked() {
        let home = tempfile::tempdir().unwrap();
        let error = resolve_with(None, None, Some(home.path())).unwrap_err();
        let message = error.to_string();
        assert!(
            message.contains(&home.path().display().to_string()),
            "{message}"
        );
        assert!(message.contains("install it"));
    }
}
