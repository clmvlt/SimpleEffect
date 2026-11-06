//! Tracing setup. Logs go to stderr: stdout is reserved for the handshake line.

use std::io::{self, IsTerminal};

use tracing_subscriber::EnvFilter;

/// Default filter when `RUST_LOG` is not set.
const DEFAULT_FILTER: &str = "info";

/// Installs the global tracing subscriber writing to stderr.
///
/// The filter comes from `RUST_LOG` (e.g. `RUST_LOG=se_core=debug`). ANSI colours are only used
/// on a terminal, so the Electron supervisor receives plain text.
pub fn init() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER));
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_ansi(io::stderr().is_terminal())
        .with_env_filter(filter)
        .init();
}
