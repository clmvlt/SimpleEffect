//! Errors that stop the core process.

use std::io;

/// A fatal error of the core process. The message is shown in the Electron logs.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// The loopback listener could not be created.
    #[error("cannot listen on 127.0.0.1: {0}")]
    Bind(#[source] io::Error),
    /// The handshake line could not be written to stdout.
    #[error("cannot write the handshake line to stdout (is the parent process gone?): {0}")]
    Handshake(#[source] io::Error),
    /// The HTTP server stopped with an IO error.
    #[error("the HTTP server stopped unexpectedly: {0}")]
    Serve(#[source] io::Error),
}
