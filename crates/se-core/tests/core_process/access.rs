//! Handshake contents and access control (token and `Host` checks).

use tokio_tungstenite::tungstenite::{
    self,
    client::IntoClientRequest,
    http::{HeaderValue, StatusCode, header::HOST},
};

use crate::running_core::RunningCore;

fn rejection_status(error: tungstenite::Error) -> StatusCode {
    match error {
        tungstenite::Error::Http(response) => response.status(),
        other => panic!("expected an HTTP rejection, got {other:?}"),
    }
}

#[tokio::test]
async fn handshake_announces_the_package_version_and_a_256_bit_token() {
    let core = RunningCore::start().await;
    assert_eq!(core.version, env!("CARGO_PKG_VERSION"));
    assert_eq!(core.token.len(), 64);
    assert!(core.token.chars().all(|c| c.is_ascii_hexdigit()));
}

#[tokio::test]
async fn a_wrong_token_is_rejected_with_401() {
    let core = RunningCore::start().await;
    let error = tokio_tungstenite::connect_async(core.rpc_url(&"0".repeat(64)))
        .await
        .expect_err("a wrong token must be refused");
    assert_eq!(rejection_status(error), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_missing_token_is_rejected_with_401() {
    let core = RunningCore::start().await;
    let url = format!("ws://127.0.0.1:{}/rpc", core.port);
    let error = tokio_tungstenite::connect_async(url)
        .await
        .expect_err("a missing token must be refused");
    assert_eq!(rejection_status(error), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_foreign_host_header_is_rejected_with_403() {
    let core = RunningCore::start().await;
    let mut request = core
        .rpc_url(&core.token)
        .into_client_request()
        .expect("valid request");
    request.headers_mut().insert(
        HOST,
        HeaderValue::from_str(&format!("evil.example:{}", core.port)).expect("header"),
    );
    let error = tokio_tungstenite::connect_async(request)
        .await
        .expect_err("a foreign Host must be refused");
    assert_eq!(rejection_status(error), StatusCode::FORBIDDEN);
}
