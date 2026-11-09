//! [`FsBrandKitLibrary`]: brand kits in `<app data>/brand-kits/<id>/` (prompt 25).
//!
//! Each kit is a folder named after its id holding `brand-kit.json` (the kit plus
//! `formatVersion`) and its logo copies under `logos/`. Saving a kit deletes the logo copies it
//! no longer lists (they are the app's own copies; the user's originals are never touched).

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use se_app::{
    brand_kits::is_logo_file_name,
    ports::{BrandKitLibrary, StorageError},
};
use se_domain::BrandKitId;
use se_protocol::LibraryBrandKit;

use crate::{
    io_errors::{blocking, io_error},
    library_files::{corrupted, entry_folders, read_json, trash_folder, write_json},
};

/// Name of a kit's description.
const KIT_FILE: &str = "brand-kit.json";
/// Folder of a kit's logo copies.
const LOGO_FOLDER: &str = "logos";

/// The brand kits on disk.
#[derive(Debug, Clone)]
pub struct FsBrandKitLibrary {
    root: PathBuf,
}

impl FsBrandKitLibrary {
    /// Kits in `root` (created on the first save).
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn folder(&self, id: BrandKitId) -> PathBuf {
        self.root.join(id.to_string())
    }
}

fn read_kit(folder: &Path) -> Result<Option<LibraryBrandKit>, StorageError> {
    let path = folder.join(KIT_FILE);
    let json = match fs::read_to_string(&path) {
        Ok(json) => json,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_error("read", &path, &error)),
    };
    let value = read_json(&path, &json)?;
    serde_json::from_value(value)
        .map(Some)
        .map_err(|error| corrupted(&path, &error))
}

/// Deletes the files of `folder/logos` that `kit` does not list.
fn prune_logos(folder: &Path, kit: &LibraryBrandKit) -> Result<(), StorageError> {
    let logos = folder.join(LOGO_FOLDER);
    let listed: BTreeSet<&str> = kit.logos.iter().map(|logo| logo.file.as_str()).collect();
    let entries = match fs::read_dir(&logos) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(io_error("list", &logos, &error)),
    };
    for entry in entries {
        let entry = entry.map_err(|error| io_error("list", &logos, &error))?;
        let name = format!("{LOGO_FOLDER}/{}", entry.file_name().to_string_lossy());
        if !listed.contains(name.as_str()) && entry.file_type().is_ok_and(|kind| kind.is_file()) {
            fs::remove_file(entry.path())
                .map_err(|error| io_error("remove", &entry.path(), &error))?;
        }
    }
    Ok(())
}

fn checked_name(folder: &Path, file_name: &str) -> Result<PathBuf, StorageError> {
    if is_logo_file_name(file_name) {
        Ok(folder.join(file_name))
    } else {
        Err(corrupted(
            &folder.join(KIT_FILE),
            &format!("'{file_name}' is not a logo file name of a kit"),
        ))
    }
}

/// Whether the files at `left` and `right` hold the same bytes.
fn same_file(left: &Path, right: &Path) -> bool {
    match (fs::metadata(left), fs::metadata(right)) {
        (Ok(a), Ok(b)) if a.len() == b.len() => fs::read(left).ok() == fs::read(right).ok(),
        _ => false,
    }
}

#[async_trait]
impl BrandKitLibrary for FsBrandKitLibrary {
    async fn list(&self) -> Result<Vec<LibraryBrandKit>, StorageError> {
        let root = self.root.clone();
        blocking(&self.root, move || {
            let mut kits = Vec::new();
            for folder in entry_folders(&root)? {
                match read_kit(&folder) {
                    Ok(Some(kit)) => kits.push(kit),
                    Ok(None) => {}
                    Err(error) => tracing::warn!(%error, "a brand kit cannot be read; skipped"),
                }
            }
            Ok(kits)
        })
        .await
    }

    async fn get(&self, id: BrandKitId) -> Result<Option<LibraryBrandKit>, StorageError> {
        let folder = self.folder(id);
        blocking(&self.root, move || read_kit(&folder)).await
    }

    async fn save(&self, kit: &LibraryBrandKit) -> Result<(), StorageError> {
        let folder = self.folder(kit.id);
        let kit = kit.clone();
        blocking(&self.root, move || {
            let path = folder.join(KIT_FILE);
            let value = serde_json::to_value(&kit).map_err(|error| corrupted(&path, &error))?;
            write_json(&path, value)?;
            prune_logos(&folder, &kit)
        })
        .await
    }

    async fn remove(&self, id: BrandKitId) -> Result<(), StorageError> {
        let folder = self.folder(id);
        blocking(&self.root, move || trash_folder(&folder)).await
    }

    async fn import_logo(
        &self,
        kit: BrandKitId,
        source: &Path,
        file_name: &str,
    ) -> Result<(), StorageError> {
        let target = checked_name(&self.folder(kit), file_name)?;
        let source = source.to_path_buf();
        blocking(&self.root, move || {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|error| io_error("create", parent, &error))?;
            }
            fs::copy(&source, &target)
                .map(drop)
                .map_err(|error| io_error("copy", &source, &error))
        })
        .await
    }

    fn logo_path(&self, kit: BrandKitId, file_name: &str) -> PathBuf {
        self.folder(kit).join(file_name)
    }

    async fn copy_logo(
        &self,
        kit: BrandKitId,
        file_name: &str,
        destination: &Path,
    ) -> Result<(), StorageError> {
        let source = checked_name(&self.folder(kit), file_name)?;
        let destination = destination.to_path_buf();
        blocking(&self.root, move || {
            if same_file(&source, &destination) {
                return Ok(());
            }
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|error| io_error("create", parent, &error))?;
            }
            fs::copy(&source, &destination)
                .map(drop)
                .map_err(|error| io_error("copy", &source, &error))
        })
        .await
    }
}
