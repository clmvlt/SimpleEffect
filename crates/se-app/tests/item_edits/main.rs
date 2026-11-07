//! Item edit use cases (prompt 11) against in-memory ports: each gesture is one transaction, so
//! one undo step, and ripple edits keep linked items in sync.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

mod placement;
mod support;
mod time_edits;
