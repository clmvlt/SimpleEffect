//! Successful results of commands.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The result of a [`crate::Command`], tagged by the **same `kind` as the command** it answers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CommandResult {
    /// Answer to [`crate::Command::Ping`]: the core is alive and reading requests.
    Ping,
    /// Answer to [`crate::Command::GetCoreInfo`].
    GetCoreInfo(CoreInfo),
}

/// Facts about the running core, shown in the UI and useful in bug reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CoreInfo {
    /// Version of the running `se-core` binary (semver).
    pub version: String,
    /// Operating system, as Rust's `std::env::consts::OS` (`windows`, `macos`, `linux`).
    pub os: String,
    /// CPU architecture, as Rust's `std::env::consts::ARCH` (`x86_64`, `aarch64`).
    pub arch: String,
    /// Absolute path of the app data folder (`SE_DATA_DIR`, set by the Electron app).
    pub app_data_dir: String,
    /// Absolute path of the folder holding the core's daily log files (`<app data>/logs`).
    pub logs_dir: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn results_carry_their_payload_next_to_the_kind() {
        let result = CommandResult::GetCoreInfo(CoreInfo {
            version: "0.1.0".to_owned(),
            os: "linux".to_owned(),
            arch: "x86_64".to_owned(),
            app_data_dir: "/data".to_owned(),
            logs_dir: "/data/logs".to_owned(),
        });
        let json = serde_json::to_string(&result).unwrap();
        assert_eq!(
            json,
            r#"{"kind":"getCoreInfo","version":"0.1.0","os":"linux","arch":"x86_64","appDataDir":"/data","logsDir":"/data/logs"}"#
        );
        assert_eq!(
            serde_json::from_str::<CommandResult>(&json).unwrap(),
            result
        );
    }

    #[test]
    fn unit_results_are_just_the_kind() {
        assert_eq!(
            serde_json::to_string(&CommandResult::Ping).unwrap(),
            r#"{"kind":"ping"}"#
        );
    }
}
