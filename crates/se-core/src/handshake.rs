//! Writes the startup handshake line on stdout.

use std::io::{self, Write};

use se_protocol::CoreHandshake;

use crate::error::CoreError;

/// Prints the handshake as a single JSON line on stdout and flushes it.
///
/// This is the only stdout output of the process: the supervisor treats the first stdout line as
/// the handshake, so nothing else may ever be printed there.
pub fn announce(handshake: &CoreHandshake) -> Result<(), CoreError> {
    let line = serde_json::to_string(handshake)
        .map_err(|error| CoreError::Handshake(io::Error::other(error)))?;
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{line}").map_err(CoreError::Handshake)?;
    stdout.flush().map_err(CoreError::Handshake)
}
