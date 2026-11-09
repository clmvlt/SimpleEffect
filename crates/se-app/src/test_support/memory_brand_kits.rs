//! [`InMemoryBrandKitLibrary`]: brand kits and their logo copies kept in memory.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use se_domain::BrandKitId;
use se_protocol::LibraryBrandKit;

use super::{FakeMedia, lock, probed_image};
use crate::ports::{BrandKitLibrary, StorageError};

/// Folder the fake pretends to keep kits in.
const KITS_DIR: &str = "/app-data/brand-kits";

#[derive(Debug, Default)]
struct State {
    kits: BTreeMap<BrandKitId, LibraryBrandKit>,
    logos: BTreeSet<(BrandKitId, String)>,
    copies: Vec<PathBuf>,
    removed: Vec<BrandKitId>,
}

/// Kits by id and the logo files copied into them. Logo copies into projects are added to
/// `files` (when given) as 512 × 512 PNGs, each destination with its own content (a copy made
/// again to the same place keeps it), so imports find them. Clones share the state.
#[derive(Debug, Clone, Default)]
pub struct InMemoryBrandKitLibrary {
    state: Arc<Mutex<State>>,
    files: Option<Arc<FakeMedia>>,
}

/// Content number of the first logo copy (each copy gets the next one).
const FIRST_LOGO_CONTENT: u64 = 9_001;

impl InMemoryBrandKitLibrary {
    /// A library whose copies into projects appear in `files`.
    #[must_use]
    pub fn writing_to(files: Arc<FakeMedia>) -> Self {
        Self {
            state: Arc::default(),
            files: Some(files),
        }
    }

    /// Logo files held by kit `kit`.
    #[must_use]
    pub fn logo_files(&self, kit: BrandKitId) -> Vec<String> {
        lock(&self.state)
            .logos
            .iter()
            .filter(|(owner, _)| *owner == kit)
            .map(|(_, file)| file.clone())
            .collect()
    }

    /// Destinations of logo copies into projects, in order (each once).
    #[must_use]
    pub fn copies(&self) -> Vec<PathBuf> {
        lock(&self.state).copies.clone()
    }

    /// Kits moved to the trash, in order.
    #[must_use]
    pub fn removed(&self) -> Vec<BrandKitId> {
        lock(&self.state).removed.clone()
    }
}

#[async_trait]
impl BrandKitLibrary for InMemoryBrandKitLibrary {
    async fn list(&self) -> Result<Vec<LibraryBrandKit>, StorageError> {
        Ok(lock(&self.state).kits.values().cloned().collect())
    }

    async fn get(&self, id: BrandKitId) -> Result<Option<LibraryBrandKit>, StorageError> {
        Ok(lock(&self.state).kits.get(&id).cloned())
    }

    async fn save(&self, kit: &LibraryBrandKit) -> Result<(), StorageError> {
        let mut state = lock(&self.state);
        state.kits.insert(kit.id, kit.clone());
        let listed: BTreeSet<&str> = kit.logos.iter().map(|logo| logo.file.as_str()).collect();
        state
            .logos
            .retain(|(owner, file)| *owner != kit.id || listed.contains(file.as_str()));
        Ok(())
    }

    async fn remove(&self, id: BrandKitId) -> Result<(), StorageError> {
        let mut state = lock(&self.state);
        if state.kits.remove(&id).is_some() {
            state.removed.push(id);
        }
        Ok(())
    }

    async fn import_logo(
        &self,
        kit: BrandKitId,
        _source: &Path,
        file_name: &str,
    ) -> Result<(), StorageError> {
        lock(&self.state).logos.insert((kit, file_name.to_owned()));
        Ok(())
    }

    fn logo_path(&self, kit: BrandKitId, file_name: &str) -> PathBuf {
        Path::new(KITS_DIR).join(kit.to_string()).join(file_name)
    }

    async fn copy_logo(
        &self,
        kit: BrandKitId,
        file_name: &str,
        destination: &Path,
    ) -> Result<(), StorageError> {
        let mut state = lock(&self.state);
        if !state.logos.contains(&(kit, file_name.to_owned())) {
            return Err(StorageError::Corrupted {
                path: self.logo_path(kit, file_name),
                reason: "the logo file is missing".to_owned(),
            });
        }
        let index = state
            .copies
            .iter()
            .position(|copy| copy == destination)
            .unwrap_or(state.copies.len());
        if index == state.copies.len() {
            state.copies.push(destination.to_path_buf());
        }
        if let Some(files) = &self.files {
            files.add(
                destination,
                FIRST_LOGO_CONTENT + index as u64,
                probed_image(),
            );
        }
        Ok(())
    }
}
