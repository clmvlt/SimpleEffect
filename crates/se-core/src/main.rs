//! # se-core
//!
//! The SimpleEffect core process, spawned and supervised by the Electron main process
//! (see `docs/adr/0001-electron-shell-rust-core-process.md`).
//!
//! At startup it binds `127.0.0.1` on a random port, generates a 256-bit session token and prints
//! **one** JSON handshake line on stdout (`{"port":…,"token":"…","version":"…"}`); logs go to daily
//! files in `<app data>/logs/` (and stderr in debug builds). It serves `/rpc` (WebSocket) and later
//! `/media` and `/mcp`.
//!
//! It stops gracefully when its stdin closes (the parent process died or asked it to stop) and on
//! Ctrl+C, so it can never outlive the app as a zombie.
//!
//! This crate is the composition root: it may depend on every workspace crate and nothing depends
//! on it. It contains wiring only, no domain logic.

mod access_guard;
mod app_paths;
mod brand_kit_routes;
mod error;
mod font_routes;
mod handshake;
mod logging;
mod mcp_routes;
mod media_cors;
mod media_routes;
mod platform;
mod render_routes;
mod rpc;
mod server;
mod services;
mod shutdown;
mod system_ports;

/// Version of this binary, reported in the handshake, `coreReady` and `getCoreInfo`.
const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let paths = app_paths::AppPaths::from_environment()?;
    let _log_guard = logging::init(&paths.logs_dir);
    platform::tie_children_to_this_process();
    // Before any HTTP client exists (model downloads use rustls with the ring provider).
    se_transcribe::install_crypto_provider();
    server::run(paths, shutdown::requested()).await?;
    tracing::info!("se-core stopped");
    Ok(())
}
