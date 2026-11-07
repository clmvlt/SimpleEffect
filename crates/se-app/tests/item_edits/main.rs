//! Item edit use cases (prompts 11 and 14) against in-memory ports: each gesture is one
//! transaction, so one undo step, ripple edits keep linked items in sync and transitions follow
//! their items.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

mod graphics;
mod placement;
mod sound;
mod support;
mod time_edits;
