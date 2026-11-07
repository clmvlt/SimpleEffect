//! Small ports for the outside world: events to clients, the wall clock and id generation.

use se_domain::Uuid;
use se_protocol::CoreEvent;

/// Pushes events to every connected client (implemented by se-core's event bus).
pub trait EventSink: Send + Sync {
    /// Publishes `event` without waiting.
    fn publish(&self, event: CoreEvent);
}

/// The wall clock, for timestamps stored with data (recent projects, journal entries).
///
/// Delays (the snapshot debounce) use tokio's timer instead, so tests control them with
/// `tokio::time::pause`.
pub trait Clock: Send + Sync {
    /// Milliseconds since the Unix epoch.
    fn now_unix_millis(&self) -> i64;
}

/// Creates ids for new entities and transactions (`docs/architecture.md` §5.2).
pub trait IdGenerator: Send + Sync {
    /// A new version-7 UUID, unique and roughly ordered by creation time.
    fn new_uuid_v7(&self) -> Uuid;
}
