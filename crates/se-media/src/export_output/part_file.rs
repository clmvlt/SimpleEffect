//! [`PartFile`]: a partial file of an export, written at positions the renderer chooses (muxers
//! go back to patch headers and sizes).

use std::{
    io::SeekFrom,
    path::{Path, PathBuf},
};

use se_app::ports::MediaError;
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt},
};

/// A partial file being written.
pub struct PartFile {
    path: PathBuf,
    file: File,
}

impl PartFile {
    /// Creates (or empties) the file at `path`.
    ///
    /// # Errors
    /// [`MediaError::Io`] when the folder cannot be written.
    pub async fn create(path: PathBuf) -> Result<Self, MediaError> {
        let file = File::create(&path)
            .await
            .map_err(|error| io_error(&path, &error))?;
        Ok(Self { path, file })
    }

    /// Writes `bytes` at `position` (the file grows as needed).
    ///
    /// # Errors
    /// [`MediaError::Io`] (disk full, removed folder…).
    pub async fn write_at(&mut self, position: u64, bytes: &[u8]) -> Result<(), MediaError> {
        self.file
            .seek(SeekFrom::Start(position))
            .await
            .map_err(|error| io_error(&self.path, &error))?;
        self.file
            .write_all(bytes)
            .await
            .map_err(|error| io_error(&self.path, &error))
    }

    /// Completes pending writes and closes the file; returns its path.
    ///
    /// # Errors
    /// [`MediaError::Io`] when a pending write failed.
    pub async fn close(mut self) -> Result<PathBuf, MediaError> {
        self.file
            .flush()
            .await
            .map_err(|error| io_error(&self.path, &error))?;
        Ok(self.path)
    }
}

/// An IO failure on an export file, as users and agents read it.
pub fn io_error(path: &Path, error: &std::io::Error) -> MediaError {
    MediaError::Io {
        path: path.to_path_buf(),
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn writes_land_at_their_position() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("film.mp4.part");
        let mut file = PartFile::create(path.clone()).await.unwrap();
        file.write_at(4, b"tail").await.unwrap();
        file.write_at(0, b"head").await.unwrap();
        file.write_at(2, b"AD").await.unwrap();
        let closed = file.close().await.unwrap();
        assert_eq!(std::fs::read(closed).unwrap(), b"heADtail");
    }
}
