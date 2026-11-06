//! The `/rpc` WebSocket endpoint: typed requests in, responses and pushed events out
//! (`docs/architecture.md` §6, ADR-0012).
//!
//! - [`router`]: one text frame in, one response frame out (parse, dispatch, encode).
//! - [`handlers`]: one file per command group, called by the router.
//! - [`event_bus`]: fan-out of [`se_protocol::CoreEvent`]s to every connection.
//! - [`connection`]: the per-socket loop interleaving responses and events.

mod connection;
mod encode;
mod event_bus;
mod handlers;
mod request;
mod router;

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
};
pub use event_bus::EventBus;
pub use handlers::HandlerContext;
pub use router::RpcRouter;
use tokio::sync::watch;

/// State shared by every `/rpc` connection.
#[derive(Clone)]
pub struct RpcState {
    router: Arc<RpcRouter>,
    events: EventBus,
    stopping: watch::Receiver<bool>,
    next_connection_id: Arc<AtomicU64>,
}

impl RpcState {
    /// `stopping` becomes `true` when the core starts shutting down, so open sockets close.
    pub fn new(router: RpcRouter, events: EventBus, stopping: watch::Receiver<bool>) -> Self {
        Self {
            router: Arc::new(router),
            events,
            stopping,
            next_connection_id: Arc::new(AtomicU64::new(1)),
        }
    }
}

/// Upgrades an authorised HTTP request (see `access_guard`) to a WebSocket.
pub async fn upgrade(websocket: WebSocketUpgrade, State(state): State<RpcState>) -> Response {
    let connection_id = state.next_connection_id.fetch_add(1, Ordering::Relaxed);
    websocket.on_upgrade(move |socket| connection::serve(socket, state, connection_id))
}
