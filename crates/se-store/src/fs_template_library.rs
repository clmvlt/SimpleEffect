//! [`FsTemplateLibrary`]: the template library in `<app data>/templates/<id>/` (prompt 25).
//!
//! Each entry is a folder named after its id holding `template.json` (the entry without its
//! module, plus `formatVersion`) and, for motion templates, `source.ts`. A `.setemplate` file is
//! a zip of the same two files.

use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use se_app::ports::{StorageError, TemplateLibrary};
use se_domain::{LibraryTemplateId, MAX_TEMPLATE_SOURCE_BYTES};
use se_protocol::LibraryTemplate;
use serde_json::Value;

use crate::{
    io_errors::{blocking, io_error},
    library_files::{corrupted, entry_folders, read_json, trash_folder, write_bytes, write_json},
};

/// Name of an entry's description.
const TEMPLATE_FILE: &str = "template.json";
/// Name of a motion template's module.
const SOURCE_FILE: &str = "source.ts";
/// Largest `template.json` read from an archive (parameters and a caption style fit easily).
const MAX_TEMPLATE_JSON_BYTES: u64 = 1_000_000;

/// The template library on disk.
#[derive(Debug, Clone)]
pub struct FsTemplateLibrary {
    root: PathBuf,
}

impl FsTemplateLibrary {
    /// A library in `root` (created on the first save).
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn folder(&self, id: LibraryTemplateId) -> PathBuf {
        self.root.join(id.to_string())
    }
}

/// The entry as stored: its JSON without the module, and the module apart.
fn split(template: &LibraryTemplate, path: &Path) -> Result<(Value, Option<String>), StorageError> {
    let mut value = serde_json::to_value(template).map_err(|error| corrupted(path, &error))?;
    let source = value
        .get_mut("content")
        .and_then(Value::as_object_mut)
        .and_then(|content| content.remove("source"))
        .and_then(|source| source.as_str().map(str::to_owned));
    Ok((value, source))
}

/// The entry of `json` (and its module), read strictly.
fn join(path: &Path, json: &str, source: Option<String>) -> Result<LibraryTemplate, StorageError> {
    let mut value = read_json(path, json)?;
    if let (Some(content), Some(source)) = (
        value.get_mut("content").and_then(Value::as_object_mut),
        source,
    ) {
        content.insert("source".to_owned(), Value::String(source));
    }
    serde_json::from_value(value).map_err(|error| corrupted(path, &error))
}

fn read_folder(folder: &Path) -> Result<Option<LibraryTemplate>, StorageError> {
    let json_path = folder.join(TEMPLATE_FILE);
    let json = match fs::read_to_string(&json_path) {
        Ok(json) => json,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_error("read", &json_path, &error)),
    };
    let source_path = folder.join(SOURCE_FILE);
    let source = match fs::read_to_string(&source_path) {
        Ok(source) => Some(source),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(io_error("read", &source_path, &error)),
    };
    join(&json_path, &json, source).map(Some)
}

fn write_folder(folder: &Path, template: &LibraryTemplate) -> Result<(), StorageError> {
    let json_path = folder.join(TEMPLATE_FILE);
    let (value, source) = split(template, &json_path)?;
    let source_path = folder.join(SOURCE_FILE);
    match source {
        Some(source) => write_bytes(&source_path, source.as_bytes())?,
        None => match fs::remove_file(&source_path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(io_error("remove", &source_path, &error)),
        },
    }
    // The description last: a folder without it is skipped, never half-read.
    write_json(&json_path, value)
}

