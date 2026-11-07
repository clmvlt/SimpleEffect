//! Spawns `se-core`, reads its handshake and talks to `/rpc`.

use std::{
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
    time::Duration,
};

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tempfile::TempDir;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpStream,
    process::{Child, Command},
    time::timeout,
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};

/// Generous bound for process start and message delivery on slow CI machines.
pub const STARTUP_TIMEOUT: Duration = Duration::from_secs(20);
/// Bound for the core to exit after its stdin closes.
pub const EXIT_TIMEOUT: Duration = Duration::from_secs(10);

/// A client socket on `/rpc`.
pub type RpcSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// The data folder of a test core: its own temporary folder, or one shared between cores.
enum DataDir {
    Owned(TempDir),
    Shared(PathBuf),
}

impl DataDir {
    fn path(&self) -> &Path {
        match self {
            Self::Owned(folder) => folder.path(),
            Self::Shared(path) => path,
        }
    }
}

/// A running core and the connection details it announced.
pub struct RunningCore {
    pub child: Child,
    pub port: u16,
    pub token: String,
    pub version: String,
    data_dir: DataDir,
}

impl RunningCore {
    /// Spawns the core with a fresh data folder and waits for its handshake.
    pub async fn start() -> Self {
        let data_dir = tempfile::tempdir().expect("temporary data folder");
        Self::spawn(DataDir::Owned(data_dir)).await
    }

    /// Spawns the core on an existing data folder (e.g. to restart after a crash).
    pub async fn start_in(data_dir: &Path) -> Self {
        Self::spawn(DataDir::Shared(data_dir.to_path_buf())).await
    }

    async fn spawn(data_dir: DataDir) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_se-core"))
            .env("SE_DATA_DIR", data_dir.path())
            .env("SE_LOG", "debug")
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
        let handshake: Value = serde_json::from_str(&line).expect("the handshake line is JSON");
        Self {
            child,
            port: u16::try_from(handshake["port"].as_u64().expect("port")).expect("u16 port"),
            token: handshake["token"].as_str().expect("token").to_owned(),
            version: handshake["version"].as_str().expect("version").to_owned(),
            data_dir,
        }
    }

    /// The data folder passed as `SE_DATA_DIR`.
    pub fn data_dir(&self) -> &Path {
        self.data_dir.path()
    }

    /// `/rpc` URL carrying `token`.
    pub fn rpc_url(&self, token: &str) -> String {
        format!("ws://127.0.0.1:{}/rpc?t={token}", self.port)
    }

    /// Connects an authorised client and checks the core greets it with `coreReady`.
    pub async fn connect(&self) -> RpcSocket {
        let (mut socket, _) = tokio_tungstenite::connect_async(self.rpc_url(&self.token))
            .await
            .expect("an authorised client can connect");
        let greeting = next_json(&mut socket).await;
        assert_eq!(greeting["type"], "event");
        assert_eq!(greeting["event"]["kind"], "coreReady");
        assert_eq!(greeting["event"]["version"], self.version.as_str());
        socket
    }

    /// Closes the core's stdin (what the Electron supervisor does) and waits for it to exit.
    pub async fn stop(&mut self) -> ExitStatus {
        drop(self.child.stdin.take());
        timeout(EXIT_TIMEOUT, self.child.wait())
            .await
            .expect("se-core exits after stdin closes")
            .expect("exit status")
    }
}

/// Sends a request and returns its response frame, skipping and collecting the events pushed
/// before it.
pub async fn call(socket: &mut RpcSocket, id: u64, command: Value) -> (Value, Vec<Value>) {
    let frame = serde_json::json!({ "type": "request", "id": id, "command": command });
    socket
        .send(Message::text(frame.to_string()))
        .await
        .expect("send request");
    let mut events = Vec::new();
    loop {
        let message = next_json(socket).await;
        if message["type"] == "response" && message["id"] == id {
            return (message, events);
        }
        events.push(message);
    }
}

/// Sends a request and returns the response frame as JSON.
pub async fn request(socket: &mut RpcSocket, id: u64, command: Value) -> Value {
    let frame = serde_json::json!({ "type": "request", "id": id, "command": command });
    socket
        .send(Message::text(frame.to_string()))
        .await
        .expect("send request");
    next_json(socket).await
}

/// The next text frame, parsed as JSON.
pub async fn next_json(socket: &mut RpcSocket) -> Value {
    let text = next_text(socket).await;
    serde_json::from_str(&text).expect("frames are JSON")
}

/// The next text frame, as sent.
pub async fn next_text(socket: &mut RpcSocket) -> String {
    let frame = timeout(STARTUP_TIMEOUT, socket.next())
        .await
        .expect("a frame in time")
        .expect("socket open")
        .expect("valid frame");
    frame.into_text().expect("text frame").as_str().to_owned()
}
