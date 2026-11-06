//! Events pushed by `se-core` to its clients.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// An event pushed inside a [`crate::EventEnvelope`], tagged by `kind` (camelCase).
///
/// Named `CoreEvent` rather than `Event` so the generated TypeScript type does not shadow the DOM
/// `Event` type in the renderer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CoreEvent {
    /// First message of every new connection: the core is ready to execute commands.
    CoreReady {
        /// Version of the running `se-core` binary.
        version: String,
    },
    /// The core is stopping (the app is quitting or restarting it): the connection closes next.
    CoreShuttingDown,
    /// This connection fell behind and the core dropped events for it: refetch any state built
    /// from events (e.g. request a fresh project snapshot).
    ResyncRequired {
        /// Number of events that were dropped.
        missed_events: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struct_variant_fields_are_camel_case() {
        let json = serde_json::to_string(&CoreEvent::ResyncRequired { missed_events: 3 }).unwrap();
        assert_eq!(json, r#"{"kind":"resyncRequired","missedEvents":3}"#);
    }

    #[test]
    fn every_event_round_trips() {
        let events = [
            CoreEvent::CoreReady {
                version: "0.1.0".to_owned(),
            },
            CoreEvent::CoreShuttingDown,
            CoreEvent::ResyncRequired { missed_events: 12 },
        ];
        for event in events {
            let json = serde_json::to_string(&event).unwrap();
            assert_eq!(serde_json::from_str::<CoreEvent>(&json).unwrap(), event);
        }
    }
}
