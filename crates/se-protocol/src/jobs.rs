//! Background job DTOs: what a job is, its state and its progress (`docs/architecture.md` §7).

use se_domain::{AssetId, JobId, ProjectId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What a background job produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum JobKind {
    /// A sheet of small pictures at regular intervals plus a poster (videos and images).
    Thumbnails,
    /// Minimum and maximum sample values per channel at several zoom levels (sound).
    WaveformPeaks,
}

/// Where a job stands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum JobState {
    /// Waiting for a free slot of its kind.
    Queued,
    /// Running.
    Running {
        /// Fraction done in `[0, 1]`, when the job can tell.
        progress: Option<f64>,
    },
    /// Finished; its outputs are in the cache.
    Completed,
    /// Failed twice (one automatic retry).
    Failed {
        /// Why the last attempt failed.
        message: String,
    },
    /// Cancelled by the user or because its project closed.
    Cancelled,
}

/// A job as clients see it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct JobSnapshot {
    /// Stable id.
    pub id: JobId,
    /// The project it works for.
    pub project_id: ProjectId,
    /// The asset it works on, if any.
    pub asset_id: Option<AssetId>,
    /// What it produces.
    pub kind: JobKind,
    /// Where it stands.
    pub state: JobState,
    /// Attempt number, from 1 (2 for the automatic retry).
    pub attempt: u8,
}

/// Progress of a running job, pushed by [`crate::CoreEvent::JobProgress`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct JobProgress {
    /// The job.
    pub job_id: JobId,
    /// Its project.
    pub project_id: ProjectId,
    /// Its asset, if any.
    pub asset_id: Option<AssetId>,
    /// Fraction done in `[0, 1]`, when the job can tell.
    pub progress: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states_are_tagged_by_kind() {
        assert_eq!(
            serde_json::to_string(&JobState::Running {
                progress: Some(0.5)
            })
            .unwrap(),
            r#"{"kind":"running","progress":0.5}"#
        );
        assert_eq!(
            serde_json::to_string(&JobState::Cancelled).unwrap(),
            r#"{"kind":"cancelled"}"#
        );
    }

    #[test]
    fn kinds_are_camel_case() {
        assert_eq!(
            serde_json::to_string(&JobKind::WaveformPeaks).unwrap(),
            r#""waveformPeaks""#
        );
    }
}
