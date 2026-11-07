//! The silent stretches of an asset's sound (prompt 14): read from its stored audio analysis, so
//! the timeline can show them on the clips of that asset.

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
        let handle = self.handle(project_id)?;
        let project = handle.view().await?.project;
        let asset = find_asset(&project, asset_id)?;
        let Some(fingerprint) = &asset.fingerprint else {
            return Ok(None);
        };
        let stored = handle
            .analyses()
            .load_analysis(asset_id, AnalysisKind::AudioAnalysis)
            .await?;
        Ok(stored.and_then(|stored| match stored.result {
            AnalysisResult::AudioAnalysis(analysis)
                if stored.content_key == fingerprint.cache_key() =>
            {
                Some(analysis.silences)
            }
            _ => None,
        }))
    }
}
