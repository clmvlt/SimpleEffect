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

/// File name of the app library (recent projects, settings) inside the app data folder.
const LIBRARY_FILE_NAME: &str = "library.db";

/// Folder of the working folders of AI agent runs, inside the app data folder.
const AGENT_WORKDIRS_DIR_NAME: &str = "agent-workdirs";
/// Folder of whisper models, under `models/` of the data folder.
const WHISPER_MODELS_DIR: [&str; 2] = ["models", "whisper"];
/// Folder of downloaded runtimes (the GPU pack of whisper.cpp).
const RUNTIMES_DIR_NAME: &str = "runtimes";

/// Folder proposed for new projects, inside the user's documents folder.
const PROJECTS_DIR_NAME: &str = "SimpleEffect";

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

    /// The app library database: `<data_dir>/library.db`.
    pub fn library_db(&self) -> PathBuf {
        self.data_dir.join(LIBRARY_FILE_NAME)
    }

    /// The working folders of AI agent runs: `<data_dir>/agent-workdirs`.
    pub fn agent_workdirs(&self) -> PathBuf {
        self.data_dir.join(AGENT_WORKDIRS_DIR_NAME)
    }

    /// Transcription models: `<data_dir>/models/whisper`.
    pub fn whisper_models(&self) -> PathBuf {
        WHISPER_MODELS_DIR
            .iter()
            .fold(self.data_dir.clone(), |dir, name| dir.join(name))
    }

    /// Downloaded runtimes (the GPU pack of whisper.cpp): `<data_dir>/runtimes`.
    pub fn runtimes(&self) -> PathBuf {
        self.data_dir.join(RUNTIMES_DIR_NAME)
    }

    /// Folder proposed for new projects: `<Documents>/SimpleEffect`, or `<data_dir>/projects`
    /// when the OS has no documents folder.
    pub fn default_projects_dir(&self) -> PathBuf {
        let documents = directories::UserDirs::new()
            .and_then(|dirs| dirs.document_dir().map(Path::to_path_buf));
        self.projects_dir_in(documents)
    }

    /// Pure rule of [`AppPaths::default_projects_dir`], separated from the environment for tests.
    fn projects_dir_in(&self, documents: Option<PathBuf>) -> PathBuf {
        match documents {
            Some(documents) => documents.join(PROJECTS_DIR_NAME),
            None => self.data_dir.join("projects"),
        }
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
    fn projects_go_to_the_documents_folder_or_the_data_folder() {
        let paths = AppPaths::under(Path::new("/data"));
        assert_eq!(paths.library_db(), Path::new("/data").join("library.db"));
        assert_eq!(
            paths.agent_workdirs(),
            Path::new("/data").join("agent-workdirs")
        );
        assert_eq!(
            paths.projects_dir_in(Some(PathBuf::from("/docs"))),
            Path::new("/docs").join("SimpleEffect")
        );
        assert_eq!(
            paths.projects_dir_in(None),
            Path::new("/data").join("projects")
        );
    }

    #[test]
    fn fails_with_an_actionable_error_without_any_data_directory() {
        let error = AppPaths::resolve(None, None).unwrap_err();
        assert!(error.to_string().contains(DATA_DIR_ENV));
    }
}
