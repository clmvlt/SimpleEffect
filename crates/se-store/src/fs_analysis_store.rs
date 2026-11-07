//! [`FsAnalysisStore`]: the `AnalysisStore` port over an open project's `project.db`.

use std::{
    path::PathBuf,
    sync::{Arc, Mutex, PoisonError},
};

use async_trait::async_trait;
use se_app::ports::{AnalysisStore, StorageError, StoredAnalysis};
use se_domain::{AnalysisKind, AssetId};

use crate::{io_errors::blocking, project_db::ProjectDatabase};

/// The analyses of one open project, sharing its database connection with its store.
pub struct FsAnalysisStore {
    root: PathBuf,
    database: Arc<Mutex<ProjectDatabase>>,
}

impl FsAnalysisStore {
    /// The analyses of the project at `root`, read and written through `database`.
    pub fn new(root: PathBuf, database: Arc<Mutex<ProjectDatabase>>) -> Self {
        Self { root, database }
    }
}

#[async_trait]
impl AnalysisStore for FsAnalysisStore {
    async fn save_analysis(&self, analysis: &StoredAnalysis) -> Result<(), StorageError> {
        let (database, analysis) = (Arc::clone(&self.database), analysis.clone());
        blocking(&self.root, move || {
            database
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .save_analysis(&analysis)
        })
        .await
    }

    async fn load_analysis(
        &self,
        asset_id: AssetId,
        kind: AnalysisKind,
    ) -> Result<Option<StoredAnalysis>, StorageError> {
        let database = Arc::clone(&self.database);
        blocking(&self.root, move || {
            database
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .load_analysis(asset_id, kind)
        })
        .await
    }
}
