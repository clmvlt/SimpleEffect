//! [`InMemoryTemplateLibrary`]: the template library and `.setemplate` files kept in memory.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use se_domain::LibraryTemplateId;
use se_protocol::LibraryTemplate;

use super::lock;
use crate::ports::{StorageError, TemplateLibrary};

#[derive(Debug, Default)]
struct State {
    entries: BTreeMap<LibraryTemplateId, LibraryTemplate>,
    archives: BTreeMap<PathBuf, LibraryTemplate>,
    removed: Vec<LibraryTemplateId>,
}

/// Library entries by id and archive files by path. Reading an unknown archive fails like a
/// file that is not a template archive. Clones share the state.
#[derive(Debug, Clone, Default)]
pub struct InMemoryTemplateLibrary {
    state: Arc<Mutex<State>>,
}

impl InMemoryTemplateLibrary {
    /// Places an archive holding `entry` at `path`.
    pub fn put_archive(&self, path: impl Into<PathBuf>, entry: LibraryTemplate) {
        lock(&self.state).archives.insert(path.into(), entry);
    }

    /// The entry written in the archive at `path`, if any.
    #[must_use]
    pub fn archive(&self, path: impl AsRef<Path>) -> Option<LibraryTemplate> {
        lock(&self.state).archives.get(path.as_ref()).cloned()
    }

    /// Entries moved to the trash, in order.
    #[must_use]
    pub fn removed(&self) -> Vec<LibraryTemplateId> {
        lock(&self.state).removed.clone()
    }
}

#[async_trait]
impl TemplateLibrary for InMemoryTemplateLibrary {
    async fn list(&self) -> Result<Vec<LibraryTemplate>, StorageError> {
        Ok(lock(&self.state).entries.values().cloned().collect())
    }

    async fn get(&self, id: LibraryTemplateId) -> Result<Option<LibraryTemplate>, StorageError> {
        Ok(lock(&self.state).entries.get(&id).cloned())
    }

    async fn save(&self, template: &LibraryTemplate) -> Result<(), StorageError> {
        lock(&self.state)
            .entries
            .insert(template.id, template.clone());
        Ok(())
    }

    async fn remove(&self, id: LibraryTemplateId) -> Result<(), StorageError> {
        let mut state = lock(&self.state);
        if state.entries.remove(&id).is_some() {
            state.removed.push(id);
        }
        Ok(())
    }

    async fn write_archive(
        &self,
        template: &LibraryTemplate,
        path: &Path,
    ) -> Result<(), StorageError> {
        self.put_archive(path, template.clone());
        Ok(())
    }

    async fn read_archive(&self, path: &Path) -> Result<LibraryTemplate, StorageError> {
        self.archive(path).ok_or_else(|| StorageError::Corrupted {
            path: path.to_path_buf(),
            reason: "not a SimpleEffect template archive (.setemplate)".to_owned(),
        })
    }
}
