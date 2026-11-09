//! [`ReframeService`]: automatic reframing use cases. Clips whose media's faces are analysed are
//! reframed at once (one user transaction « Auto reframe »); for the others the analyses are
//! queued and a background waiter reframes them when they end (one more transaction) and
//! publishes `autoReframeFinished`. Agents wait inline instead ([`ReframeService::analyses_for`]).

use std::{collections::BTreeMap, sync::Arc, time::Duration};

use se_domain::{
    Asset, AssetId, Format, FormatAdaptation, ItemId, ProjectId, SequenceId, SubjectDetection,
    TrackingSettings,
};
use se_protocol::{AutoReframeReport, CoreEvent, ReframeFailure, VisionStatus};

use super::{
    detection_job::SubjectDetectionJob,
    labels,
    plan::{AssetAnalyses, ReframableClip, reframe_operations, select_clips, video_clips},
    readiness::{AnalysisState, analysis_state, lacks_scene_detection},
    sampling::sample_request,
};
use crate::{
    AppError, MediaService, ProjectService,
    jobs::{JobPriority, JobQueue, JobRequest},
    ports::{Clock, EventSink, FrameSampler, SubjectDetector},
    project_actor::ProjectActorHandle,
};

/// How often waiting analyses are checked.
const POLL_INTERVAL: Duration = Duration::from_millis(300);

/// A background waiter gives up after this long (the project is probably closed or stuck).
const WAIT_LIMIT: Duration = Duration::from_hours(6);

/// What the reframing service needs.
#[derive(Clone)]
pub struct ReframeServiceDeps {
    /// Open projects.
    pub projects: Arc<ProjectService>,
    /// Media files and the scene detection.
    pub media: Arc<MediaService>,
    /// The job queue (shared with the media and transcription services).
    pub jobs: JobQueue,
    /// Pictures sampled from videos (se-media).
    pub sampler: Arc<dyn FrameSampler>,
    /// The face detector (se-vision).
    pub detector: Arc<dyn SubjectDetector>,
    /// Events to clients.
    pub events: Arc<dyn EventSink>,
    /// Timestamps of stored analyses.
    pub clock: Arc<dyn Clock>,
}

/// Automatic reframing. Cheap to clone.
#[derive(Clone)]
pub struct ReframeService {
    deps: Arc<ReframeServiceDeps>,
}

/// The analyses of a set of assets after waiting for them (agents).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AnalysesRound {
    /// Assets ready, with their analyses.
    pub ready: BTreeMap<AssetId, AssetAnalyses>,
    /// Assets still being analysed when the wait ended.
    pub waiting: Vec<AssetId>,
    /// Assets whose analysis failed, and why.
    pub failed: Vec<ReframeFailure>,
}

impl ReframeService {
    /// A service over `deps`.
    #[must_use]
    pub fn new(deps: ReframeServiceDeps) -> Self {
        Self {
            deps: Arc::new(deps),
        }
    }

    /// Whether faces can be detected on this machine.
    pub async fn status(&self) -> VisionStatus {
        self.deps.detector.status().await
    }

    /// Reframes the clips of `item_ids` to follow their subject with `settings`: at once for
    /// media already analysed, after their analyses for the others.
    ///
    /// # Errors
    /// Detection unavailable, settings out of range, not open, unknown items, storage errors.
    pub async fn reframe_items(
        &self,
        project_id: ProjectId,
        item_ids: &[ItemId],
        settings: TrackingSettings,
    ) -> Result<AutoReframeReport, AppError> {
        self.ensure_available().await?;
        check_settings(settings)?;
        let handle = self.deps.projects.handle(project_id).await?;
        let project = handle.view().await?.project;
        let (clips, skipped) = select_clips(&project, item_ids)?;
        let assets = distinct_assets(&clips);
        let round = self.start(&handle, &assets).await?;
        let applied = self
            .apply(project_id, &clips, &round.ready, settings)
            .await?;
        let waiting: Vec<ReframableClip> = clips
            .iter()
            .filter(|clip| !round.ready.contains_key(&clip.asset_id))
            .copied()
            .collect();
        if !waiting.is_empty() {
            self.wait_in_background(project_id, waiting, settings);
        }
        Ok(AutoReframeReport {
            applied_item_ids: applied,
            waiting_asset_ids: round.waiting,
            skipped,
        })
    }

    /// Creates a version of `sequence_id` in `format` (a user step « Create version »), then
    /// reframes every clip of its video tracks with `settings` (see
    /// [`ReframeService::reframe_items`]).
    ///
    /// # Errors
    /// Those of `duplicate_sequence` and `reframe_items`.
    pub async fn create_version(
        &self,
        project_id: ProjectId,
        sequence_id: SequenceId,
        name: &str,
        format: Option<Format>,
        adaptation: FormatAdaptation,
        settings: TrackingSettings,
    ) -> Result<(SequenceId, AutoReframeReport), AppError> {
        self.ensure_available().await?;
        check_settings(settings)?;
        let copy = self
            .deps
            .projects
            .duplicate_sequence(project_id, sequence_id, name, format, adaptation)
            .await?;
        let handle = self.deps.projects.handle(project_id).await?;
        let project = handle.view().await?.project;
        let items = project
            .sequences
            .iter()
            .find(|sequence| sequence.id == copy)
            .map(video_clips)
            .unwrap_or_default();
        let report = self.reframe_items(project_id, &items, settings).await?;
        Ok((copy, report))
    }

