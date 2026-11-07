//! Mezzanines and proxies: what an asset needs ([`crate::media_policy`]), recorded as pending in
//! the document (a system transaction), then queued; and the request to generate one.
//!
//! A derived file's state lives in the document (`Asset.derived`): pending while queued or
//! running, ready once written, failed with the reason. The file itself is in the content's cache
//! folder, so a ready state whose file disappeared (cache cleaned) is queued again.

use std::{path::PathBuf, sync::Arc};

use se_domain::{
    Actor, Asset, AssetPatch, Assign, DerivedFile, DerivedStatus, Project, UpdateAsset,
};
use se_protocol::{DerivedKind, GenerateDerivedRequest, MediaSettings};

use super::{
    MediaService, find_asset,
    job_work::DerivedRecorder,
    labels,
    media_paths::{
        cached_file_relative_path, media_cache_dir, mezzanine_file_name, proxy_file_name,
        resolve_relative,
    },
    scheduling::{Trigger, is_used, priority},
};
use crate::{
    AppError,
    media_policy::{MezzanineDecision, auto_proxy_wanted, mezzanine_decision, proxy_recipe},
    ports::{MediaError, MediaTask, TranscodePlan, TranscodeRecipe},
    project_actor::ProjectActorHandle,
};

/// A derived copy to queue.
struct DerivedJob {
    derived: DerivedKind,
    relative_path: String,
    output: PathBuf,
    recipe: TranscodeRecipe,
}

/// The current state of one of an asset's derived copies.
fn current(asset: &Asset, derived: DerivedKind) -> Option<&DerivedFile> {
    match derived {
        DerivedKind::Mezzanine => asset.derived.mezzanine.as_ref(),
        DerivedKind::Proxy => asset.derived.proxy.as_ref(),
    }
}

/// Sets one derived copy in `patch`.
fn set(patch: &mut AssetPatch, derived: DerivedKind, file: DerivedFile) {
    let value = Some(Assign { value: Some(file) });
    match derived {
        DerivedKind::Mezzanine => patch.mezzanine = value,
        DerivedKind::Proxy => patch.proxy = value,
    }
}

/// The file name of a new derived copy.
fn file_name(derived: DerivedKind, recipe: &TranscodeRecipe, settings: &MediaSettings) -> String {
    match derived {
        DerivedKind::Mezzanine => mezzanine_file_name(recipe.container),
        DerivedKind::Proxy => proxy_file_name(settings.proxy_resolution),
    }
}

impl MediaService {
    /// Queues the mezzanine and proxy `asset` needs: a mezzanine when the policy says so, a proxy
    /// when one was ever wanted or automatic proxies want it. On [`Trigger::Opening`], failed
    /// copies are tried again.
    pub(super) async fn schedule_derived(
        &self,
        handle: &ProjectActorHandle,
        project: &Project,
        asset: &Asset,
        trigger: Trigger,
    ) -> Result<(), AppError> {
        let settings = self.media_settings().await?;
        let capabilities = self.decoder_capabilities().await?;
        let mut wanted = Vec::new();
        let decision = mezzanine_decision(asset, capabilities, settings.mezzanine_quality);
        if let MezzanineDecision::Needed(recipe) = decision {
            wanted.push((DerivedKind::Mezzanine, recipe));
        }
        let proxy_wanted =
            asset.derived.proxy.is_some() || (settings.auto_proxies && auto_proxy_wanted(asset));
        if proxy_wanted
            && let Some(recipe) = proxy_recipe(asset, capabilities, settings.proxy_resolution)
        {
            wanted.push((DerivedKind::Proxy, recipe));
        }
        let retry_failed = trigger == Trigger::Opening;
        self.queue_derived(handle, project, asset, wanted, retry_failed, &settings)
            .await
    }

