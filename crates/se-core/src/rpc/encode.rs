//! Serialisation of outgoing messages.

use se_protocol::{CoreEvent, EventEnvelope, ServerMessage};

/// Sent if a server message ever fails to serialise (a bug): still a valid protocol message.
pub const SERIALISATION_FAILURE: &str = r#"{"type":"response","id":null,"error":{"code":"INTERNAL","message":"se-core failed to encode its response. Retry; if it persists, report a bug with the se-core logs."}}"#;

/// JSON text frame of a server message.
pub fn encode(message: &ServerMessage) -> String {
    serde_json::to_string(message).unwrap_or_else(|error| {
        tracing::error!(%error, "cannot serialise a server message");
        SERIALISATION_FAILURE.to_owned()
    })
}

/// JSON text frame pushing `event`.
pub fn event_frame(event: CoreEvent) -> String {
    encode(&ServerMessage::Event(EventEnvelope { event }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialisation_fallback_is_a_valid_internal_error() {
        let fallback: serde_json::Value = serde_json::from_str(SERIALISATION_FAILURE).unwrap();
        assert_eq!(fallback["type"], "response");
        assert_eq!(fallback["error"]["code"], "INTERNAL");
    }

    #[test]
    fn events_are_wrapped_in_an_event_envelope() {
        assert_eq!(
            event_frame(CoreEvent::CoreShuttingDown),
            r#"{"type":"event","event":{"kind":"coreShuttingDown"}}"#
        );
    }
}