    /// The stored faces of `asset_id` (current content and model), for the viewer.
    ///
    /// # Errors
    /// Not open, unknown asset, an unreadable analysis.
    pub async fn subject_detection(
        &self,
        project_id: ProjectId,
        asset_id: AssetId,
    ) -> Result<Option<SubjectDetection>, AppError> {
        let handle = self.deps.projects.handle(project_id).await?;
        let project = handle.view().await?.project;
        let asset = find_asset(&project.assets, project_id, asset_id)?;
        let jobs = self.deps.jobs.list(project_id);
        Ok(
            match analysis_state(&handle, &jobs, asset, self.deps.detector.model()).await? {
                AnalysisState::Ready(analyses) => Some(analyses.detection),
                _ => None,
            },
        )
    }

    /// Starts the analyses `asset_ids` lack and waits for them up to `timeout` (agents).
    ///
    /// # Errors
    /// Detection unavailable, not open, unknown assets, storage errors.
    pub async fn analyses_for(
        &self,
        project_id: ProjectId,
        asset_ids: &[AssetId],
        timeout: Duration,
    ) -> Result<AnalysesRound, AppError> {
        self.ensure_available().await?;
        let handle = self.deps.projects.handle(project_id).await?;
        let mut round = self.start(&handle, asset_ids).await?;
        let deadline = tokio::time::Instant::now() + timeout;
        while !round.waiting.is_empty() && tokio::time::Instant::now() < deadline {
            tokio::time::sleep_until(deadline.min(tokio::time::Instant::now() + POLL_INTERVAL))
                .await;
            round = self.states(&handle, asset_ids).await?;
        }
        Ok(round)
    }

    /// Refuses when faces cannot be detected here.
    async fn ensure_available(&self) -> Result<(), AppError> {
        let status = self.deps.detector.status().await;
        if status.available {
            Ok(())
        } else {
            Err(AppError::VisionUnavailable {
                reason: status
                    .reason
                    .unwrap_or_else(|| "the face detector cannot start".to_owned()),
            })
        }
    }

    /// The states of `asset_ids` after queueing what they lack.
    async fn start(
        &self,
        handle: &ProjectActorHandle,
        asset_ids: &[AssetId],
    ) -> Result<AnalysesRound, AppError> {
        let project = handle.view().await?.project;
        let jobs = self.deps.jobs.list(handle.project_id());
        for &asset_id in asset_ids {
            let asset = find_asset(&project.assets, handle.project_id(), asset_id)?;
            if lacks_scene_detection(handle, &jobs, asset).await? {
                self.deps
                    .media
                    .request_scene_detection(handle.project_id(), asset_id)
                    .await?;
            }
            let state = analysis_state(handle, &jobs, asset, self.deps.detector.model()).await?;
            if matches!(state, AnalysisState::NotStarted | AnalysisState::Failed(_)) {
                self.queue_detection(handle, asset).await?;
            }
        }
        self.states(handle, asset_ids).await
    }

    /// The analyses of `asset_ids` as they stand.
    async fn states(
        &self,
        handle: &ProjectActorHandle,
        asset_ids: &[AssetId],
    ) -> Result<AnalysesRound, AppError> {
        let project = handle.view().await?.project;
        let jobs = self.deps.jobs.list(handle.project_id());
        let mut round = AnalysesRound::default();
        for &asset_id in asset_ids {
            let Ok(asset) = find_asset(&project.assets, handle.project_id(), asset_id) else {
                round.failed.push(ReframeFailure {
                    asset_id,
                    message: "the media was removed from the project".to_owned(),
                });
                continue;
            };
            match analysis_state(handle, &jobs, asset, self.deps.detector.model()).await? {
                AnalysisState::Ready(analyses) => {
                    round.ready.insert(asset_id, analyses);
                }
                AnalysisState::Waiting | AnalysisState::NotStarted => round.waiting.push(asset_id),
                AnalysisState::Failed(message) => {
                    round.failed.push(ReframeFailure { asset_id, message });
                }
            }
        }
        Ok(round)
    }

