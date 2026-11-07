//! Fakes of [`EventSink`], [`Clock`] and [`IdGenerator`].

use std::sync::{
    Mutex,
    atomic::{AtomicI64, AtomicU64, Ordering},
};

use se_domain::{ProjectId, Uuid};
use se_protocol::CoreEvent;

use super::lock;
use crate::ports::{Clock, EventSink, IdGenerator};

/// Keeps every published event, in order.
#[derive(Debug, Default)]
pub struct RecordingEventSink {
    events: Mutex<Vec<CoreEvent>>,
}

impl RecordingEventSink {
    /// The events published since the last [`RecordingEventSink::take`].
    pub fn take(&self) -> Vec<CoreEvent> {
        std::mem::take(&mut *lock(&self.events))
    }
}

impl EventSink for RecordingEventSink {
    fn publish(&self, event: CoreEvent) {
        lock(&self.events).push(event);
    }
}

/// A clock that only moves when told to.
#[derive(Debug)]
pub struct FixedClock {
    now: AtomicI64,
}

impl FixedClock {
    /// A clock showing `unix_millis`.
    #[must_use]
    pub fn at(unix_millis: i64) -> Self {
        Self {
            now: AtomicI64::new(unix_millis),
        }
    }

    /// Moves the clock forward by `millis`.
    pub fn advance(&self, millis: i64) {
        self.now.fetch_add(millis, Ordering::SeqCst);
    }
}

impl Default for FixedClock {
    /// 2026-01-01T00:00:00Z.
    fn default() -> Self {
        Self::at(1_767_225_600_000)
    }
}

impl Clock for FixedClock {
    fn now_unix_millis(&self) -> i64 {
        self.now.load(Ordering::SeqCst)
    }
}

/// Version-7 UUIDs from a counter (`from_test_index`), starting far above the indices used by
/// `se_domain::test_support`, so they never collide with example ids.
#[derive(Debug)]
pub struct SequentialIds {
    next: AtomicU64,
}

impl Default for SequentialIds {
    fn default() -> Self {
        Self {
            next: AtomicU64::new(1_000_000),
        }
    }
}

impl IdGenerator for SequentialIds {
    fn new_uuid_v7(&self) -> Uuid {
        let index = self.next.fetch_add(1, Ordering::SeqCst);
        ProjectId::from_test_index(index).as_uuid()
    }
}
