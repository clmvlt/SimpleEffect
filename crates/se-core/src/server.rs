//! Binds the loopback listener, announces the handshake and serves the routes until shutdown.

use std::{net::Ipv4Addr, sync::Arc, time::Duration};

use axum::{Router, middleware, routing::get};
use se_protocol::{CoreEvent, CoreHandshake};
use tokio::{net::TcpListener, sync::watch};

use crate::{
    CORE_VERSION,
    access_guard::{AccessGuard, SessionToken, require_local_access},
    app_paths::AppPaths,
    error::CoreError,
    handshake,
    rpc::{self, EventBus, HandlerContext, RpcRouter, RpcState},
};

/// How long open connections get to close after shutdown starts before the process exits anyway.
const DRAIN_TIMEOUT: Duration = Duration::from_secs(2);

/// Runs the core server until `shutdown` resolves, then drains connections and returns.
pub async fn run(
    paths: AppPaths,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), CoreError> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .map_err(CoreError::Bind)?;
    let port = listener.local_addr().map_err(CoreError::Bind)?.port();
    let guard = Arc::new(AccessGuard::new(SessionToken::generate(), port));

    handshake::announce(&CoreHandshake {
        port,
        token: guard.token().as_str().to_owned(),
        version: CORE_VERSION.to_owned(),
    })?;
    tracing::info!(
        port,
        version = CORE_VERSION,
        data_dir = %paths.data_dir.display(),
        "se-core listening on 127.0.0.1"
    );

    let events = EventBus::new();
    let (stopping_sender, stopping) = watch::channel(false);
    let router = RpcRouter::new(HandlerContext {
        version: CORE_VERSION,
        paths,
    });
    let state = RpcState::new(router, events.clone(), stopping.clone());
    let server = axum::serve(listener, routes(guard, state)).with_graceful_shutdown(async move {
        shutdown.await;
        // Tell every client first, then close the sockets (see `rpc::connection`).
        events.publish(CoreEvent::CoreShuttingDown);
        stopping_sender.send_replace(true);
    });

    tokio::select! {
        result = server.into_future() => result.map_err(CoreError::Serve),
        () = drain_deadline(stopping) => {
            tracing::warn!("connections did not close within {DRAIN_TIMEOUT:?}; exiting anyway");
            Ok(())
        }
    }
}

fn routes(guard: Arc<AccessGuard>, state: RpcState) -> Router {
    Router::new()
        .route("/rpc", get(rpc::upgrade))
        .with_state(state)
        .layer(middleware::from_fn_with_state(guard, require_local_access))
}

/// Resolves `DRAIN_TIMEOUT` after shutdown starts, bounding the graceful shutdown.
async fn drain_deadline(mut stopping: watch::Receiver<bool>) {
    if stopping.wait_for(|stopping| *stopping).await.is_err() {
        // The sender is gone only once the server future has finished: let that branch win.
        std::future::pending::<()>().await;
    }
    tokio::time::sleep(DRAIN_TIMEOUT).await;
}
