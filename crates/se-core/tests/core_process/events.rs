//! Broadcast of events to every connection.

use futures_util::StreamExt;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

use crate::running_core::{RunningCore, STARTUP_TIMEOUT, next_frame};

#[tokio::test]
async fn two_clients_both_receive_the_shutdown_event_then_a_close() {
    let mut core = RunningCore::start().await;
    let mut first = core.connect().await;
    let mut second = core.connect().await;

    let status = core.stop().await;
    assert!(
        status.success(),
        "se-core should exit cleanly, got {status}"
    );

    for socket in [&mut first, &mut second] {
        let event = next_frame(socket).await;
        assert_eq!(event["type"], "event");
        assert_eq!(event["event"]["kind"], "coreShuttingDown");
        let after = timeout(STARTUP_TIMEOUT, socket.next())
            .await
            .expect("the socket ends in time");
        assert!(
            matches!(after, None | Some(Ok(Message::Close(_)) | Err(_))),
            "expected the socket to close after coreShuttingDown, got {after:?}"
        );
    }
}
