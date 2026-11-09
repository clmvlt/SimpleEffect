//! [`BrandKitLibrary`]: the app's brand kits on disk (prompt 25), implemented by se-store
//! (`FsBrandKitLibrary`, `<app data>/brand-kits/<id>/`).

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use se_domain::BrandKitId;
use se_protocol::LibraryBrandKit;

use super::StorageError;

/// Stores brand kits and copies of their logo files.
#[async_trait]
pub trait BrandKitLibrary: Send + Sync {
    /// Every readable kit (unreadable folders are skipped and logged), in no particular order.
    ///
    /// # Errors
    /// The library folder cannot be read.
    async fn list(&self) -> Result<Vec<LibraryBrandKit>, StorageError>;

    /// The kit `id`, if the library has it.
    ///
    /// # Errors
    /// Its file exists but cannot be read.
    async fn get(&self, id: BrandKitId) -> Result<Option<LibraryBrandKit>, StorageError>;

    /// Writes `kit` (replacing the kit with its id) atomically, then deletes the logo copies of
    /// its folder it no longer lists.
    ///
    /// # Errors
    /// IO errors.
    async fn save(&self, kit: &LibraryBrandKit) -> Result<(), StorageError>;

    /// Moves the kit's folder to the system trash (nothing happens when it does not exist).
    ///
    /// # Errors
    /// IO errors.
    async fn remove(&self, id: BrandKitId) -> Result<(), StorageError>;

    /// Copies the image at `source` into the kit's folder under `file_name` (a relative name
    /// such as `logos/<uuid>.png` chosen by the caller), creating the folder if needed.
    ///
    /// # Errors
    /// The source cannot be read or the copy written.
    async fn import_logo(
        &self,
        kit: BrandKitId,
        source: &Path,
        file_name: &str,
    ) -> Result<(), StorageError>;

    /// Absolute path of a logo file of a kit (it may not exist).
    fn logo_path(&self, kit: BrandKitId, file_name: &str) -> PathBuf;

    /// Copies a logo file of a kit to `destination` (a file path, its folder created), keeping an
    /// identical file already there.
    ///
    /// # Errors
    /// The logo is missing or the copy cannot be written.
    async fn copy_logo(
        &self,
        kit: BrandKitId,
        file_name: &str,
        destination: &Path,
    ) -> Result<(), StorageError>;
}
