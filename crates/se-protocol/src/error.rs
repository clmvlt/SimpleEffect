//! Protocol errors: a stable code plus an actionable message.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// An error answering a request. Messages are read by users and AI agents: keep them actionable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ProtocolError {
    /// Stable, machine-readable error code.
    pub code: ErrorCode,
    /// Human- and agent-readable explanation, including how to fix the problem.
    pub message: String,
    /// Optional structured context (e.g. the unknown command kind). Omitted when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub details: Option<serde_json::Value>,
}

impl ProtocolError {
    /// An error without details.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    /// The same error carrying structured `details`.
    #[must_use]
    pub fn with_details(self, details: serde_json::Value) -> Self {
        Self {
            details: Some(details),
            ..self
        }
    }
}

/// Stable error codes (`SCREAMING_SNAKE_CASE` on the wire). Never rename or reuse a code: clients
/// and AI agents match on them. Later prompts add codes for their own failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// The message is not valid JSON, is not a request envelope, or the command's fields are wrong.
    InvalidRequest,
    /// The request is well-formed but this `se-core` does not know the command `kind`.
    UnknownCommand,
    /// Unexpected failure inside `se-core` (a bug): retry, and report it if it persists.
    Internal,
    /// No response arrived in time. Raised by clients (`CoreClient`), not sent by `se-core`.
    Timeout,
    /// The connection dropped after the request was sent: the command may or may not have run.
    /// Raised by clients, not sent by `se-core`.
    ConnectionLost,
    /// The client was closed by its owner before the response arrived. Raised by clients.
    ClientClosed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn details_are_omitted_when_absent_and_kept_when_present() {
        let plain = ProtocolError::new(ErrorCode::Timeout, "late");
        assert_eq!(
            serde_json::to_string(&plain).unwrap(),
            r#"{"code":"TIMEOUT","message":"late"}"#
        );
        let detailed = ProtocolError::new(ErrorCode::UnknownCommand, "nope")
            .with_details(serde_json::json!({ "kind": "explode" }));
        let json = serde_json::to_string(&detailed).unwrap();
        assert_eq!(
            json,
            r#"{"code":"UNKNOWN_COMMAND","message":"nope","details":{"kind":"explode"}}"#
        );
        assert_eq!(
            serde_json::from_str::<ProtocolError>(&json).unwrap(),
            detailed
        );
    }

    #[test]
    fn codes_are_screaming_snake_case() {
        assert_eq!(
            serde_json::to_string(&ErrorCode::ConnectionLost).unwrap(),
            r#""CONNECTION_LOST""#
        );
    }
}
