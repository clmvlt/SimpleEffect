//! The `analyses` table: one result per asset and analysis kind, with the content it describes.

use rusqlite::{OptionalExtension, params};
use se_app::ports::{StorageError, StoredAnalysis};
use se_domain::{AnalysisKind, AnalysisResult, AssetId};

use super::ProjectDatabase;
use crate::io_errors::sqlite_error;

/// Version of the JSON stored in `analyses.payload` (`se_domain::AnalysisResult`). Rows of a newer
/// version are ignored (the analysis runs again); bump it when the format changes.
const ANALYSIS_FORMAT_VERSION: u32 = 1;

impl ProjectDatabase {
    /// Saves `analysis`, replacing the previous result of the same asset and kind.
    ///
    /// # Errors
    /// IO errors.
    pub fn save_analysis(&self, analysis: &StoredAnalysis) -> Result<(), StorageError> {
        let payload =
            serde_json::to_string(&analysis.result).map_err(|error| StorageError::Io {
                action: "serialise an analysis for",
                path: self.path.clone(),
                reason: error.to_string(),
            })?;
        self.connection
            .execute(
                "INSERT INTO analyses (asset_id, kind, created_at, payload, content_key, format_version)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT (asset_id, kind) DO UPDATE SET created_at = excluded.created_at,
                 payload = excluded.payload, content_key = excluded.content_key,
                 format_version = excluded.format_version",
                params![
                    analysis.asset_id.to_string(),
                    analysis.result.kind().as_str(),
                    analysis.created_at,
                    payload,
                    analysis.content_key,
                    ANALYSIS_FORMAT_VERSION
                ],
            )
            .map(drop)
            .map_err(|error| sqlite_error("save an analysis in", &self.path, &error))
    }

    /// The stored result of `kind` for `asset_id`; `None` when absent or written by a newer
    /// version.
    ///
    /// # Errors
    /// [`StorageError::Corrupted`] for an unreadable payload; IO errors.
    pub fn load_analysis(
        &self,
        asset_id: AssetId,
        kind: AnalysisKind,
    ) -> Result<Option<StoredAnalysis>, StorageError> {
        let row = self
            .connection
            .query_row(
                "SELECT created_at, payload, content_key, format_version FROM analyses
                 WHERE asset_id = ?1 AND kind = ?2",
                params![asset_id.to_string(), kind.as_str()],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, u32>(3)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| sqlite_error("read an analysis of", &self.path, &error))?;
        let Some((created_at, payload, content_key, format_version)) = row else {
            return Ok(None);
        };
        if format_version > ANALYSIS_FORMAT_VERSION {
            return Ok(None);
        }
        let result: AnalysisResult =
            serde_json::from_str(&payload).map_err(|error| StorageError::Corrupted {
                path: self.path.clone(),
                reason: format!(
                    "the {} analysis of asset {asset_id} is unreadable: {error}",
                    kind.as_str()
                ),
            })?;
        Ok(Some(StoredAnalysis {
            asset_id,
            content_key,
            created_at,
            result,
        }))
    }
}

#[cfg(test)]
mod tests {
    use se_domain::{Flicks, SceneDetection};

    use super::*;

    fn detection(asset_id: AssetId, key: &str, changes: usize) -> StoredAnalysis {
        StoredAnalysis {
            asset_id,
            content_key: key.to_owned(),
            created_at: 42,
            result: AnalysisResult::SceneDetection(SceneDetection {
                threshold: 0.3,
                changes: vec![Flicks::one_second(); changes],
            }),
        }
    }

    #[test]
    fn analyses_are_saved_replaced_and_read_back() {
        let dir = tempfile::tempdir().unwrap();
        let database = ProjectDatabase::open(&dir.path().join("project.db")).unwrap();
        let asset = AssetId::from_test_index(1);
        assert_eq!(
            database
                .load_analysis(asset, AnalysisKind::SceneDetection)
                .unwrap(),
            None
        );
        database.save_analysis(&detection(asset, "old", 1)).unwrap();
        database.save_analysis(&detection(asset, "new", 2)).unwrap();
        let loaded = database
            .load_analysis(asset, AnalysisKind::SceneDetection)
            .unwrap();
        assert_eq!(loaded, Some(detection(asset, "new", 2)));
        assert_eq!(
            database
                .load_analysis(asset, AnalysisKind::AudioAnalysis)
                .unwrap(),
            None
        );
    }
}
