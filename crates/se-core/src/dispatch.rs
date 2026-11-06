//! Pure request handling: one incoming text frame in, one outgoing text frame out.

use se_protocol::{
    ClientMessage, Command, CommandResult, ErrorCode, ProtocolError, RequestId, Response,
    ResponseOutcome, ServerMessage,
};

use crate::CORE_VERSION;

/// Sent if a server message ever fails to serialise (a bug): still a valid protocol message.
const SERIALISATION_FAILURE: &str = r#"{"type":"response","id":null,"error":{"code":"INTERNAL","message":"se-core failed to encode its response. Retry; if it persists, report a bug with the se-core logs."}}"#;

/// Handles one text frame received on `/rpc` and returns the JSON text frame to send back.
pub fn handle_text(text: &str) -> String {
    let response = match serde_json::from_str::<ClientMessage>(text) {
        Ok(ClientMessage::Request { id, command }) => Response {
            id: Some(id),
            outcome: ResponseOutcome::Result(execute(&command)),
        },
        Err(error) => Response {
            id: request_id_hint(text),
            outcome: ResponseOutcome::Error(ProtocolError {
                code: ErrorCode::InvalidRequest,
                message: format!(
                    "Invalid request ({error}). Send {{\"type\":\"request\",\"id\":<integer>,\"command\":{{\"kind\":\"ping\"}}}}."
                ),
            }),
        },
    };
    encode(&ServerMessage::Response(response))
}

fn execute(command: &Command) -> CommandResult {
    match command {
        Command::Ping => CommandResult::Pong {
            version: CORE_VERSION.to_owned(),
        },
    }
}

/// Recovers the request id from an invalid request so the client can still correlate the error.
fn request_id_hint(text: &str) -> Option<RequestId> {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()?
        .get("id")?
        .as_u64()
}

fn encode(message: &ServerMessage) -> String {
    serde_json::to_string(message).unwrap_or_else(|error| {
        tracing::error!(%error, "cannot serialise a server message");
        SERIALISATION_FAILURE.to_owned()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers_ping_with_pong_and_the_core_version() {
        let reply = handle_text(r#"{"type":"request","id":7,"command":{"kind":"ping"}}"#);
        assert_eq!(
            reply,
            format!(
                r#"{{"type":"response","id":7,"result":{{"kind":"pong","version":"{CORE_VERSION}"}}}}"#
            )
        );
    }

    #[test]
    fn answers_malformed_json_with_an_invalid_request_error() {
        let reply: serde_json::Value = serde_json::from_str(&handle_text("not json")).unwrap();
        assert_eq!(reply["type"], "response");
        assert_eq!(reply["id"], serde_json::Value::Null);
        assert_eq!(reply["error"]["code"], "INVALID_REQUEST");
    }

    #[test]
    fn keeps_the_request_id_when_the_command_is_unknown() {
        let reply: serde_json::Value = serde_json::from_str(&handle_text(
            r#"{"type":"request","id":3,"command":{"kind":"explode"}}"#,
        ))
        .unwrap();
        assert_eq!(reply["id"], 3);
        assert_eq!(reply["error"]["code"], "INVALID_REQUEST");
    }

    #[test]
    fn serialisation_fallback_is_valid_json() {
        let fallback: serde_json::Value = serde_json::from_str(SERIALISATION_FAILURE).unwrap();
        assert_eq!(fallback["error"]["code"], "INTERNAL");
    }
}
