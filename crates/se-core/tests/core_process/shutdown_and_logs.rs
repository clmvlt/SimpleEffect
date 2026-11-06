//! Shutdown when stdin closes, and log files in `<app data>/logs/`.

use std::fs;

use crate::running_core::RunningCore;

#[tokio::test]
async fn closing_stdin_stops_the_core() {
    let mut core = RunningCore::start().await;
    let status = core.stop().await;
    assert!(
        status.success(),
        "se-core should exit cleanly, got {status}"
    );
}

#[tokio::test]
async fn closing_stdin_with_an_open_socket_still_stops_the_core() {
    let mut core = RunningCore::start().await;
    let _socket = core.connect().await;
    let status = core.stop().await;
    assert!(
        status.success(),
        "open sockets must not keep se-core alive, got {status}"
    );
}

#[tokio::test]
async fn logs_are_written_to_a_daily_file_without_the_token() {
    let mut core = RunningCore::start().await;
    let _socket = core.connect().await;
    // Stopping flushes the background log writer.
    core.stop().await;

    let logs_dir = core.data_dir().join("logs");
    let files: Vec<_> = fs::read_dir(&logs_dir)
        .expect("the logs folder exists")
        .map(|entry| entry.expect("readable entry").path())
        .collect();
    assert_eq!(files.len(), 1, "one log file per day, got {files:?}");
    let name = files[0]
        .file_name()
        .expect("file name")
        .to_string_lossy()
        .into_owned();
    assert!(
        name.starts_with("se-core.") && files[0].extension().is_some_and(|ext| ext == "log"),
        "unexpected log file name {name}"
    );
    let contents = fs::read_to_string(&files[0]).expect("readable log file");
    assert!(
        contents.contains("se-core listening on 127.0.0.1"),
        "{contents}"
    );
    assert!(contents.contains("client connected"), "{contents}");
    assert!(
        !contents.contains(&core.token),
        "the session token must never be logged"
    );
}
