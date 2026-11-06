//! Fan-out of core events to every `/rpc` connection.

use se_protocol::CoreEvent;
use tokio::sync::broadcast::{self, error::RecvError};

/// Events buffered per connection before a slow one starts losing them (and gets
/// [`CoreEvent::ResyncRequired`] instead).
const DEFAULT_CAPACITY: usize = 256;

/// Broadcasts [`CoreEvent`]s to all connections.
///
/// Publishing never waits: each connection reads from its own position in a bounded ring buffer,
/// so a slow connection only ever delays itself. When it falls more than the capacity behind, it
/// skips the oldest events and is told to resync.
#[derive(Debug, Clone)]
pub struct EventBus {
    sender: broadcast::Sender<CoreEvent>,
}

impl EventBus {
    /// A bus buffering [`DEFAULT_CAPACITY`] events per connection.
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// A bus buffering `capacity` events per connection (at least 1).
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity.max(1));
        Self { sender }
    }

    /// Sends `event` to every current connection. Events published while nobody is connected are
    /// dropped: new connections start from [`CoreEvent::CoreReady`] and fetch the state they need.
    pub fn publish(&self, event: CoreEvent) {
        // `Err` only means "no connection right now", which is fine.
        let _ = self.sender.send(event);
    }

    /// A receiver getting every event published from now on.
    pub fn subscribe(&self) -> broadcast::Receiver<CoreEvent> {
        self.sender.subscribe()
    }
}

/// The event a connection must forward for one receive result, or `None` once the bus is gone.
pub fn event_to_forward(received: Result<CoreEvent, RecvError>) -> Option<CoreEvent> {
    match received {
        Ok(event) => Some(event),
        Err(RecvError::Lagged(missed_events)) => Some(CoreEvent::ResyncRequired { missed_events }),
        Err(RecvError::Closed) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready(version: &str) -> CoreEvent {
        CoreEvent::CoreReady {
            version: version.to_owned(),
        }
    }

    #[tokio::test]
    async fn every_subscriber_receives_every_event() {
        let bus = EventBus::new();
        let mut first = bus.subscribe();
        let mut second = bus.subscribe();
        bus.publish(CoreEvent::CoreShuttingDown);
        assert_eq!(first.recv().await.unwrap(), CoreEvent::CoreShuttingDown);
        assert_eq!(second.recv().await.unwrap(), CoreEvent::CoreShuttingDown);
    }

    #[tokio::test]
    async fn a_slow_subscriber_is_told_to_resync_without_slowing_the_others() {
        let bus = EventBus::with_capacity(2);
        let mut slow = bus.subscribe();
        let mut fast = bus.subscribe();
        for index in 0..5 {
            bus.publish(ready(&index.to_string()));
            // The fast subscriber keeps up; publishing never waits for the slow one.
            assert_eq!(fast.recv().await.unwrap(), ready(&index.to_string()));
        }
        assert_eq!(
            event_to_forward(slow.recv().await),
            Some(CoreEvent::ResyncRequired { missed_events: 3 })
        );
        // After the resync notice, the slow subscriber continues with the newest events.
        assert_eq!(event_to_forward(slow.recv().await), Some(ready("3")));
    }

    #[tokio::test]
    async fn forwarding_stops_once_the_bus_is_dropped() {
        let bus = EventBus::new();
        let mut receiver = bus.subscribe();
        drop(bus);
        assert_eq!(event_to_forward(receiver.recv().await), None);
    }
}
