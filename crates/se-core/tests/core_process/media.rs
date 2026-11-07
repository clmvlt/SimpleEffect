//! Media end to end: the `/media` routes behind the token, and (when ffmpeg is installed) an
//! import whose thumbnails and peaks are generated and served with Range support.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::running_core::{RpcSocket, RunningCore, call, next_json};

/// A raw HTTP response.
struct HttpResponse {
    status: u16,
    head: String,
    body: Vec<u8>,
}

impl HttpResponse {
    fn header(&self, name: &str) -> Option<&str> {
        self.head.lines().find_map(|line| {
            let (key, value) = line.split_once(':')?;
            key.eq_ignore_ascii_case(name).then(|| value.trim())
        })
    }
}

/// `GET path` on the core with an optional `Range` header (HTTP/1.1, connection closed after).
async fn http_get(port: u16, path: &str, range: Option<&str>) -> HttpResponse {
    let mut stream = TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("connect");
    let range = range
        .map(|range| format!("Range: {range}\r\n"))
        .unwrap_or_default();
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n{range}\r\n"
    );
    stream.write_all(request.as_bytes()).await.expect("send");
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await.expect("read");
    let split = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("an HTTP head");
    let head = String::from_utf8_lossy(&raw[..split]).into_owned();
    let status = head
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .expect("a status code");
    HttpResponse {
        status,
        head,
        body: raw[split + 4..].to_vec(),
    }
}

/// The repository's ffmpeg (`pnpm fetch-binaries`), if installed.
fn repository_ffmpeg() -> Option<PathBuf> {
    let name = if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../resources/bin")
        .join(se_media::PLATFORM_DIR)
        .join(name);
    path.is_file().then_some(path)
}

/// Runs the repository's ffmpeg with `args` (lavfi sources) writing `output`.
fn generate(ffmpeg: &Path, args: &[&str], output: &Path) {
    let status = Command::new(ffmpeg)
        .args(["-hide_banner", "-v", "error", "-y"])
        .args(args)
        .arg(output)
        .status()
        .expect("run ffmpeg");
    assert!(status.success());
}

/// Creates a project in `parent` and returns its id and folder path.
async fn create_project(socket: &mut RpcSocket, parent: &Path) -> (Value, Value) {
    let (created, _) = call(
        socket,
        1,
        json!({
            "kind": "createProject", "name": "Film", "parentDir": parent.to_string_lossy(),
            "format": { "width": 1920, "height": 1080, "frameRate": { "numerator": 25, "denominator": 1 }, "sampleRate": 48_000 }
        }),
    )
    .await;
    let result = &created["result"];
    (
        result["snapshot"]["project"]["id"].clone(),
        result["path"].clone(),
    )
}

#[tokio::test]
async fn media_routes_need_the_token_and_an_open_asset() {
    let core = RunningCore::start().await;
    let path = "/media/assets/00000000-0001-7000-8000-000000000000/original";
    assert_eq!(http_get(core.port, path, None).await.status, 401);
    let wrong = format!("{path}?t={}", "0".repeat(64));
    assert_eq!(http_get(core.port, &wrong, None).await.status, 401);
    let unknown = http_get(core.port, &format!("{path}?t={}", core.token), None).await;
    assert_eq!(unknown.status, 404);
}

