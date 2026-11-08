//! [`HttpDownloader`]: the `FileDownloader` port over HTTPS (reqwest + rustls). It runs only when
//! a use case asks for a file the user agreed to download; nothing else in the app reaches the
//! network (ADR-0031).
//!
//! A download writes `<name>.part`, resumes it with a `Range` request when it exists, checks the
//! size and the SHA-256, then renames it. A mismatch deletes the part.

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use async_trait::async_trait;
use reqwest::{StatusCode, header};
use se_app::ports::{DownloadError, DownloadSink, DownloadSpec, FileDownloader};
use tokio::io::AsyncWriteExt;

use crate::hashing::sha256_file;

/// How long the server may stay silent before the download fails.
const READ_TIMEOUT: Duration = Duration::from_secs(60);

/// Downloads files over HTTPS.
#[derive(Debug, Clone)]
pub struct HttpDownloader {
    client: reqwest::Client,
}

/// Makes rustls's ring provider the process's default (once; a provider already installed is
/// kept). Every reqwest client of the process needs one (the workspace enables rustls without a
/// built-in provider), so the core calls this at startup, before any client exists.
pub fn install_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

impl HttpDownloader {
    /// A downloader with the system's certificate verifier.
    ///
    /// # Errors
    /// A message when the TLS client cannot be built.
    pub fn new() -> Result<Self, String> {
        install_crypto_provider();
        let client = reqwest::Client::builder()
            .user_agent(concat!("SimpleEffect/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(20))
            .read_timeout(READ_TIMEOUT)
            .build()
            .map_err(|error| format!("the HTTPS client cannot start: {error}"))?;
        Ok(Self { client })
    }
}

/// `<file>.part` next to `destination`.
#[must_use]
pub fn part_path(destination: &Path) -> PathBuf {
    let mut name = destination.file_name().unwrap_or_default().to_os_string();
    name.push(".part");
    destination.with_file_name(name)
}

#[async_trait]
impl FileDownloader for HttpDownloader {
    async fn download(
        &self,
        spec: &DownloadSpec,
        progress: &dyn DownloadSink,
    ) -> Result<(), DownloadError> {
        let part = part_path(&spec.destination);
        if let Some(dir) = spec.destination.parent() {
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(|error| io(dir, &error))?;
        }
        let mut have = tokio::fs::metadata(&part)
            .await
            .map_or(0, |metadata| metadata.len());
        if have > spec.size {
            tokio::fs::remove_file(&part)
                .await
                .map_err(|error| io(&part, &error))?;
            have = 0;
        }
        if have < spec.size {
            self.fetch(spec, &part, have, progress).await?;
        }
        progress.received(spec.size, spec.size);
        progress.verifying();
        finish(spec, &part).await
    }
}

impl HttpDownloader {
    /// Appends the missing bytes to `part` (all of them when the server ignores the range).
    async fn fetch(
        &self,
        spec: &DownloadSpec,
        part: &Path,
        have: u64,
        progress: &dyn DownloadSink,
    ) -> Result<(), DownloadError> {
        let network = |error: reqwest::Error| DownloadError::Network {
            url: spec.url.clone(),
            message: error.to_string(),
        };
        let mut request = self.client.get(&spec.url);
        if have > 0 {
            request = request.header(header::RANGE, format!("bytes={have}-"));
        }
        let mut response = request.send().await.map_err(network)?;
        let resumed = response.status() == StatusCode::PARTIAL_CONTENT;
        if !resumed && !response.status().is_success() {
            return Err(DownloadError::Network {
                url: spec.url.clone(),
                message: format!("the server answered {}", response.status()),
            });
        }
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(resumed)
            .truncate(!resumed)
            .open(part)
            .await
            .map_err(|error| io(part, &error))?;
        let mut received = if resumed { have } else { 0 };
        while let Some(chunk) = response.chunk().await.map_err(network)? {
            received += chunk.len() as u64;
            if received > spec.size {
                drop(file);
                let _ = tokio::fs::remove_file(part).await;
                return Err(DownloadError::Mismatch {
                    url: spec.url.clone(),
                });
            }
            file.write_all(&chunk)
                .await
                .map_err(|error| io(part, &error))?;
            progress.received(received, spec.size);
        }
        file.flush().await.map_err(|error| io(part, &error))?;
        file.sync_all().await.map_err(|error| io(part, &error))?;
        Ok(())
    }
}

/// Checks the part's size and SHA-256, then renames it (or deletes it on a mismatch).
async fn finish(spec: &DownloadSpec, part: &Path) -> Result<(), DownloadError> {
    let size = tokio::fs::metadata(part)
        .await
        .map_err(|error| io(part, &error))?
        .len();
    let digest = sha256_file(part).await.map_err(DownloadError::Io)?;
    if size != spec.size || !digest.eq_ignore_ascii_case(&spec.sha256) {
        let _ = tokio::fs::remove_file(part).await;
        return Err(DownloadError::Mismatch {
            url: spec.url.clone(),
        });
    }
    tokio::fs::rename(part, &spec.destination)
        .await
        .map_err(|error| io(&spec.destination, &error))
}

fn io(path: &Path, error: &std::io::Error) -> DownloadError {
    DownloadError::Io(format!("{} cannot be written: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    #[derive(Default)]
    struct Recorded(Mutex<Vec<(u64, u64)>>, Mutex<bool>);

    impl DownloadSink for Recorded {
        fn received(&self, received: u64, total: u64) {
            self.0.lock().unwrap().push((received, total));
        }

        fn verifying(&self) {
            *self.1.lock().unwrap() = true;
        }
    }

    fn spec(dir: &Path, content: &[u8]) -> DownloadSpec {
        use sha2::Digest;
        DownloadSpec {
            url: "https://example.invalid/model.bin".to_owned(),
            sha256: hex::encode(sha2::Sha256::digest(content)),
            size: content.len() as u64,
            destination: dir.join("model.bin"),
        }
    }

    #[tokio::test]
    async fn a_complete_part_is_checked_and_installed_without_the_network() {
        let dir = tempfile::tempdir().unwrap();
        let spec = spec(dir.path(), b"ggml model bytes");
        std::fs::write(part_path(&spec.destination), b"ggml model bytes").unwrap();
        let sink = Recorded::default();
        HttpDownloader::new()
            .unwrap()
            .download(&spec, &sink)
            .await
            .unwrap();
        assert_eq!(
            std::fs::read(&spec.destination).unwrap(),
            b"ggml model bytes"
        );
        assert!(!part_path(&spec.destination).exists());
        assert!(*sink.1.lock().unwrap());
    }

    #[tokio::test]
    async fn a_part_with_other_content_is_deleted_as_a_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let spec = spec(dir.path(), b"expected bytes!!");
        std::fs::write(part_path(&spec.destination), b"tampered bytes!!").unwrap();
        let error = HttpDownloader::new()
            .unwrap()
            .download(&spec, &Recorded::default())
            .await
            .unwrap_err();
        assert!(matches!(error, DownloadError::Mismatch { .. }));
        assert!(!part_path(&spec.destination).exists() && !spec.destination.exists());
    }
}
