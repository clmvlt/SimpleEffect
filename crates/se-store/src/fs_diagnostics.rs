//! [`FsDiagnosticsFiles`]: the core's log files read from disk and diagnostics archives written
//! as zip files (prompt 30). Masking is se-app's job: this adapter only reads and writes.

use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

use async_trait::async_trait;
use se_app::ports::{ArchiveEntry, DiagnosticsFiles, LogFile, StorageError};

use crate::{
    io_errors::{blocking, io_error},
    library_files::write_bytes,
};

/// Reads logs and writes zip archives on the local disk.
#[derive(Debug, Clone, Copy, Default)]
pub struct FsDiagnosticsFiles;

#[async_trait]
impl DiagnosticsFiles for FsDiagnosticsFiles {
    async fn recent_logs(
        &self,
        dir: &Path,
        max_files: usize,
        max_bytes: usize,
    ) -> Result<Vec<LogFile>, StorageError> {
        let dir = dir.to_path_buf();
        blocking(&dir.clone(), move || {
            read_recent_logs(&dir, max_files, max_bytes)
        })
        .await
    }

    async fn write_archive(
        &self,
        path: &Path,
        entries: Vec<ArchiveEntry>,
    ) -> Result<u64, StorageError> {
        let target = path.to_path_buf();
        blocking(path, move || write_zip(&target, &entries)).await
    }
}

fn read_recent_logs(
    dir: &Path,
    max_files: usize,
    max_bytes: usize,
) -> Result<Vec<LogFile>, StorageError> {
    let listing = match fs::read_dir(dir) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(io_error("list", dir, &error)),
    };
    let mut files: Vec<(SystemTime, PathBuf)> = listing
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .filter_map(|entry| {
            let modified = entry.metadata().and_then(|meta| meta.modified()).ok()?;
            Some((modified, entry.path()))
        })
        .collect();
    files.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| right.1.cmp(&left.1)));
    files
        .into_iter()
        .take(max_files)
        .map(|(_, path)| read_tail(&path, max_bytes))
        .collect()
}

/// The last `max_bytes` of `path` as lossy UTF-8.
fn read_tail(path: &Path, max_bytes: usize) -> Result<LogFile, StorageError> {
    let mut file = fs::File::open(path).map_err(|error| io_error("read", path, &error))?;
    let length = file
        .metadata()
        .map_err(|error| io_error("read", path, &error))?
        .len();
    let keep = u64::try_from(max_bytes).unwrap_or(u64::MAX).min(length);
    file.seek(SeekFrom::Start(length - keep))
        .map_err(|error| io_error("read", path, &error))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| io_error("read", path, &error))?;
    Ok(LogFile {
        name: path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default(),
        text: String::from_utf8_lossy(&bytes).into_owned(),
    })
}

fn write_zip(path: &Path, entries: &[ArchiveEntry]) -> Result<u64, StorageError> {
    let zip_error =
        |error: zip::result::ZipError| io_error("write", path, &std::io::Error::other(error));
    let mut buffer = std::io::Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut buffer);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for entry in entries {
            writer
                .start_file(entry.name.as_str(), options)
                .map_err(zip_error)?;
            writer
                .write_all(&entry.bytes)
                .map_err(|error| io_error("write", path, &error))?;
        }
        writer.finish().map_err(zip_error)?;
    }
    let bytes = buffer.into_inner();
    write_bytes(path, &bytes)?;
    Ok(bytes.len() as u64)
}
