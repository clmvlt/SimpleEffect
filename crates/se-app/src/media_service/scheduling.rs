//! Queuing what an asset still needs: cache files (thumbnails, peaks), analyses, then its
//! mezzanine and proxy (`derived.rs`). Idempotent: called after imports, relinks, decodability
//! reports, settings changes and when a project opens.

use std::{path::Path, sync::Arc};

use se_domain::{
    AnalysisKind, Asset, AssetKind, ItemKind, Project, SceneDetection, SilenceSettings,
};
use se_protocol::JobKind;

use super::{
    MediaService,
    job_work::{AnalysisRecorder, MediaJobWork, OutcomeRecorder},
    media_paths::media_cache_dir,
};
use crate::{
    jobs::{JobPriority, JobRequest},
    ports::{MediaJobSpec, MediaTask},
    project_actor::ProjectActorHandle,
};

/// Why an asset is being scheduled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Trigger {
    /// Something changed (import, relink, decodability report, settings): fill what is missing.
    Change,
    /// The project opened or its cache was cleared: also resume pending copies and retry failed
    /// ones.
    Opening,
}

/// The cache files an asset needs: pictures for videos and images, peaks for sound.
fn cache_tasks(asset: &Asset) -> Vec<MediaTask> {
    let mut tasks = Vec::new();
    if matches!(asset.kind, AssetKind::Video | AssetKind::Image) {
        tasks.push(MediaTask::Thumbnails);
    }
    if has_sound(asset) {
        tasks.push(MediaTask::WaveformPeaks);
    }
    tasks
}

/// The analyses an asset needs: sound analysis when it has sound, scene detection for videos.
fn analysis_tasks(asset: &Asset) -> Vec<(AnalysisKind, MediaTask)> {
    let mut tasks = Vec::new();
    if has_sound(asset) {
        let silence = SilenceSettings::SPEECH;
        tasks.push((
            AnalysisKind::AudioAnalysis,
            MediaTask::AudioAnalysis { silence },
        ));
    }
    let has_picture = asset
        .media_info
        .as_ref()
        .is_some_and(|info| info.video_stream.is_some());
    if asset.kind == AssetKind::Video && has_picture {
        let threshold = SceneDetection::DEFAULT_THRESHOLD;
        tasks.push((
            AnalysisKind::SceneDetection,
            MediaTask::SceneDetection { threshold },
        ));
    }
    tasks
}

fn has_sound(asset: &Asset) -> bool {
    asset.kind != AssetKind::Image
        && asset
            .media_info
            .as_ref()
            .is_some_and(|info| !info.audio_streams.is_empty())
}

/// Whether a clip or image item of `project` uses the asset.
pub(super) fn is_used(project: &Project, asset: &Asset) -> bool {
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.items)
        .any(|item| match &item.kind {
            ItemKind::MediaClip(clip) => clip.asset_id == asset.id,
            ItemKind::Image(image) => image.asset_id == asset.id,
            _ => false,
        })
}

/// Start order (ADR-0019): thumbnails and peaks (the bin shows them) > proxies and mezzanines of
/// used media > analyses > mezzanines of media no clip uses yet.
pub(super) fn priority(kind: JobKind, used: bool) -> JobPriority {
    match kind {
        JobKind::Thumbnails | JobKind::WaveformPeaks => JobPriority::High,
        JobKind::Proxy => JobPriority::Normal,
        JobKind::Mezzanine if used => JobPriority::Normal,
        JobKind::Mezzanine => JobPriority::Idle,
        JobKind::AudioAnalysis | JobKind::SceneDetection => JobPriority::Low,
    }
}

impl MediaService {
    /// Queues what `asset` still needs (skipped for missing originals and assets without
    /// fingerprint or probe). Failures are logged: scheduling never fails a use case.
    pub(super) async fn schedule_asset(
        &self,
        handle: &ProjectActorHandle,
        project: &Project,
        asset: &Asset,
        trigger: Trigger,
    ) {
        let Some(fingerprint) = &asset.fingerprint else {
            return;
        };
        if asset.missing || asset.media_info.is_none() {
            return;
        }
        let output_dir = media_cache_dir(handle.path(), fingerprint);
        for task in cache_tasks(asset) {
            let kind = task.kind();
            if !self.deps.jobs.has_active(project.id, asset.id, kind)
                && !self.deps.runner.outputs_ready(kind, &output_dir).await
            {
                self.enqueue(
                    handle,
                    asset,
                    task,
                    &output_dir,
                    priority(kind, false),
                    None,
                );
            }
        }
        let content_key = fingerprint.cache_key();
        for (kind, task) in analysis_tasks(asset) {
            self.schedule_analysis(handle, asset, kind, task, &output_dir, &content_key)
                .await;
        }
        if let Err(error) = self.schedule_derived(handle, project, asset, trigger).await {
            tracing::warn!(%error, asset = %asset.id, "queuing derived media failed");
        }
    }

