//! Messages of the `/rpc` WebSocket (see `docs/architecture.md` §6).
//!
//! - Client → core: `{"type":"request","id":1,"command":{"kind":"ping"}}`
//! - Core → client: `{"type":"response","id":1,"result":{"kind":"pong","version":"0.1.0"}}`
//!   or `{"type":"response","id":1,"error":{"code":"INVALID_REQUEST","message":"…"}}`.

use serde::{Deserialize, Serialize};

/// Client-chosen identifier correlating a response with its request.
pub type RequestId = u64;

/// A message sent by a client to `se-core`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ClientMessage {
    /// Asks the core to execute a command; answered by exactly one [`ServerMessage::Response`].
    Request {
        /// Echoed back in the response.
        id: RequestId,
        /// The command to execute.
        command: Command,
    },
}

/// A command a client can ask `se-core` to execute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Command {
    /// Liveness check; answered by [`CommandResult::Pong`].
    Ping,
}

/// The successful result of a [`Command`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CommandResult {
    /// Answer to [`Command::Ping`].
    Pong {
        /// Version of the running `se-core` binary.
        version: String,
    },
}

/// A message sent by `se-core` to a client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ServerMessage {
    /// The answer to a [`ClientMessage::Request`].
    Response(Response),
}

/// The answer to a request: either a result or an error, never both.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    /// Id of the request being answered, or `None` when the request could not be read at all.
    pub id: Option<RequestId>,
    /// Result or error, serialised as a `result` or `error` field.
    #[serde(flatten)]
    pub outcome: ResponseOutcome,
}

/// Outcome of a request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResponseOutcome {
    /// The command succeeded.
    Result(CommandResult),
    /// The command failed or the request was invalid.
    Error(ProtocolError),
}

/// An error returned to a client. Messages are read by users and AI agents: keep them actionable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolError {
    /// Stable, machine-readable error code.
    pub code: ErrorCode,
    /// Human- and agent-readable explanation, including how to fix the problem.
    pub message: String,
}

/// Stable error codes of the protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// The message is not valid JSON or does not match any known request shape.
    InvalidRequest,
    /// Unexpected failure inside `se-core` (a bug): retry, and report it if it persists.
    Internal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_documented_ping_request() {
        let message: ClientMessage =
            serde_json::from_str(r#"{"type":"request","id":1,"command":{"kind":"ping"}}"#).unwrap();
        assert_eq!(
            message,
            ClientMessage::Request {
                id: 1,
                command: Command::Ping
            }
        );
    }

    #[test]
    fn serialises_a_pong_response_in_the_documented_shape() {
        let message = ServerMessage::Response(Response {
            id: Some(1),
            outcome: ResponseOutcome::Result(CommandResult::Pong {
                version: "0.1.0".to_owned(),
            }),
        });
        assert_eq!(
            serde_json::to_string(&message).unwrap(),
            r#"{"type":"response","id":1,"result":{"kind":"pong","version":"0.1.0"}}"#
        );
    }

    #[test]
    fn serialises_an_error_response_with_a_stable_code() {
        let message = ServerMessage::Response(Response {
            id: None,
            outcome: ResponseOutcome::Error(ProtocolError {
                code: ErrorCode::InvalidRequest,
                message: "bad".to_owned(),
            }),
        });
        assert_eq!(
            serde_json::to_string(&message).unwrap(),
            r#"{"type":"response","id":null,"error":{"code":"INVALID_REQUEST","message":"bad"}}"#
        );
    }
}
