//! Shutdown triggers: stdin reaching end-of-file (the parent is gone) or Ctrl+C.

use std::io::{self, Read};

use tokio::sync::oneshot;

/// Resolves when the core should stop: its stdin closed or Ctrl+C was pressed.
///
/// The Electron supervisor keeps the core's stdin pipe open and never writes to it. The pipe closes
/// when the app asks the core to stop or when the app process dies for any reason, so watching it
/// guarantees the core never outlives the app.
pub async fn requested() {
    tokio::select! {
        () = stdin_closed() => tracing::info!("stdin closed: shutting down"),
        () = ctrl_c() => tracing::info!("Ctrl+C received: shutting down"),
    }
}

/// Resolves when stdin reaches end-of-file or fails.
///
/// A dedicated OS thread does the blocking read: a pending read on tokio's async stdin occupies a
/// runtime blocking thread and would keep the runtime from shutting down.
fn stdin_closed() -> impl Future<Output = ()> {
    let (closed_sender, closed_receiver) = oneshot::channel::<()>();
    let watcher = std::thread::Builder::new()
        .name("stdin-watch".to_owned())
        .spawn(move || {
            drain_until_closed(io::stdin().lock());
            let _ = closed_sender.send(());
        });
    if let Err(error) = watcher {
        // The sender was dropped with the closure, so the core stops right away: exiting is safer
        // than running without a way to notice that the parent died.
        tracing::error!(%error, "cannot start the stdin watcher thread");
    }
    async move {
        // `Err` means the watcher thread is gone, which is treated as "stdin closed" too.
        let _ = closed_receiver.await;
    }
}

/// Reads and discards input until end-of-file or a read error.
fn drain_until_closed(mut input: impl Read) {
    let mut buffer = [0_u8; 1024];
    loop {
        match input.read(&mut buffer) {
            Ok(0) => return,
            Err(error) if error.kind() != io::ErrorKind::Interrupted => return,
            Ok(_) | Err(_) => {}
        }
    }
}

async fn ctrl_c() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::warn!(%error, "cannot listen for Ctrl+C; only stdin closing will stop the core");
        std::future::pending::<()>().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draining_returns_at_end_of_file() {
        drain_until_closed(&b"anything the parent might write"[..]);
    }
}
