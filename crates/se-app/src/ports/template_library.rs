//! [`TemplateLibrary`]: the app's template library on disk (prompt 25), implemented by se-store
//! (`FsTemplateLibrary`, `<app data>/templates/<id>/`).

use std::path::Path;

use async_trait::async_trait;
use se_domain::LibraryTemplateId;
use se_protocol::LibraryTemplate;

use super::StorageError;

/// Stores motion templates and caption styles reused across projects, and reads and writes
/// their `.setemplate` exchange files.
#[async_trait]
pub trait TemplateLibrary: Send + Sync {
    /// Every readable entry (unreadable folders are skipped and logged), in no particular order.
    ///
    /// # Errors
    /// The library folder cannot be read.
    async fn list(&self) -> Result<Vec<LibraryTemplate>, StorageError>;

    /// The entry `id`, if the library has it.
    ///
    /// # Errors
    /// Its files exist but cannot be read.
    async fn get(&self, id: LibraryTemplateId) -> Result<Option<LibraryTemplate>, StorageError>;

    /// Writes `template` (replacing the entry with its id), atomically.
    ///
    /// # Errors
    /// IO errors.
    async fn save(&self, template: &LibraryTemplate) -> Result<(), StorageError>;

    /// Moves the entry to the system trash (nothing happens when it does not exist).
    ///
    /// # Errors
    /// IO errors.
    async fn remove(&self, id: LibraryTemplateId) -> Result<(), StorageError>;

    /// Writes `template` as a `.setemplate` file (a zip of `template.json` and `source.ts`) at
    /// `path`, replacing it atomically.
    ///
    /// # Errors
    /// IO errors.
    async fn write_archive(
        &self,
        template: &LibraryTemplate,
        path: &Path,
    ) -> Result<(), StorageError>;

    /// Reads the entry held by the `.setemplate` file at `path`. Only its shape is checked here;
    /// the caller checks its content (the module through the renderer).
    ///
    /// # Errors
    /// [`StorageError::Corrupted`] for a file that is not a template archive (with the reason),
    /// [`StorageError::SchemaTooNew`] for one written by a newer SimpleEffect, IO errors.
    async fn read_archive(&self, path: &Path) -> Result<LibraryTemplate, StorageError>;
}
