//! End-to-end tests of the real `se-core` binary: handshake, `/rpc` ping, access control and
//! shutdown when stdin closes. Each test spawns its own core on a random port.

// Test helpers outside #[test] functions must also fail loudly on unexpected states.
#![allow(clippy::expect_used, clippy::panic)]

use std::{process::Stdio, time::Duration};

use futures_util::{SinkExt, StreamExt};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    time::timeout,
};
use tokio_tungstenite::tungstenite::{
    self, Message,
    client::IntoClientRequest,
    http::{HeaderValue, StatusCode, header::HOST},
};

const STARTUP_TIMEOUT: Duration = Duration::from_secs(20);
const EXIT_TIMEOUT: Duration = Duration::from_secs(10);

/// A running core and the connection details it announced.
struct RunningCore {
    child: Child,
    port: u16,
    token: String,
    version: String,
}

impl RunningCore {
    async fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_se-core"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .expect("se-core should spawn");
        let stdout = child.stdout.take().expect("stdout is piped");
        let line = timeout(STARTUP_TIMEOUT, BufReader::new(stdout).lines().next_line())
            .await
            .expect("handshake within the startup timeout")
            .expect("stdout is readable")
            .expect("se-core prints a handshake line");
        let handshake: serde_json::Value =
            serde_json::from_str(&line).expect("the handshake line is JSON");
        Self {
            child,
            port: u16::try_from(handshake["port"].as_u64().expect("port")).expect("u16 port"),
            token: handshake["token"].as_str().expect("token").to_owned(),
            version: handshake["version"].as_str().expect("version").to_owned(),
        }
    }

    fn rpc_url(&self, token: &str) -> String {
        format!("ws://127.0.0.1:{}/rpc?t={token}", self.port)
    }
}

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
async fn ping_is_answered_with_pong_and_the_core_version() {
    let core = RunningCore::start().await;
    let (mut socket, _) = tokio_tungstenite::connect_async(core.rpc_url(&core.token))
        .await
        .expect("an authorised client can connect");

    socket
        .send(Message::text(
            r#"{"type":"request","id":1,"command":{"kind":"ping"}}"#,
        ))
        .await
        .expect("send ping");
    let reply = timeout(STARTUP_TIMEOUT, socket.next())
        .await
        .expect("reply in time")
        .expect("socket open")
        .expect("valid frame");

    let expected = format!(
        r#"{{"type":"response","id":1,"result":{{"kind":"pong","version":"{}"}}}}"#,
        core.version
    );
    assert_eq!(reply.into_text().expect("text frame").as_str(), expected);
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

#[tokio::test]
async fn closing_stdin_stops_the_core() {
    let mut core = RunningCore::start().await;
    drop(core.child.stdin.take());
    let status = timeout(EXIT_TIMEOUT, core.child.wait())
        .await
        .expect("se-core exits after stdin closes")
        .expect("exit status");
    assert!(
        status.success(),
        "se-core should exit cleanly, got {status}"
    );
}

#[tokio::test]
async fn closing_stdin_with_an_open_socket_still_stops_the_core() {
    let mut core = RunningCore::start().await;
    let (_socket, _) = tokio_tungstenite::connect_async(core.rpc_url(&core.token))
        .await
        .expect("connect");
    drop(core.child.stdin.take());
    let status = timeout(EXIT_TIMEOUT, core.child.wait())
        .await
        .expect("open sockets must not keep se-core alive")
        .expect("exit status");
    assert!(
        status.success(),
        "se-core should exit cleanly, got {status}"
    );
}