    /// Queues the subject detection of `asset` (its ready proxy, else its original).
    async fn queue_detection(
        &self,
        handle: &ProjectActorHandle,
        asset: &Asset,
    ) -> Result<(), AppError> {
        let (Some(fingerprint), Some(info)) = (&asset.fingerprint, &asset.media_info) else {
            return Ok(());
        };
        let Some(stream) = &info.video_stream else {
            return Ok(());
        };
        let files = self.deps.media.asset_files(asset.id).await;
        let proxy = files.and_then(|files| files.proxy);
        let job = SubjectDetectionJob {
            sampler: Arc::clone(&self.deps.sampler),
            detector: Arc::clone(&self.deps.detector),
            clock: Arc::clone(&self.deps.clock),
            handle: handle.clone(),
            asset_id: asset.id,
            content_key: fingerprint.cache_key(),
            request: sample_request(stream, asset.original_path.clone().into(), proxy),
            duration: info.duration,
        };
        self.deps
            .jobs
            .enqueue(JobRequest {
                project_id: handle.project_id(),
                asset_id: Some(asset.id),
                kind: se_protocol::JobKind::SubjectDetection,
                priority: JobPriority::Normal,
                work: Arc::new(job),
            })
            .map(drop)
    }

    /// Reframes the `clips` whose asset is in `ready` as one user transaction; returns them.
    async fn apply(
        &self,
        project_id: ProjectId,
        clips: &[ReframableClip],
        ready: &BTreeMap<AssetId, AssetAnalyses>,
        settings: TrackingSettings,
    ) -> Result<Vec<ItemId>, AppError> {
        let concerned: Vec<ReframableClip> = clips
            .iter()
            .filter(|clip| ready.contains_key(&clip.asset_id))
            .copied()
            .collect();
        if concerned.is_empty() {
            return Ok(Vec::new());
        }
        self.deps
            .projects
            .plan_user_edit(project_id, labels::AUTO_REFRAME, |plan| {
                let project = plan.project()?.clone();
                let present: Vec<ReframableClip> = select_clips(
                    &project,
                    &concerned
                        .iter()
                        .map(|clip| clip.item_id)
                        .filter(|id| project.items().any(|item| item.id == *id))
                        .collect::<Vec<_>>(),
                )?
                .0;
                let ops = reframe_operations(&project, &present, ready, settings);
                let applied = present.iter().map(|clip| clip.item_id).collect();
                for operation in ops {
                    plan.push(operation);
                }
                Ok(applied)
            })
            .await
    }

    /// Waits for the analyses of `clips`' media, then reframes them and tells the clients.
    fn wait_in_background(
        &self,
        project_id: ProjectId,
        clips: Vec<ReframableClip>,
        settings: TrackingSettings,
    ) {
        let service = self.clone();
        tokio::spawn(async move {
            let assets = distinct_assets(&clips);
            let deadline = tokio::time::Instant::now() + WAIT_LIMIT;
            loop {
                tokio::time::sleep(POLL_INTERVAL).await;
                let Ok(handle) = service.deps.projects.handle(project_id).await else {
                    return;
                };
                let round = match service.states(&handle, &assets).await {
                    Ok(round) => round,
                    Err(error) => {
                        tracing::warn!(%error, "reading face analyses failed");
                        return;
                    }
                };
                if !round.waiting.is_empty() && tokio::time::Instant::now() < deadline {
                    continue;
                }
                service.finish(project_id, &clips, round, settings).await;
                return;
            }
        });
    }

    /// Reframes what `round` made ready and publishes `autoReframeFinished`.
    async fn finish(
        &self,
        project_id: ProjectId,
        clips: &[ReframableClip],
        mut round: AnalysesRound,
        settings: TrackingSettings,
    ) {
        let applied = match self.apply(project_id, clips, &round.ready, settings).await {
            Ok(applied) => applied,
            Err(error) => {
                tracing::warn!(%error, "automatic reframing failed");
                round
                    .ready
                    .keys()
                    .map(|&asset_id| ReframeFailure {
                        asset_id,
                        message: error.to_string(),
                    })
                    .for_each(|failure| round.failed.push(failure));
                Vec::new()
            }
        };
        round
            .failed
            .extend(round.waiting.iter().map(|&asset_id| ReframeFailure {
                asset_id,
                message: "the face analysis took too long: try again".to_owned(),
            }));
        self.deps.events.publish(CoreEvent::AutoReframeFinished {
            project_id,
            applied_item_ids: applied,
            failures: round.failed,
        });
    }
}

/// Refuses settings out of range.
fn check_settings(settings: TrackingSettings) -> Result<(), AppError> {
    let problems = settings.problems();
    if problems.is_empty() {
        return Ok(());
    }
    let details: Vec<String> = problems
        .iter()
        .map(|(field, rule)| format!("{field} {rule}"))
        .collect();
    Err(AppError::InvalidEdit {
        reason: format!("invalid tracking settings: {}", details.join(", ")),
    })
}

fn distinct_assets(clips: &[ReframableClip]) -> Vec<AssetId> {
    let mut assets: Vec<AssetId> = clips.iter().map(|clip| clip.asset_id).collect();
    assets.sort_unstable();
    assets.dedup();
    assets
}

fn find_asset(
    assets: &[Asset],
    project_id: ProjectId,
    asset_id: AssetId,
) -> Result<&Asset, AppError> {
    assets
        .iter()
        .find(|asset| asset.id == asset_id)
        .ok_or(AppError::AssetNotFound {
            project_id,
            asset_id,
        })
}
