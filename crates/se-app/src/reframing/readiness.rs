//! Where the analyses automatic reframing needs stand for one asset: faces (subject detection,
//! of the current content and model) and shot changes (scene detection), read from the project
//! database and the job queue.

use se_domain::{AnalysisKind, AnalysisResult, Asset, AssetId, ContentFingerprint};
use se_protocol::{JobKind, JobSnapshot, JobState};

use super::plan::AssetAnalyses;
use crate::{AppError, project_actor::ProjectActorHandle};

/// The analyses of one asset.
#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisState {
    /// Both are there (no shot changes when the scene detection could not run).
    Ready(AssetAnalyses),
    /// A job of either is queued or running.
    Waiting,
    /// The last subject detection failed or was cancelled.
    Failed(String),
    /// No subject detection was asked for this content yet.
    NotStarted,
}

/// The state of `asset`'s analyses for the detector `model`, given the project's `jobs`.
///
/// # Errors
/// An unreadable stored analysis.
pub async fn analysis_state(
    handle: &ProjectActorHandle,
    jobs: &[JobSnapshot],
    asset: &Asset,
    model: &str,
) -> Result<AnalysisState, AppError> {
    let Some(content_key) = asset
        .fingerprint
        .as_ref()
        .map(ContentFingerprint::cache_key)
    else {
        return Ok(AnalysisState::Failed(
            "the media is still being imported: try again in a moment".to_owned(),
        ));
    };
    let store = handle.analyses();
    let detection = store
        .load_analysis(asset.id, AnalysisKind::SubjectDetection)
        .await?
        .filter(|stored| stored.content_key == content_key)
        .and_then(|stored| match stored.result {
            AnalysisResult::SubjectDetection(detection) if detection.model == model => {
                Some(detection)
            }
            _ => None,
        });
    let scene_changes = store
        .load_analysis(asset.id, AnalysisKind::SceneDetection)
        .await?
        .filter(|stored| stored.content_key == content_key)
        .and_then(|stored| match stored.result {
            AnalysisResult::SceneDetection(scenes) => Some(scenes.changes),
            _ => None,
        });
    let active = |kind| last_job(jobs, asset.id, kind).is_some_and(|job| is_active(&job.state));
    Ok(match detection {
        Some(_) if active(JobKind::SceneDetection) => AnalysisState::Waiting,
        Some(detection) => AnalysisState::Ready(AssetAnalyses {
            detection,
            scene_changes: scene_changes.unwrap_or_default(),
        }),
        None => match last_job(jobs, asset.id, JobKind::SubjectDetection).map(|job| &job.state) {
            Some(JobState::Queued | JobState::Running { .. }) => AnalysisState::Waiting,
            Some(JobState::Failed { message }) => AnalysisState::Failed(message.clone()),
            Some(JobState::Cancelled) => {
                AnalysisState::Failed("the face analysis was cancelled".to_owned())
            }
            Some(JobState::Completed) | None => AnalysisState::NotStarted,
        },
    })
}

/// Whether a scene detection of the current content is missing (neither stored nor running).
///
/// # Errors
/// An unreadable stored analysis.
pub async fn lacks_scene_detection(
    handle: &ProjectActorHandle,
    jobs: &[JobSnapshot],
    asset: &Asset,
) -> Result<bool, AppError> {
    let Some(content_key) = asset
        .fingerprint
        .as_ref()
        .map(ContentFingerprint::cache_key)
    else {
        return Ok(false);
    };
    let stored = handle
        .analyses()
        .load_analysis(asset.id, AnalysisKind::SceneDetection)
        .await?
        .is_some_and(|stored| stored.content_key == content_key);
    let running =
        last_job(jobs, asset.id, JobKind::SceneDetection).is_some_and(|job| is_active(&job.state));
    Ok(!stored && !running)
}

fn last_job(jobs: &[JobSnapshot], asset_id: AssetId, kind: JobKind) -> Option<&JobSnapshot> {
    jobs.iter()
        .rev()
        .find(|job| job.kind == kind && job.asset_id == Some(asset_id))
}

const fn is_active(state: &JobState) -> bool {
    matches!(state, JobState::Queued | JobState::Running { .. })
}