#[tokio::test]
async fn imported_media_get_thumbnails_and_peaks_served_by_media_routes() {
    let Some(ffmpeg) = repository_ffmpeg() else {
        eprintln!("skipped: ffmpeg not installed (run `pnpm fetch-binaries`)");
        return;
    };
    let folder = tempfile::tempdir().expect("folder");
    let clip = folder.path().join("clip.mp4");
    let sources = [
        "-f",
        "lavfi",
        "-i",
        "testsrc2=size=640x360:rate=25",
        "-f",
        "lavfi",
        "-i",
        "sine=sample_rate=48000",
        "-t",
        "2",
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        "-c:a",
        "aac",
        "-shortest",
    ];
    generate(&ffmpeg, &sources, &clip);

    let mut core = RunningCore::start().await;
    let mut socket = core.connect().await;
    let (project_id, _) = create_project(&mut socket, folder.path()).await;
    let (imported, mut events) = call(
        &mut socket,
        2,
        json!({ "kind": "importMedia", "projectId": project_id, "paths": [clip.to_string_lossy()] }),
    )
    .await;
    assert_eq!(imported["result"]["kind"], "importMedia", "{imported}");
    let asset_id = imported["result"]["imported"][0]
        .as_str()
        .expect("an asset")
        .to_owned();

    // Thumbnails and peaks complete (events may arrive before or after the response; analyses
    // run too and may finish first).
    let completed = |events: &[Value]| {
        events
            .iter()
            .filter(|message| {
                let kind = &message["event"]["job"]["kind"];
                message["event"]["kind"] == "jobCompleted"
                    && (kind == "thumbnails" || kind == "waveformPeaks")
            })
            .count()
    };
    while completed(&events) < 2 {
        let message = next_json(&mut socket).await;
        assert_ne!(message["event"]["kind"], "jobFailed", "{message}");
        events.push(message);
    }

    let token = &core.token;
    let base = format!("/media/assets/{asset_id}");
    let sheet = http_get(core.port, &format!("{base}/thumbnails?t={token}"), None).await;
    assert_eq!(sheet.status, 200);
    assert_eq!(sheet.header("content-type"), Some("application/json"));
    let layout: Value = serde_json::from_slice(&sheet.body).expect("JSON");
    assert_eq!(layout["tileHeight"], 90);
    let image = http_get(
        core.port,
        &format!("{base}/thumbnails/sheet?t={token}"),
        None,
    )
    .await;
    assert_eq!(image.header("content-type"), Some("image/jpeg"));
    let peaks = http_get(core.port, &format!("{base}/peaks/1024?t={token}"), None).await;
    assert_eq!(&peaks.body[..4], b"SEPK");

    let part = http_get(
        core.port,
        &format!("{base}/original?t={token}"),
        Some("bytes=0-99"),
    )
    .await;
    assert_eq!(part.status, 206);
    assert_eq!(part.body.len(), 100);
    assert!(
        part.header("content-range")
            .expect("a range")
            .starts_with("bytes 0-99/")
    );
    let beyond = http_get(
        core.port,
        &format!("{base}/original?t={token}"),
        Some("bytes=99999999-"),
    )
    .await;
    assert_eq!(beyond.status, 416);
    // A graceful stop cancels the jobs and waits for ffmpeg: killing the core in the middle of a
    // spawn could leave a suspended child behind (backlog, Media).
    core.stop().await;
}

#[tokio::test]
async fn a_renamed_original_is_missing_on_reopening_until_relinked() {
    let Some(ffmpeg) = repository_ffmpeg() else {
        eprintln!("skipped: ffmpeg not installed (run `pnpm fetch-binaries`)");
        return;
    };
    let folder = tempfile::tempdir().expect("folder");
    let sound = folder.path().join("voice.wav");
    generate(&ffmpeg, &["-f", "lavfi", "-i", "sine=duration=1"], &sound);

    let mut core = RunningCore::start().await;
    let mut socket = core.connect().await;
    let (project_id, project_path) = create_project(&mut socket, folder.path()).await;
    let (imported, _) = call(
        &mut socket,
        2,
        json!({ "kind": "importMedia", "projectId": project_id, "paths": [sound.to_string_lossy()] }),
    )
    .await;
    let asset_id = imported["result"]["imported"][0].clone();
    let (closed, _) = call(
        &mut socket,
        3,
        json!({ "kind": "closeProject", "projectId": project_id }),
    )
    .await;
    assert_eq!(closed["result"]["kind"], "closeProject", "{closed}");

    let renamed = folder.path().join("voice (renamed).wav");
    std::fs::rename(&sound, &renamed).expect("rename");
    let (reopened, _) = call(
        &mut socket,
        4,
        json!({ "kind": "openProject", "path": project_path }),
    )
    .await;
    assert_eq!(
        reopened["result"]["snapshot"]["assets"][0]["missing"], true,
        "{reopened}"
    );

    let (relinked, _) = call(
        &mut socket,
        5,
        json!({
            "kind": "relinkAsset", "projectId": project_id, "assetId": asset_id,
            "path": renamed.to_string_lossy(), "force": false
        }),
    )
    .await;
    assert_eq!(relinked["result"]["outcome"], "relinked", "{relinked}");
    let (snapshot, _) = call(
        &mut socket,
        6,
        json!({ "kind": "getProjectSnapshot", "projectId": project_id }),
    )
    .await;
    let asset = &snapshot["result"]["snapshot"]["assets"][0];
    assert_eq!(asset["missing"], false);
    assert_eq!(asset["originalPath"], renamed.to_string_lossy().as_ref());
    core.stop().await;
}
