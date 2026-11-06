//! The single JSON line `se-core` prints on stdout once it is ready to accept connections.

use std::fmt;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Connection details announced by `se-core` on stdout at startup, read by the Electron supervisor.
///
/// Serialised as exactly one line: `{"port":<u16>,"token":"<hex>","version":"<semver>"}`.
/// Everything else `se-core` logs goes to stderr. Exported to TypeScript for the Electron
/// supervisor, which parses and validates the line.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct CoreHandshake {
    /// TCP port on 127.0.0.1 where `se-core` listens (chosen by the OS).
    pub port: u16,
    /// Session token (256 random bits, lowercase hex) every client must present.
    pub token: String,
    /// Version of the running `se-core` binary.
    pub version: String,
}

/// Redacts the token so the handshake can never leak through logs.
impl fmt::Debug for CoreHandshake {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CoreHandshake")
            .field("port", &self.port)
            .field("token", &"<redacted>")
            .field("version", &self.version)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> CoreHandshake {
        CoreHandshake {
            port: 5123,
            token: "ab".repeat(32),
            version: "0.1.0".to_owned(),
        }
    }

    #[test]
    fn serialises_to_the_documented_single_line_shape() {
        let json = serde_json::to_string(&sample()).unwrap();
        assert_eq!(
            json,
            format!(
                r#"{{"port":5123,"token":"{}","version":"0.1.0"}}"#,
                "ab".repeat(32)
            )
        );
    }

    #[test]
    fn debug_output_never_contains_the_token() {
        let debug = format!("{:?}", sample());
        assert!(!debug.contains(&"ab".repeat(32)));
        assert!(debug.contains("<redacted>"));
    }
}
