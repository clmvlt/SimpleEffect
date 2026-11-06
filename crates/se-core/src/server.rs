//! Binds the loopback listener, announces the handshake and serves the routes until shutdown.

use std::{net::Ipv4Addr, sync::Arc, time::Duration};

use axum::{Router, middleware, routing::get};
use se_protocol::CoreHandshake;
use tokio::{net::TcpListener, sync::watch};

use crate::{
    CORE_VERSION,
    access_guard::{AccessGuard, SessionToken, require_local_access},
    error::CoreError,
    handshake,
    rpc::{self, RpcState},
};

/// How long open connections get to close after shutdown starts before the process exits anyway.
const DRAIN_TIMEOUT: Duration = Duration::from_secs(2);

/// Runs the core server until `shutdown` resolves, then drains connections and returns.
pub async fn run(shutdown: impl Future<Output = ()> + Send + 'static) -> Result<(), CoreError> {
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
        "se-core listening on 127.0.0.1"
    );

    let (stopping_sender, stopping) = watch::channel(false);
    let server =
        axum::serve(listener, router(guard, stopping.clone())).with_graceful_shutdown(async move {
            shutdown.await;
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

fn router(guard: Arc<AccessGuard>, stopping: watch::Receiver<bool>) -> Router {
    Router::new()
        .route("/rpc", get(rpc::upgrade))
        .with_state(RpcState { stopping })
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
