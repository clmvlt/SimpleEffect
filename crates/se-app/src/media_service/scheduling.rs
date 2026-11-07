//! Queuing the derived files of an asset (thumbnails, waveform peaks) unless they are cached.

use std::{path::Path, sync::Arc};

use async_trait::async_trait;
use se_domain::{Asset, AssetKind, ProjectId};
use se_protocol::JobKind;

use super::{MediaService, media_paths::media_cache_dir};
use crate::{
    jobs::{JobPriority, JobRequest, JobWork},
    ports::{MediaJobRunner, MediaJobSpec, ProgressSink},
};

/// A media job as queue work: runs the spec with the runner.
struct MediaJobWork {
    runner: Arc<dyn MediaJobRunner>,
    spec: MediaJobSpec,
}

#[async_trait]
impl JobWork for MediaJobWork {
    async fn run(&self, progress: &dyn ProgressSink) -> Result<(), String> {
        self.runner
            .run(&self.spec, progress)
            .await
            .map_err(|error| error.to_string())
    }
}

/// The derived files an asset needs: pictures for videos and images, peaks for sound.
fn needed_jobs(asset: &Asset) -> Vec<JobKind> {
    let has_sound = asset
        .media_info
        .as_ref()
        .is_some_and(|info| !info.audio_streams.is_empty());
    let mut kinds = Vec::new();
    if matches!(asset.kind, AssetKind::Video | AssetKind::Image) {
        kinds.push(JobKind::Thumbnails);
    }
    if has_sound && asset.kind != AssetKind::Image {
        kinds.push(JobKind::WaveformPeaks);
    }
    kinds
}

/// Thumbnails first: the media bin shows them; peaks wait for the timeline.
fn priority(kind: JobKind) -> JobPriority {
    match kind {
        JobKind::Thumbnails => JobPriority::High,
        JobKind::WaveformPeaks => JobPriority::Normal,
    }
}

impl MediaService {
    /// Queues the missing derived files of `asset` (skipped for missing originals, assets without
    /// fingerprint or probe, cached outputs and jobs already queued or running).
    pub(super) async fn schedule_jobs(
        &self,
        project_dir: &Path,
        project_id: ProjectId,
        asset: &Asset,
    ) {
        let (Some(fingerprint), Some(info)) = (&asset.fingerprint, &asset.media_info) else {
            return;
        };
        if asset.missing {
            return;
        }
        let output_dir = media_cache_dir(project_dir, fingerprint);
        for kind in needed_jobs(asset) {
            if self.deps.jobs.has_active(project_id, asset.id, kind)
                || self.deps.runner.outputs_ready(kind, &output_dir).await
            {
                continue;
            }
            let work = MediaJobWork {
                runner: Arc::clone(&self.deps.runner),
                spec: MediaJobSpec {
                    kind,
                    input: asset.original_path.clone().into(),
                    output_dir: output_dir.clone(),
                    asset_kind: asset.kind,
                    media_info: info.clone(),
                },
            };
            let request = JobRequest {
                project_id,
                asset_id: Some(asset.id),
                kind,
                priority: priority(kind),
                work: Arc::new(work),
            };
            if let Err(error) = self.deps.jobs.enqueue(request) {
                tracing::error!(%error, asset = %asset.id, "queuing a media job failed");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use se_domain::test_support::{example_ids, example_project};

    use super::*;

    #[test]
    fn videos_need_pictures_and_peaks_sounds_peaks_images_pictures() {
        let project = example_project();
        let kinds = |id| {
            let asset = project.assets.iter().find(|asset| asset.id == id).unwrap();
            needed_jobs(asset)
        };
        assert_eq!(
            kinds(example_ids::VIDEO_ASSET),
            [JobKind::Thumbnails, JobKind::WaveformPeaks]
        );
        assert_eq!(kinds(example_ids::AUDIO_ASSET), [JobKind::WaveformPeaks]);
        assert_eq!(kinds(example_ids::IMAGE_ASSET), [JobKind::Thumbnails]);
    }
}
