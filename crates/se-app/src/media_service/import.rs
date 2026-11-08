//! Importing media files: fingerprint, deduplicate, probe, add the assets in one user
//! transaction, then queue their derived files.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use se_domain::{
    Actor, AddAsset, Asset, AssetAnalysis, AssetId, ContentFingerprint, Decodability, DerivedMedia,
    Project, ProjectId, Transaction,
};
use se_protocol::{DuplicateMedia, ImportFailure, ImportReport};
use tokio::task::JoinSet;

use super::{MediaService, labels, media_paths::relative_hint, scheduling::Trigger};
use crate::{
    AppError,
    error::media_code,
    new_ids::NewIds,
    ports::{MediaError, MediaProber, ProbedMedia},
};

/// Probes running at once (each is an ffprobe process).
const PROBE_CONCURRENCY: usize = 4;

/// A file whose content is new to the project.
struct Candidate {
    path: PathBuf,
    fingerprint: ContentFingerprint,
}

/// The files of a request sorted out before probing.
#[derive(Default)]
struct Sorted {
    /// New contents, first occurrence of each.
    candidates: Vec<Candidate>,
    /// Later paths with the content of `candidates[index]`.
    batch_duplicates: Vec<(PathBuf, usize)>,
}

impl MediaService {
    /// Imports `paths` into an open project (absolute paths, in bin order). Contents the project
    /// already has (or that appear twice) are reported as duplicates; unreadable files as
    /// failures. The new assets form one undoable transaction, then their thumbnails, peaks,
    /// analyses and (when needed) proxies are queued.
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`], or the commit error (nothing is imported then).
    pub async fn import_files(
        &self,
        project_id: ProjectId,
        paths: &[PathBuf],
    ) -> Result<ImportReport, AppError> {
        let handle = self.handle(project_id)?;
        let project = handle.view().await?.project;
        let mut report = ImportReport::default();
        let sorted = self.sort_files(&project, paths, &mut report).await;
        let probe_results = probe_all(&self.deps.prober, &sorted.candidates).await;
        let ids = NewIds(self.deps.ids.as_ref());
        let mut assets = Vec::new();
        let mut outcomes: Vec<Result<AssetId, MediaError>> = Vec::new();
        for (candidate, result) in sorted.candidates.into_iter().zip(probe_results) {
            match result {
                Ok(probed) => {
                    let asset = new_asset(ids.asset()?, &candidate, handle.path(), probed);
                    outcomes.push(Ok(asset.id));
                    assets.push(asset);
                }
                Err(error) => {
                    report.failures.push(failure(&candidate.path, &error));
                    outcomes.push(Err(error));
                }
            }
        }
        for (path, index) in sorted.batch_duplicates {
            match &outcomes[index] {
                Ok(asset_id) => report.duplicates.push(DuplicateMedia {
                    path: display(&path),
                    asset_id: *asset_id,
                }),
                Err(error) => report.failures.push(failure(&path, error)),
            }
        }
        if assets.is_empty() {
            return Ok(report);
        }
        let transaction = Transaction {
            id: ids.transaction()?,
            actor: Actor::User,
            label: labels::IMPORT_MEDIA.to_owned(),
            ops: assets
                .iter()
                .map(|asset| {
                    AddAsset {
                        index: None,
                        asset: asset.clone(),
                    }
                    .into()
                })
                .collect(),
        };
        handle.apply(transaction).await?;
        let project = handle.view().await?.project;
        for asset in &assets {
            report.imported.push(asset.id);
            self.schedule_asset(&handle, &project, asset, Trigger::Change)
                .await;
        }
        for listener in self.import_listeners() {
            listener.assets_imported(&handle, &report.imported).await;
        }
        Ok(report)
    }

    /// Checks and fingerprints each path, recording missing files, unreadable files and contents
    /// the project already has in `report`.
    async fn sort_files(
        &self,
        project: &Project,
        paths: &[PathBuf],
        report: &mut ImportReport,
    ) -> Sorted {
        let mut sorted = Sorted::default();
        for path in paths {
            if !self.deps.files.is_file(path).await {
                let error = MediaError::NotFound { path: path.clone() };
                report.failures.push(failure(path, &error));
                continue;
            }
            let fingerprint = match self.deps.files.fingerprint(path).await {
                Ok(fingerprint) => fingerprint,
                Err(error) => {
                    report.failures.push(failure(path, &error));
                    continue;
                }
            };
            let existing = project.assets.iter().find(|asset| {
                asset
                    .fingerprint
                    .as_ref()
                    .is_some_and(|known| known.same_content(&fingerprint))
            });
            if let Some(asset) = existing {
                report.duplicates.push(DuplicateMedia {
                    path: display(path),
                    asset_id: asset.id,
                });
            } else if let Some(index) = sorted
                .candidates
                .iter()
                .position(|candidate| candidate.fingerprint.same_content(&fingerprint))
            {
                sorted.batch_duplicates.push((path.clone(), index));
            } else {
                sorted.candidates.push(Candidate {
                    path: path.clone(),
                    fingerprint,
                });
            }
        }
        sorted
    }
}

/// Probes every candidate, [`PROBE_CONCURRENCY`] at a time; results in candidate order.
async fn probe_all(
    prober: &Arc<dyn MediaProber>,
    candidates: &[Candidate],
) -> Vec<Result<ProbedMedia, MediaError>> {
    let mut results: Vec<Option<Result<ProbedMedia, MediaError>>> =
        candidates.iter().map(|_| None).collect();
    let mut running = JoinSet::new();
    let mut pending = candidates.iter().enumerate();
    loop {
        while running.len() < PROBE_CONCURRENCY {
            let Some((index, candidate)) = pending.next() else {
                break;
            };
            let prober = Arc::clone(prober);
            let path = candidate.path.clone();
            running.spawn(async move { (index, prober.probe(&path).await) });
        }
        let Some(joined) = running.join_next().await else {
            break;
        };
        match joined {
            Ok((index, result)) => results[index] = Some(result),
            Err(error) => tracing::error!(%error, "a probe task stopped unexpectedly"),
        }
    }
    results
        .into_iter()
        .zip(candidates)
        .map(|(result, candidate)| {
            result.unwrap_or_else(|| {
                Err(MediaError::Failed(format!(
                    "probing {} stopped unexpectedly: try again",
                    candidate.path.display()
                )))
            })
        })
        .collect()
}

/// The asset of a probed candidate.
fn new_asset(id: AssetId, candidate: &Candidate, project_dir: &Path, probed: ProbedMedia) -> Asset {
    Asset {
        id,
        original_path: display(&candidate.path),
        relative_hint: relative_hint(project_dir, &candidate.path),
        kind: probed.kind,
        decodability: Decodability::unchecked(&probed.info),
        media_info: Some(probed.info),
        fingerprint: Some(candidate.fingerprint.clone()),
        derived: DerivedMedia::default(),
        analysis: AssetAnalysis::default(),
        missing: false,
    }
}

/// The report entry of a file that could not be imported.
fn failure(path: &Path, error: &MediaError) -> ImportFailure {
    ImportFailure {
        path: display(path),
        code: media_code(error),
        message: error.to_string(),
    }
}

/// A path as stored and sent to clients. Non-UTF-8 characters are replaced (backlog).
pub(super) fn display(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
