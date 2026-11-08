//! Installing and removing the GPU pack (Windows: the official CUDA build of whisper.cpp): the
//! archive is downloaded into `<runtimes>/downloads/` (resumable, SHA-256 checked), the needed
//! files are unpacked into `<runtimes>/<pack>.part/`, which is then renamed to `<runtimes>/<pack>/`
//! (a folder that exists is complete), and the archive is deleted.

use std::path::{Path, PathBuf};

use se_app::ports::{DownloadError, DownloadSink, DownloadSpec, FileDownloader};

use crate::{
    platform::{GpuPack, archive_tool},
    process::{run_logged, tail},
};

/// Folder of the installed pack.
#[must_use]
pub fn pack_dir(runtimes_dir: &Path, pack: &GpuPack) -> PathBuf {
    runtimes_dir.join(pack.dir_name)
}

/// Downloads and unpacks `pack` into `runtimes_dir`.
///
/// # Errors
/// [`DownloadError`] of the download, or [`DownloadError::Io`] when unpacking fails.
pub async fn install(
    downloader: &dyn FileDownloader,
    runtimes_dir: &Path,
    pack: &GpuPack,
    progress: &dyn DownloadSink,
) -> Result<(), DownloadError> {
    let archive_name = pack.url.rsplit('/').next().unwrap_or("gpu-pack.zip");
    let archive = runtimes_dir.join("downloads").join(archive_name);
    let spec = DownloadSpec {
        url: pack.url.to_owned(),
        sha256: pack.sha256.to_owned(),
        size: pack.size,
        destination: archive.clone(),
    };
    downloader.download(&spec, progress).await?;
    let staging = runtimes_dir.join(format!("{}.part", pack.dir_name));
    remove_dir(&staging).await?;
    tokio::fs::create_dir_all(&staging)
        .await
        .map_err(|error| io(&staging, &error))?;
    unpack(&archive, &staging, pack).await?;
    let unpacked = staging.join(pack.archive_dir);
    let target = pack_dir(runtimes_dir, pack);
    remove_dir(&target).await?;
    tokio::fs::rename(&unpacked, &target)
        .await
        .map_err(|error| io(&target, &error))?;
    remove_dir(&staging).await?;
    let _ = tokio::fs::remove_file(&archive).await;
    Ok(())
}

/// Deletes the installed pack (nothing when absent).
///
/// # Errors
/// [`DownloadError::Io`].
pub async fn remove(runtimes_dir: &Path, pack: &GpuPack) -> Result<(), DownloadError> {
    remove_dir(&pack_dir(runtimes_dir, pack)).await
}

/// Extracts the pack's files with the system's bsdtar (it reads zip archives).
async fn unpack(archive: &Path, into: &Path, pack: &GpuPack) -> Result<(), DownloadError> {
    let mut args = vec![
        "-xf".to_owned(),
        archive.to_string_lossy().into_owned(),
        "-C".to_owned(),
        into.to_string_lossy().into_owned(),
    ];
    args.extend(
        pack.files
            .iter()
            .map(|file| format!("{}/{file}", pack.archive_dir)),
    );
    let tool = archive_tool();
    let finished = run_logged(&tool, &args, None, &mut |_| {})
        .await
        .map_err(DownloadError::Io)?;
    if !finished.success {
        return Err(DownloadError::Io(format!(
            "the GPU pack could not be unpacked: {}",
            tail(&finished.log, 3)
        )));
    }
    for file in pack.files {
        let path = into.join(pack.archive_dir).join(file);
        if !tokio::fs::try_exists(&path).await.unwrap_or(false) {
            return Err(DownloadError::Io(format!(
                "the GPU pack lacks {file}: the archive is not the expected one"
            )));
        }
    }
    Ok(())
}

async fn remove_dir(dir: &Path) -> Result<(), DownloadError> {
    match tokio::fs::remove_dir_all(dir).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io(dir, &error)),
    }
}

fn io(path: &Path, error: &std::io::Error) -> DownloadError {
    DownloadError::Io(format!(
        "{} cannot be changed: {error} (close programs using it and try again)",
        path.display()
    ))
}
