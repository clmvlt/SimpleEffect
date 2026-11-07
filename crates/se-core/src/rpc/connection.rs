//! One `/rpc` connection: answers requests and forwards events until the client or the core stops.

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use se_protocol::CoreEvent;
use tokio::sync::{broadcast, mpsc, watch};
use tracing::Instrument;

use super::{RpcState, encode::event_frame, event_bus::event_to_forward};
use crate::CORE_VERSION;

/// Why the loop should go on or stop after handling one input.
enum Flow {
    Continue,
    Stop,
}

/// Serves one upgraded socket until it closes or the core shuts down.
pub async fn serve(socket: WebSocket, state: RpcState, connection_id: u64) {
    let span = tracing::info_span!("rpc_connection", id = connection_id);
    run(socket, state).instrument(span).await;
}

async fn run(mut socket: WebSocket, state: RpcState) {
    tracing::debug!("client connected");
    // Subscribe before greeting so no event published from now on is missed.
    let mut events = state.events.subscribe();
    let mut stopping = state.stopping.clone();
    let ready = CoreEvent::CoreReady {
        version: CORE_VERSION.to_owned(),
    };
    if send(&mut socket, event_frame(ready)).await {
        serve_until_done(&mut socket, &state, &mut events, &mut stopping).await;
    }
    // Best effort: the client sees a clean close frame instead of a reset.
    let _ = socket.send(Message::Close(None)).await;
    tracing::debug!("client disconnected");
}

async fn serve_until_done(
    socket: &mut WebSocket,
    state: &RpcState,
    events: &mut broadcast::Receiver<CoreEvent>,
    stopping: &mut watch::Receiver<bool>,
) {
    // Each request runs in its own task and sends its response frame here.
    let (replies, mut finished) = mpsc::unbounded_channel::<String>();
    loop {
        // `biased`: pending events (notably `coreShuttingDown`, published just before `stopping`
        // flips, and the patch a command published before answering) are forwarded first.
        let flow = tokio::select! {
            biased;
            received = events.recv() => forward_event(socket, event_to_forward(received)).await,
            Some(reply) = finished.recv() => {
                if send(socket, reply).await { Flow::Continue } else { Flow::Stop }
            }
            incoming = socket.recv() => match incoming {
                Some(Ok(message)) => answer(state, message, &replies),
                Some(Err(_)) | None => Flow::Stop,
            },
            () = stop_requested(stopping) => Flow::Stop,
        };
        if matches!(flow, Flow::Stop) {
            return;
        }
    }
}

/// Forwards one event; the connection ends after `coreShuttingDown` or once the bus is gone.
async fn forward_event(socket: &mut WebSocket, event: Option<CoreEvent>) -> Flow {
    let Some(event) = event else {
        return Flow::Stop;
    };
    if let CoreEvent::ResyncRequired { missed_events } = event {
        tracing::warn!(
            missed_events,
            "slow client dropped events; asking it to resync"
        );
    }
    let is_last = matches!(event, CoreEvent::CoreShuttingDown);
    if send(socket, event_frame(event)).await && !is_last {
        Flow::Continue
    } else {
        Flow::Stop
    }
}

/// Starts answering a text frame in its own task, so a slow command blocks nothing else.
fn answer(state: &RpcState, message: Message, replies: &mpsc::UnboundedSender<String>) -> Flow {
    match message {
        Message::Text(text) => {
            let router = Arc::clone(&state.router);
            let replies = replies.clone();
            let request = async move {
                let reply = router.handle_text(text.as_str()).await;
                // The connection may have closed meanwhile: the answer is then dropped.
                let _ = replies.send(reply);
            };
            tokio::spawn(request.instrument(tracing::Span::current()));
            Flow::Continue
        }
        Message::Close(_) => Flow::Stop,
        // Axum answers pings itself; binary frames are not part of the protocol.
        Message::Binary(_) | Message::Ping(_) | Message::Pong(_) => Flow::Continue,
    }
}

/// Sends a text frame; `false` means the socket is gone.
async fn send(socket: &mut WebSocket, frame: String) -> bool {
    socket.send(Message::Text(frame.into())).await.is_ok()
}

/// Resolves once shutdown starts (or the shutdown signal is gone).
///
/// Kept in its own function so the non-`Send` `watch::Ref` never lives across an `.await`.
async fn stop_requested(stopping: &mut watch::Receiver<bool>) {
    let _ = stopping.wait_for(|stopping| *stopping).await;
}
