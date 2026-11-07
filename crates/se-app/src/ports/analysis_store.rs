//! [`AnalysisStore`]: the analyses of a project's assets, kept in its database (`analyses` table
//! of `project.db`, implemented by se-store).

use async_trait::async_trait;
use se_domain::{AnalysisKind, AnalysisResult, AssetId};

use super::StorageError;

/// One stored analysis of an asset.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredAnalysis {
    /// The asset analysed.
    pub asset_id: AssetId,
    /// Cache key of the content analysed (`ContentFingerprint::cache_key`): a result whose key
    /// differs from the asset's current one is stale (the asset was relinked to other content).
    pub content_key: String,
    /// When it was computed, in milliseconds since the Unix epoch.
    pub created_at: i64,
    /// The result.
    pub result: AnalysisResult,
}

/// Analyses of one open project. Shared by the project's actor handle and its jobs; it is not
/// document state, so it is written outside the actor.
#[async_trait]
pub trait AnalysisStore: Send + Sync {
    /// Saves `analysis`, replacing the previous result of the same asset and kind.
    ///
    /// # Errors
    /// IO errors.
    async fn save_analysis(&self, analysis: &StoredAnalysis) -> Result<(), StorageError>;

    /// The stored result of `kind` for `asset_id`, if any.
    ///
    /// # Errors
    /// [`StorageError::Corrupted`] for an unreadable row; IO errors.
    async fn load_analysis(
        &self,
        asset_id: AssetId,
        kind: AnalysisKind,
    ) -> Result<Option<StoredAnalysis>, StorageError>;
}
