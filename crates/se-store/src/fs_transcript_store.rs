//! [`FsTranscriptStore`]: the `TranscriptStore` port over an open project's `project.db`.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex, PoisonError},
};

use async_trait::async_trait;
use se_app::ports::{StorageError, StoredTranscript, TranscriptHit, TranscriptStore};
use se_domain::AssetId;

use crate::{io_errors::blocking, project_db::ProjectDatabase};

/// The transcripts of one open project, sharing its database connection with its store.
pub struct FsTranscriptStore {
    root: PathBuf,
    database: Arc<Mutex<ProjectDatabase>>,
}

impl FsTranscriptStore {
    /// The transcripts of the project at `root`, read and written through `database`.
    pub fn new(root: PathBuf, database: Arc<Mutex<ProjectDatabase>>) -> Self {
        Self { root, database }
    }
}

#[async_trait]
impl TranscriptStore for FsTranscriptStore {
    async fn save_transcript(&self, transcript: &StoredTranscript) -> Result<(), StorageError> {
        let (database, transcript) = (Arc::clone(&self.database), transcript.clone());
        blocking(&self.root, move || {
            database
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .save_transcript(&transcript)
        })
        .await
    }

    async fn load_transcript(
        &self,
        asset_id: AssetId,
    ) -> Result<Option<StoredTranscript>, StorageError> {
        let database = Arc::clone(&self.database);
        blocking(&self.root, move || {
            database
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .load_transcript(asset_id)
        })
        .await
    }

    async fn search_transcripts(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<TranscriptHit>, StorageError> {
        let (database, query) = (Arc::clone(&self.database), query.to_owned());
        blocking(&self.root, move || {
            database
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .search_transcripts(&query, limit)
        })
        .await
    }
}