fn write_zip(path: &Path, template: &LibraryTemplate) -> Result<(), StorageError> {
    let (mut value, source) = split(template, path)?;
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "formatVersion".to_owned(),
            Value::from(crate::library_files::LIBRARY_FORMAT_VERSION),
        );
    }
    let json = serde_json::to_vec_pretty(&value).map_err(|error| corrupted(path, &error))?;
    let zip_error =
        |error: zip::result::ZipError| io_error("write", path, &std::io::Error::other(error));
    let mut buffer = std::io::Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut buffer);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        writer
            .start_file(TEMPLATE_FILE, options)
            .map_err(zip_error)?;
        writer
            .write_all(&json)
            .map_err(|error| io_error("write", path, &error))?;
        if let Some(source) = source {
            writer.start_file(SOURCE_FILE, options).map_err(zip_error)?;
            writer
                .write_all(source.as_bytes())
                .map_err(|error| io_error("write", path, &error))?;
        }
        writer.finish().map_err(zip_error)?;
    }
    write_bytes(path, &buffer.into_inner())
}

fn read_zip(path: &Path) -> Result<LibraryTemplate, StorageError> {
    let not_archive = |reason: &str| {
        corrupted(
            path,
            &format!("not a SimpleEffect template archive (.setemplate): {reason}"),
        )
    };
    let file = fs::File::open(path).map_err(|error| io_error("read", path, &error))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|error| not_archive(&error.to_string()))?;
    let json = read_member(&mut archive, TEMPLATE_FILE, MAX_TEMPLATE_JSON_BYTES)
        .map_err(|reason| not_archive(&reason))?
        .ok_or_else(|| not_archive("template.json is missing"))?;
    let source_limit = u64::try_from(MAX_TEMPLATE_SOURCE_BYTES).unwrap_or(u64::MAX);
    let source = read_member(&mut archive, SOURCE_FILE, source_limit)
        .map_err(|reason| not_archive(&reason))?;
    join(path, &json, source)
}

/// The text of `name` in the archive (`None` when absent), refused beyond `limit` bytes.
fn read_member(
    archive: &mut zip::ZipArchive<fs::File>,
    name: &str,
    limit: u64,
) -> Result<Option<String>, String> {
    let member = match archive.by_name(name) {
        Ok(member) => member,
        Err(zip::result::ZipError::FileNotFound) => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let mut text = String::new();
    member
        .take(limit + 1)
        .read_to_string(&mut text)
        .map_err(|error| format!("{name} cannot be read: {error}"))?;
    if text.len() as u64 > limit {
        return Err(format!("{name} is larger than {limit} bytes"));
    }
    Ok(Some(text))
}

#[async_trait]
impl TemplateLibrary for FsTemplateLibrary {
    async fn list(&self) -> Result<Vec<LibraryTemplate>, StorageError> {
        let root = self.root.clone();
        blocking(&self.root, move || {
            let mut entries = Vec::new();
            for folder in entry_folders(&root)? {
                match read_folder(&folder) {
                    Ok(Some(entry)) => entries.push(entry),
                    Ok(None) => {}
                    Err(error) => {
                        tracing::warn!(%error, "a template library entry cannot be read; skipped");
                    }
                }
            }
            Ok(entries)
        })
        .await
    }

    async fn get(&self, id: LibraryTemplateId) -> Result<Option<LibraryTemplate>, StorageError> {
        let folder = self.folder(id);
        blocking(&self.root, move || read_folder(&folder)).await
    }

    async fn save(&self, template: &LibraryTemplate) -> Result<(), StorageError> {
        let folder = self.folder(template.id);
        let template = template.clone();
        blocking(&self.root, move || write_folder(&folder, &template)).await
    }

    async fn remove(&self, id: LibraryTemplateId) -> Result<(), StorageError> {
        let folder = self.folder(id);
        blocking(&self.root, move || trash_folder(&folder)).await
    }

    async fn write_archive(
        &self,
        template: &LibraryTemplate,
        path: &Path,
    ) -> Result<(), StorageError> {
        let template = template.clone();
        let target = path.to_path_buf();
        blocking(path, move || write_zip(&target, &template)).await
    }

    async fn read_archive(&self, path: &Path) -> Result<LibraryTemplate, StorageError> {
        let target = path.to_path_buf();
        blocking(path, move || read_zip(&target)).await
    }
}
