//! Production implementations of the small se-app ports: wall clock, UUIDv7 ids, and the event
//! bus as an event sink.

use std::time::{SystemTime, UNIX_EPOCH};

use se_app::ports::{Clock, EventSink, IdGenerator};
use se_domain::Uuid;
use se_protocol::CoreEvent;

use crate::rpc::EventBus;

/// The system clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix_millis(&self) -> i64 {
        // A clock set before 1970 reads as the epoch rather than failing.
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| {
                i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
            })
    }
}

/// Random, time-ordered UUIDv7 ids.
#[derive(Debug, Clone, Copy, Default)]
pub struct UuidV7Ids;

impl IdGenerator for UuidV7Ids {
    fn new_uuid_v7(&self) -> Uuid {
        Uuid::now_v7()
    }
}

impl EventSink for EventBus {
    fn publish(&self, event: CoreEvent) {
        EventBus::publish(self, event);
    }
}

#[cfg(test)]
mod tests {
    use se_domain::ItemId;

    use super::*;

    #[test]
    fn ids_are_valid_uuid_v7() {
        let uuid = UuidV7Ids.new_uuid_v7();
        assert!(ItemId::from_uuid(uuid).is_ok());
        assert_ne!(uuid, UuidV7Ids.new_uuid_v7());
    }

    #[test]
    fn the_clock_reads_after_2025() {
        assert!(SystemClock.now_unix_millis() > 1_735_689_600_000);
    }
}
