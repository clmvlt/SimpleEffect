//! Parsing of incoming text frames into requests, with actionable errors for bad ones.

use se_protocol::{ClientMessage, Command, ErrorCode, ProtocolError, RequestEnvelope, RequestId};
use serde_json::{Value, json};

/// Shape of a valid request, quoted in error messages.
const REQUEST_SHAPE: &str =
    r#"{"type":"request","id":<positive integer>,"command":{"kind":"<command>", …}}"#;

/// A frame that could not be turned into a request.
#[derive(Debug, Clone, PartialEq)]
pub struct Rejection {
    /// The request id, when it could still be read, so the client can correlate the error.
    pub id: Option<RequestId>,
    /// `INVALID_REQUEST` or `UNKNOWN_COMMAND`.
    pub error: ProtocolError,
}

/// Parses one text frame received on `/rpc`.
pub fn parse_request(text: &str) -> Result<RequestEnvelope, Rejection> {
    match serde_json::from_str::<ClientMessage>(text) {
        Ok(ClientMessage::Request(request)) => Ok(request),
        Err(error) => Err(reject(text, &error)),
    }
}

fn reject(text: &str, error: &serde_json::Error) -> Rejection {
    let Ok(value) = serde_json::from_str::<Value>(text) else {
        return Rejection {
            id: None,
            error: ProtocolError::new(
                ErrorCode::InvalidRequest,
                format!("The message is not valid JSON ({error}). Send {REQUEST_SHAPE}."),
            ),
        };
    };
    let id = value.get("id").and_then(Value::as_u64);
    let kind = value
        .get("command")
        .and_then(|command| command.get("kind"))
        .and_then(Value::as_str);
    let error = match kind.and_then(unknown_kind_message) {
        Some(message) => ProtocolError::new(ErrorCode::UnknownCommand, message)
            .with_details(json!({ "kind": kind })),
        None => ProtocolError::new(
            ErrorCode::InvalidRequest,
            format!("Invalid request ({error}). Send {REQUEST_SHAPE}."),
        ),
    };
    Rejection { id, error }
}

/// The error message for `kind` if no command has that kind, `None` if the kind exists.
///
/// serde reports unknown tags as "unknown variant `x`, expected `a` or `b`": its list of expected
/// kinds is quoted, so the message always names the commands this core supports.
fn unknown_kind_message(kind: &str) -> Option<String> {
    let error = serde_json::from_value::<Command>(json!({ "kind": kind })).err()?;
    let message = error.to_string();
    message.starts_with("unknown variant").then(|| {
        format!(
            "Unknown command `{kind}` ({message}). The renderer and se-core are out of sync: \
             rebuild both from the same revision."
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rejection(text: &str) -> Rejection {
        parse_request(text).unwrap_err()
    }

    #[test]
    fn parses_a_valid_request() {
        let request =
            parse_request(r#"{"type":"request","id":4,"command":{"kind":"getCoreInfo"}}"#).unwrap();
        assert_eq!(request.id, 4);
        assert_eq!(request.command, Command::GetCoreInfo);
    }

    #[test]
    fn non_json_is_an_invalid_request_without_id() {
        let rejection = rejection("not json");
        assert_eq!(rejection.id, None);
        assert_eq!(rejection.error.code, ErrorCode::InvalidRequest);
        assert!(rejection.error.message.contains("not valid JSON"));
    }

    #[test]
    fn an_unknown_kind_is_an_unknown_command_naming_the_supported_ones() {
        let rejection = rejection(r#"{"type":"request","id":3,"command":{"kind":"explode"}}"#);
        assert_eq!(rejection.id, Some(3));
        assert_eq!(rejection.error.code, ErrorCode::UnknownCommand);
        assert!(rejection.error.message.contains("`explode`"));
        assert!(rejection.error.message.contains("getCoreInfo"));
        assert_eq!(rejection.error.details, Some(json!({ "kind": "explode" })));
    }

    #[test]
    fn a_wrong_envelope_is_an_invalid_request_keeping_the_id() {
        for text in [
            r#"{"type":"notify","id":5,"command":{"kind":"ping"}}"#,
            r#"{"type":"request","id":5}"#,
            r#"{"type":"request","id":5,"command":{"kind":7}}"#,
        ] {
            let rejection = rejection(text);
            assert_eq!(rejection.id, Some(5), "{text}");
            assert_eq!(rejection.error.code, ErrorCode::InvalidRequest, "{text}");
        }
    }

    #[test]
    fn a_missing_or_negative_id_is_an_invalid_request() {
        for text in [
            r#"{"type":"request","command":{"kind":"ping"}}"#,
            r#"{"type":"request","id":-1,"command":{"kind":"ping"}}"#,
        ] {
            let rejection = rejection(text);
            assert_eq!(rejection.id, None, "{text}");
            assert_eq!(rejection.error.code, ErrorCode::InvalidRequest, "{text}");
        }
    }
}
