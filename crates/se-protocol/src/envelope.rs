//! Messages of the `/rpc` WebSocket (see `docs/architecture.md` §6), tagged by `type`.
//!
//! - Client → core: `{"type":"request","id":1,"command":{"kind":"ping"}}`
//! - Core → client: `{"type":"response","id":1,"result":{"kind":"ping"}}`,
//!   `{"type":"response","id":1,"error":{"code":"UNKNOWN_COMMAND","message":"…"}}` or
//!   `{"type":"event","event":{"kind":"coreShuttingDown"}}`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Command, CommandResult, CoreEvent, ProtocolError};

/// Client-chosen identifier correlating a response with its request.
///
/// Exported to TypeScript as `number`: clients count up from 1, far below 2^53.
pub type RequestId = u64;

/// A message sent by a client to `se-core`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ClientMessage {
    /// Asks the core to execute a command; answered by exactly one [`ServerMessage::Response`].
    Request(RequestEnvelope),
}

/// A command to execute, with the id its response will carry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RequestEnvelope {
    /// Echoed back in the response. Unique per client connection lifetime.
    pub id: RequestId,
    /// The command to execute.
    pub command: Command,
}

/// A message sent by `se-core` to a client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ServerMessage {
    /// The answer to a [`ClientMessage::Request`].
    Response(ResponseEnvelope),
    /// An event pushed by the core, not tied to any request.
    Event(EventEnvelope),
}

/// The answer to a request: either a result or an error, never both.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ResponseEnvelope {
    /// Id of the request being answered, or `None` when the request could not be read at all.
    pub id: Option<RequestId>,
    /// Result or error, serialised as a `result` or `error` field next to `id`.
    #[serde(flatten)]
    pub outcome: ResponseOutcome,
}

/// Outcome of a request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[allow(
    clippy::large_enum_variant,
    reason = "an outcome is built once per request and serialised right away"
)]
pub enum ResponseOutcome {
    /// The command succeeded.
    Result(CommandResult),
    /// The command failed or the request was invalid.
    Error(ProtocolError),
}

/// An event pushed by the core.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EventEnvelope {
    /// The event.
    pub event: CoreEvent,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CoreInfo, ErrorCode};

    fn round_trip_client(message: &ClientMessage, expected_json: &str) {
        let json = serde_json::to_string(message).unwrap();
        assert_eq!(json, expected_json);
        assert_eq!(
            &serde_json::from_str::<ClientMessage>(&json).unwrap(),
            message
        );
    }

    fn round_trip_server(message: &ServerMessage, expected_json: &str) {
        let json = serde_json::to_string(message).unwrap();
        assert_eq!(json, expected_json);
        assert_eq!(
            &serde_json::from_str::<ServerMessage>(&json).unwrap(),
            message
        );
    }

    #[test]
    fn request_envelope_round_trips_in_the_documented_shape() {
        round_trip_client(
            &ClientMessage::Request(RequestEnvelope {
                id: 1,
                command: Command::Ping,
            }),
            r#"{"type":"request","id":1,"command":{"kind":"ping"}}"#,
        );
    }

    #[test]
    fn result_response_round_trips_in_the_documented_shape() {
        round_trip_server(
            &ServerMessage::Response(ResponseEnvelope {
                id: Some(2),
                outcome: ResponseOutcome::Result(CommandResult::GetCoreInfo(CoreInfo {
                    version: "0.1.0".to_owned(),
                    os: "windows".to_owned(),
                    arch: "x86_64".to_owned(),
                    app_data_dir: "C:\\data".to_owned(),
                    logs_dir: "C:\\data\\logs".to_owned(),
                })),
            }),
            r#"{"type":"response","id":2,"result":{"kind":"getCoreInfo","version":"0.1.0","os":"windows","arch":"x86_64","appDataDir":"C:\\data","logsDir":"C:\\data\\logs"}}"#,
        );
    }

    #[test]
    fn error_response_round_trips_with_a_null_id() {
        round_trip_server(
            &ServerMessage::Response(ResponseEnvelope {
                id: None,
                outcome: ResponseOutcome::Error(ProtocolError::new(
                    ErrorCode::InvalidRequest,
                    "bad",
                )),
            }),
            r#"{"type":"response","id":null,"error":{"code":"INVALID_REQUEST","message":"bad"}}"#,
        );
    }

    #[test]
    fn event_envelope_round_trips_in_the_documented_shape() {
        round_trip_server(
            &ServerMessage::Event(EventEnvelope {
                event: CoreEvent::CoreShuttingDown,
            }),
            r#"{"type":"event","event":{"kind":"coreShuttingDown"}}"#,
        );
    }
}
