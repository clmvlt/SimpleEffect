//! System commands: liveness and facts about the running core.

use se_protocol::{CommandResult, CoreInfo};

use super::HandlerContext;

/// `ping`: proves the core reads and answers requests.
pub fn ping() -> CommandResult {
    CommandResult::Ping
}

/// `getCoreInfo`: version, platform and folders of the running core.
pub fn get_core_info(context: &HandlerContext) -> CommandResult {
    CommandResult::GetCoreInfo(CoreInfo {
        version: context.version.to_owned(),
        os: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
        // Display-only strings: a non-UTF-8 path is shown with replacement characters.
        app_data_dir: context.paths.data_dir.to_string_lossy().into_owned(),
        logs_dir: context.paths.logs_dir.to_string_lossy().into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::app_paths::AppPaths;

    #[test]
    fn core_info_reports_the_version_platform_and_folders() {
        let context = HandlerContext {
            version: "9.9.9",
            paths: AppPaths {
                data_dir: PathBuf::from("data"),
                logs_dir: PathBuf::from("data").join("logs"),
            },
        };
        let CommandResult::GetCoreInfo(info) = get_core_info(&context) else {
            panic!("getCoreInfo must answer with a getCoreInfo result");
        };
        assert_eq!(info.version, "9.9.9");
        assert_eq!(info.os, std::env::consts::OS);
        assert_eq!(info.arch, std::env::consts::ARCH);
        assert_eq!(info.app_data_dir, "data");
        assert!(info.logs_dir.ends_with("logs"));
    }
}
