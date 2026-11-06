//! End-to-end tests of the real `se-core` binary. Each test spawns its own core on a random port
//! with its own temporary data folder (`SE_DATA_DIR`), so tests run in parallel and never touch the
//! user's app data.

// Test helpers outside #[test] functions must also fail loudly on unexpected states.
#![allow(clippy::expect_used, clippy::panic)]

mod running_core;

mod access;
mod commands;
mod events;
mod shutdown_and_logs;
