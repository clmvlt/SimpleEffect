//! The `/rpc` WebSocket endpoint: JSON requests in, JSON responses out.

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use tokio::sync::watch;

use crate::dispatch;

/// State shared by every `/rpc` connection.
#[derive(Clone)]
pub struct RpcState {
    /// Becomes `true` when the core starts shutting down, so open sockets close cleanly.
    pub stopping: watch::Receiver<bool>,
}

/// Upgrades an authorised HTTP request (see `access_guard`) to a WebSocket.
pub async fn upgrade(websocket: WebSocketUpgrade, State(state): State<RpcState>) -> Response {
    websocket.on_upgrade(move |socket| serve(socket, state.stopping))
}

async fn serve(mut socket: WebSocket, mut stopping: watch::Receiver<bool>) {
    tracing::debug!("rpc client connected");
    loop {
        tokio::select! {
            incoming = socket.recv() => {
                let Some(Ok(message)) = incoming else { break };
                if !answer(&mut socket, message).await {
                    break;
                }
            }
            () = stop_requested(&mut stopping) => {
                // Best effort: the client sees a clean close frame instead of a reset.
                let _ = socket.send(Message::Close(None)).await;
                break;
            }
        }
    }
    tracing::debug!("rpc client disconnected");
}

/// Resolves once shutdown starts (or the shutdown signal is gone).
///
/// Kept in its own function so the non-`Send` `watch::Ref` never lives across an `.await`.
async fn stop_requested(stopping: &mut watch::Receiver<bool>) {
    let _ = stopping.wait_for(|stopping| *stopping).await;
}

/// Answers one frame. Returns `false` when the connection must end.
async fn answer(socket: &mut WebSocket, message: Message) -> bool {
    match message {
        Message::Text(text) => {
            let reply = dispatch::handle_text(text.as_str());
            socket.send(Message::Text(reply.into())).await.is_ok()
        }
        Message::Close(_) => false,
        // Axum answers pings itself; binary frames are not part of the protocol yet.
        Message::Binary(_) | Message::Ping(_) | Message::Pong(_) => true,
    }
}