    /// Queues a mezzanine or proxy on request (« Générer un proxy », « Réessayer »).
    ///
    /// # Errors
    /// Not open, unknown asset, missing original, [`AppError::DerivedNotApplicable`] (a proxy of
    /// a sound file, a mezzanine of media WebCodecs decodes), storage and commit errors.
    pub async fn generate_derived(&self, request: &GenerateDerivedRequest) -> Result<(), AppError> {
        let handle = self.handle(request.project_id)?;
        let project = handle.view().await?.project;
        let asset = find_asset(&project, request.asset_id)?;
        if asset.missing {
            return Err(MediaError::NotFound {
                path: asset.original_path.clone().into(),
            }
            .into());
        }
        let settings = self.media_settings().await?;
        let capabilities = self.decoder_capabilities().await?;
        let not_applicable = |derived: &'static str, reason: &str| AppError::DerivedNotApplicable {
            asset_id: asset.id,
            derived,
            reason: reason.to_owned(),
        };
        let recipe = match request.derived {
            DerivedKind::Mezzanine => {
                match mezzanine_decision(asset, capabilities, settings.mezzanine_quality) {
                    MezzanineDecision::Needed(recipe) => recipe,
                    MezzanineDecision::Waiting => {
                        return Err(not_applicable(
                            "mezzanine",
                            "its decodability is still being checked: try again in a moment",
                        ));
                    }
                    MezzanineDecision::NotNeeded => {
                        return Err(not_applicable(
                            "mezzanine",
                            "this machine decodes the original directly",
                        ));
                    }
                }
            }
            DerivedKind::Proxy => proxy_recipe(asset, capabilities, settings.proxy_resolution)
                .ok_or_else(|| not_applicable("proxy", "only videos get proxies"))?,
        };
        self.queue_derived(
            &handle,
            &project,
            asset,
            vec![(request.derived, recipe)],
            true,
            &settings,
        )
        .await
    }

    /// Records the wanted copies that need a job as pending (one system transaction), then queues
    /// them. A pending copy without a running job, a ready copy whose file is gone and (when
    /// `retry_failed`) a failed copy get a new job.
    async fn queue_derived(
        &self,
        handle: &ProjectActorHandle,
        project: &Project,
        asset: &Asset,
        wanted: Vec<(DerivedKind, TranscodeRecipe)>,
        retry_failed: bool,
        settings: &MediaSettings,
    ) -> Result<(), AppError> {
        let Some(fingerprint) = &asset.fingerprint else {
            return Ok(());
        };
        if asset.missing {
            return Ok(());
        }
        let mut patch = AssetPatch::default();
        let mut jobs = Vec::new();
        for (derived, recipe) in wanted {
            let existing = current(asset, derived);
            let relative_path = existing.map_or_else(
                || cached_file_relative_path(fingerprint, &file_name(derived, &recipe, settings)),
                |file| file.relative_path.clone(),
            );
            let Some(output) = resolve_relative(handle.path(), &relative_path) else {
                tracing::warn!(asset = %asset.id, %relative_path, "a derived file points outside the project");
                continue;
            };
            let pending = DerivedFile {
                relative_path: relative_path.clone(),
                status: DerivedStatus::Pending,
            };
            let job = DerivedJob {
                derived,
                relative_path,
                output,
                recipe,
            };
            let needs_job = match existing.map(|file| &file.status) {
                Some(DerivedStatus::Pending) => {
                    let kind = TranscodePlan::kind_of(derived);
                    !self.deps.jobs.has_active(project.id, asset.id, kind)
                }
                Some(DerivedStatus::Ready) if self.deps.files.is_file(&job.output).await => false,
                Some(DerivedStatus::Failed { .. }) if !retry_failed => false,
                _ => {
                    set(&mut patch, derived, pending);
                    true
                }
            };
            if needs_job {
                jobs.push(job);
            }
        }
        if patch != AssetPatch::default() {
            let operation = UpdateAsset {
                asset: asset.id,
                patch,
            };
            self.commit(
                handle,
                Actor::System,
                labels::QUEUE_DERIVED_MEDIA,
                vec![operation.into()],
            )
            .await?;
        }
        let output_dir = media_cache_dir(handle.path(), fingerprint);
        let used = is_used(project, asset);
        for job in jobs {
            let recorder = DerivedRecorder {
                handle: handle.clone(),
                ids: Arc::clone(&self.deps.ids),
                asset_id: asset.id,
                derived: job.derived,
                relative_path: job.relative_path,
            };
            let plan = TranscodePlan {
                derived: job.derived,
                output: job.output,
                recipe: job.recipe,
            };
            let priority = priority(plan.job_kind(), used);
            let task = MediaTask::Transcode(plan);
            self.enqueue(
                handle,
                asset,
                task,
                &output_dir,
                priority,
                Some(Arc::new(recorder)),
            );
        }
        Ok(())
    }
}
