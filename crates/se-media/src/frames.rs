//! Extracting one picture of a media file as a JPEG (the `FrameExtractor` port). Pictures are
//! shown upright (ffmpeg applies the display rotation) and never enlarged.

use std::path::{Path, PathBuf};

use se_app::ports::{FrameRequest, MediaError};
use se_domain::AssetKind;

use crate::process::Sidecar;

/// JPEG quality (`-q:v`, 2 best … 31 worst).
const JPEG_QUALITY: &str = "3";

/// Writes the picture of `request` with the ffmpeg at `ffmpeg`. A time at the very end of a
/// video (past its last frame) falls back to the last frame.
///
/// # Errors
/// [`MediaError`] when ffmpeg fails or produces nothing.
pub async fn extract(ffmpeg: &Path, request: &FrameRequest) -> Result<(), MediaError> {
    let io = |error: std::io::Error| MediaError::Io {
        path: request.output.clone(),
        message: error.to_string(),
    };
    if let Some(dir) = request.output.parent() {
        tokio::fs::create_dir_all(dir).await.map_err(io)?;
    }
    let temporary = temporary_jpeg(&request.output);
    let seek = (request.asset_kind == AssetKind::Video).then(|| {
        vec![
            "-ss".to_owned(),
            format!("{:.3}", request.time.to_seconds_f64()),
        ]
    });
    let first = run(ffmpeg, request, seek.unwrap_or_default(), &temporary).await;
    if !exists(&temporary).await && request.asset_kind == AssetKind::Video {
        // Past the last frame: decode the last second, keeping (overwriting with) each frame.
        let last_second = vec!["-sseof".to_owned(), "-1".to_owned()];
        run(ffmpeg, request, last_second, &temporary).await?;
    } else {
        first?;
    }
    if !exists(&temporary).await {
        return Err(MediaError::Failed(format!(
            "ffmpeg found no picture in {} at {:.3} s",
            request.input.display(),
            request.time.to_seconds_f64()
        )));
    }
    tokio::fs::rename(&temporary, &request.output)
        .await
        .map_err(io)
}

async fn run(
    ffmpeg: &Path,
    request: &FrameRequest,
    seek: Vec<String>,
    output: &Path,
) -> Result<(), MediaError> {
    let mut args: Vec<String> = ["-hide_banner", "-nostdin", "-v", "error", "-y"]
        .map(str::to_owned)
        .to_vec();
    let to_the_end = seek.first().is_some_and(|option| option == "-sseof");
    args.extend(seek);
    args.push("-i".to_owned());
    args.push(request.input.to_string_lossy().into_owned());
    args.extend(["-map", "0:v:0", "-an", "-sn", "-dn", "-update", "1"].map(str::to_owned));
    if !to_the_end {
        args.extend(["-frames:v", "1"].map(str::to_owned));
    }
    args.push("-vf".to_owned());
    args.push(format!("scale='min({},iw)':-2", request.max_width));
    args.extend(["-q:v", JPEG_QUALITY].map(str::to_owned));
    args.push(output.to_string_lossy().into_owned());
    Sidecar::spawn(ffmpeg, &args, None)?.finish().await
}

async fn exists(path: &Path) -> bool {
    tokio::fs::metadata(path)
        .await
        .is_ok_and(|metadata| metadata.len() > 0)
}

/// `<dir>/<stem>.part.jpg`.
fn temporary_jpeg(output: &Path) -> PathBuf {
    let stem = output.file_stem().unwrap_or_default().to_string_lossy();
    output.with_file_name(format!("{stem}.part.jpg"))
}
