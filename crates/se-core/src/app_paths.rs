//! Folders the core writes to, resolved once at startup.

use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::error::CoreError;

/// Environment variable carrying the app data folder. The Electron app sets it to its
/// `userData` folder so the app and the core share one place (logs, later settings and library).
pub const DATA_DIR_ENV: &str = "SE_DATA_DIR";

/// Folder name under the OS data directory when `SE_DATA_DIR` is not set (e.g. `cargo run`).
const APP_DIR_NAME: &str = "SimpleEffect";

/// Name of the log folder inside the app data folder.
const LOGS_DIR_NAME: &str = "logs";

/// Absolute folders used by the core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    /// The app data folder (`SE_DATA_DIR`, or `<OS data dir>/SimpleEffect`).
    pub data_dir: PathBuf,
    /// Daily log files: `<data_dir>/logs`.
    pub logs_dir: PathBuf,
}

impl AppPaths {
    /// Resolves the folders from `SE_DATA_DIR`, falling back to the OS data directory
    /// (`%APPDATA%`, `~/Library/Application Support`, `$XDG_DATA_HOME`).
    pub fn from_environment() -> Result<Self, CoreError> {
        let os_data_dir = directories::BaseDirs::new().map(|dirs| dirs.data_dir().to_path_buf());
        Self::resolve(env::var_os(DATA_DIR_ENV), os_data_dir)
    }

    /// Pure resolution rule, separated from the environment for tests.
    fn resolve(
        data_dir_override: Option<OsString>,
        os_data_dir: Option<PathBuf>,
    ) -> Result<Self, CoreError> {
        let data_dir = match data_dir_override.filter(|value| !value.is_empty()) {
            Some(value) => std::path::absolute(PathBuf::from(value)).map_err(CoreError::DataDir)?,
            None => os_data_dir
                .map(|dir| dir.join(APP_DIR_NAME))
                .ok_or(CoreError::NoDataDir)?,
        };
        Ok(Self::under(&data_dir))
    }

    fn under(data_dir: &Path) -> Self {
        Self {
            data_dir: data_dir.to_path_buf(),
            logs_dir: data_dir.join(LOGS_DIR_NAME),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_override_wins_and_logs_live_inside_it() {
        let data = std::env::temp_dir().join("se-data");
        let paths = AppPaths::resolve(
            Some(data.clone().into_os_string()),
            Some(PathBuf::from("/ignored")),
        )
        .unwrap();
        assert_eq!(paths.data_dir, data);
        assert_eq!(paths.logs_dir, data.join("logs"));
    }

    #[test]
    fn a_relative_override_becomes_absolute() {
        let paths = AppPaths::resolve(Some(OsString::from("relative-data")), None).unwrap();
        assert!(paths.data_dir.is_absolute());
        assert!(paths.data_dir.ends_with("relative-data"));
    }

    #[test]
    fn falls_back_to_the_os_data_directory() {
        let os_dir = std::env::temp_dir();
        let paths = AppPaths::resolve(Some(OsString::new()), Some(os_dir.clone())).unwrap();
        assert_eq!(paths.data_dir, os_dir.join("SimpleEffect"));
    }

    #[test]
    fn fails_with_an_actionable_error_without_any_data_directory() {
        let error = AppPaths::resolve(None, None).unwrap_err();
        assert!(error.to_string().contains(DATA_DIR_ENV));
    }
}