    /// Schedules every asset of every open project (after the settings changed).
    pub(super) async fn schedule_open_projects(&self) {
        let handles: Vec<ProjectActorHandle> = self.registry().values().cloned().collect();
        for handle in handles {
            let Ok(view) = handle.view().await else {
                continue;
            };
            for asset in &view.project.assets {
                self.schedule_asset(&handle, &view.project, asset, Trigger::Change)
                    .await;
            }
        }
    }

    /// Queues an analysis unless one runs or the database holds it for the current content;
    /// a stored result whose summary never reached the asset (crash) is summarised now.
    async fn schedule_analysis(
        &self,
        handle: &ProjectActorHandle,
        asset: &Asset,
        kind: AnalysisKind,
        task: MediaTask,
        output_dir: &Path,
        content_key: &str,
    ) {
        if self
            .deps
            .jobs
            .has_active(handle.project_id(), asset.id, task.kind())
        {
            return;
        }
        let recorder = AnalysisRecorder {
            handle: handle.clone(),
            ids: Arc::clone(&self.deps.ids),
            clock: Arc::clone(&self.deps.clock),
            asset_id: asset.id,
            content_key: content_key.to_owned(),
        };
        match handle.analyses().load_analysis(asset.id, kind).await {
            Ok(Some(stored)) if stored.content_key == content_key => {
                recorder.record_summary(&stored.result).await;
                return;
            }
            Ok(_) => {}
            Err(error) => {
                tracing::warn!(%error, asset = %asset.id, "reading a stored analysis failed");
            }
        }
        let priority = priority(task.kind(), false);
        self.enqueue(
            handle,
            asset,
            task,
            output_dir,
            priority,
            Some(Arc::new(recorder)),
        );
    }

    /// Queues `task` on `asset`.
    pub(super) fn enqueue(
        &self,
        handle: &ProjectActorHandle,
        asset: &Asset,
        task: MediaTask,
        output_dir: &Path,
        priority: JobPriority,
        recorder: Option<Arc<dyn OutcomeRecorder>>,
    ) {
        let Some(info) = &asset.media_info else {
            return;
        };
        let kind = task.kind();
        let work = MediaJobWork {
            runner: Arc::clone(&self.deps.runner),
            spec: MediaJobSpec {
                task,
                input: asset.original_path.clone().into(),
                output_dir: output_dir.to_path_buf(),
                asset_kind: asset.kind,
                media_info: info.clone(),
            },
            recorder,
        };
        let request = JobRequest {
            project_id: handle.project_id(),
            asset_id: Some(asset.id),
            kind,
            priority,
            work: Arc::new(work),
        };
        if let Err(error) = self.deps.jobs.enqueue(request) {
            tracing::error!(%error, asset = %asset.id, "queuing a media job failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use se_domain::test_support::{example_ids, example_project};

    use super::*;

    #[test]
    fn videos_need_pictures_peaks_and_both_analyses() {
        let project = example_project();
        let asset = |id| project.assets.iter().find(|asset| asset.id == id).unwrap();
        let kinds = |id| {
            cache_tasks(asset(id))
                .iter()
                .map(MediaTask::kind)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            kinds(example_ids::VIDEO_ASSET),
            [JobKind::Thumbnails, JobKind::WaveformPeaks]
        );
        assert_eq!(kinds(example_ids::AUDIO_ASSET), [JobKind::WaveformPeaks]);
        assert_eq!(kinds(example_ids::IMAGE_ASSET), [JobKind::Thumbnails]);
        let analyses = |id| {
            analysis_tasks(asset(id))
                .into_iter()
                .map(|(kind, _)| kind)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            analyses(example_ids::VIDEO_ASSET),
            [AnalysisKind::AudioAnalysis, AnalysisKind::SceneDetection]
        );
        assert_eq!(
            analyses(example_ids::AUDIO_ASSET),
            [AnalysisKind::AudioAnalysis]
        );
        assert_eq!(analyses(example_ids::IMAGE_ASSET), []);
    }

    #[test]
    fn previews_come_first_and_unused_mezzanines_last() {
        assert_eq!(priority(JobKind::Thumbnails, false), JobPriority::High);
        assert!(priority(JobKind::Proxy, false) > priority(JobKind::AudioAnalysis, false));
        assert!(priority(JobKind::SceneDetection, false) > priority(JobKind::Mezzanine, false));
        assert_eq!(priority(JobKind::Mezzanine, true), JobPriority::Normal);
        let project = example_project();
        assert!(is_used(&project, &project.assets[0]));
    }
}
