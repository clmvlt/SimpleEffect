//! Test helpers shared with dependent crates (feature `test-support`): time shorthands, item and
//! track builders, and a valid example project using every item kind.
//!
//! Ids come from `from_test_index`, so the example is identical on every run and its JSON
//! snapshot (`document/project.rs`) fixes the document format.

#![allow(
    clippy::expect_used,
    reason = "test helpers build constant, known-valid values; a failure is a bug in the helper"
)]

mod example_project;

pub use example_project::{example_ids, example_project};

use crate::{Flicks, Item, ItemId, ItemKind, Origin, TimeRange, Track, TrackId, TrackKind};

/// `milliseconds` as flicks.
///
/// # Panics
/// Beyond ±147 days, like every helper here: test values must be valid.
#[must_use]
pub fn millis(milliseconds: i64) -> Flicks {
    Flicks::new(milliseconds * (Flicks::PER_SECOND / 1000)).expect("test time out of range")
}

/// The range starting at `start_ms` and lasting `duration_ms` milliseconds.
///
/// # Panics
/// For a negative duration or an unrepresentable range.
#[must_use]
pub fn range_ms(start_ms: i64, duration_ms: i64) -> TimeRange {
    TimeRange::new(millis(start_ms), millis(duration_ms)).expect("invalid test range")
}

/// An item created by the user with a fresh revision.
#[must_use]
pub fn item(index: u64, timeline_range: TimeRange, kind: ItemKind) -> Item {
    Item {
        id: ItemId::from_test_index(index),
        revision: 1,
        origin: Origin::User,
        timeline_range,
        link_group: None,
        kind,
    }
}

/// An empty, unmuted track at unity gain.
#[must_use]
pub fn track(index: u64, kind: TrackKind, name: &str, items: Vec<Item>) -> Track {
    Track {
        id: TrackId::from_test_index(index),
        kind,
        name: name.to_owned(),
        muted: false,
        solo: false,
        locked: false,
        hidden: false,
        volume_db: 0.0,
        pan: 0.0,
        items,
    }
}
