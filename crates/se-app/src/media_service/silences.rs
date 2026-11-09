//! Stored analyses of an asset, as long as they describe its current content: the silences the
//! timeline shows on clips (prompt 14) and the full results AI read tools return (prompt 18).

use se_domain::{AnalysisKind, AnalysisResult, AssetId, ProjectId, TimeRange};

use super::{MediaService, find_asset};
use crate::AppError;

impl MediaService {
    /// The silences of `asset_id` in source time, in order; `None` while its sound is not
    /// analysed (no analysis yet, or one of other content after a relink).
    ///
    /// # Errors
    /// Not open, unknown asset, an unreadable analysis.
    pub async fn audio_silences(
        &self,
        project_id: ProjectId,
        asset_id: AssetId,
    ) -> Result<Option<Vec<TimeRange>>, AppError> {
        Ok(self
            .analysis(project_id, asset_id, AnalysisKind::AudioAnalysis)
            .await?
            .and_then(|result| match result {
                AnalysisResult::AudioAnalysis(analysis) => Some(analysis.silences),
                AnalysisResult::SceneDetection(_) | AnalysisResult::SubjectDetection(_) => None,
            }))
    }

    /// The stored `kind` analysis of `asset_id` (source time); `None` while it is not analysed
    /// (no analysis yet, or one of other content after a relink).
    ///
    /// # Errors
    /// Not open, unknown asset, an unreadable analysis.
    pub async fn analysis(
        &self,
        project_id: ProjectId,
        asset_id: AssetId,
        kind: AnalysisKind,
    ) -> Result<Option<AnalysisResult>, AppError> {
        let handle = self.handle(project_id)?;
        let project = handle.view().await?.project;
        let asset = find_asset(&project, asset_id)?;
        let Some(fingerprint) = &asset.fingerprint else {
            return Ok(None);
        };
        let stored = handle.analyses().load_analysis(asset_id, kind).await?;
        Ok(stored.and_then(|stored| {
            (stored.content_key == fingerprint.cache_key() && stored.result.kind() == kind)
                .then_some(stored.result)
        }))
    }
}
