//! Requests and responses over the real socket.

use futures_util::SinkExt;
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;

use crate::running_core::{RunningCore, next_json, next_text, request};

#[tokio::test]
async fn ping_is_answered_in_the_documented_shape() {
    let core = RunningCore::start().await;
    let mut socket = core.connect().await;
    socket
        .send(Message::text(
            r#"{"type":"request","id":1,"command":{"kind":"ping"}}"#,
        ))
        .await
        .expect("send ping");
    assert_eq!(
        next_text(&mut socket).await,
        r#"{"type":"response","id":1,"result":{"kind":"ping"}}"#
    );
}

#[tokio::test]
async fn get_core_info_reports_the_version_and_the_data_folders() {
    let core = RunningCore::start().await;
    let mut socket = core.connect().await;
    let reply = request(&mut socket, 2, json!({ "kind": "getCoreInfo" })).await;
    let info = &reply["result"];
    assert_eq!(reply["id"], 2);
    assert_eq!(info["kind"], "getCoreInfo");
    assert_eq!(info["version"], core.version.as_str());
    assert_eq!(info["os"], std::env::consts::OS);
    let data_dir = core.data_dir().to_string_lossy().into_owned();
    assert_eq!(info["appDataDir"], data_dir.as_str());
    let logs_dir = core.data_dir().join("logs").to_string_lossy().into_owned();
    assert_eq!(info["logsDir"], logs_dir.as_str());
}

#[tokio::test]
async fn unknown_commands_and_invalid_json_get_stable_error_codes() {
    let core = RunningCore::start().await;
    let mut socket = core.connect().await;

    let unknown = request(&mut socket, 3, json!({ "kind": "explode" })).await;
    assert_eq!(unknown["id"], 3);
    assert_eq!(unknown["error"]["code"], "UNKNOWN_COMMAND");

    socket
        .send(Message::text("{not json"))
        .await
        .expect("send garbage");
    let invalid = next_json(&mut socket).await;
    assert_eq!(invalid["id"], serde_json::Value::Null);
    assert_eq!(invalid["error"]["code"], "INVALID_REQUEST");

    // The connection survives bad requests.
    let pong = request(&mut socket, 4, json!({ "kind": "ping" })).await;
    assert_eq!(pong["result"]["kind"], "ping");
}
