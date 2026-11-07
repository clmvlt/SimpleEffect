//! Changing assets on request: relinking to another file, removing, recording decodability.

use std::path::Path;

use se_domain::{
    Actor, AssetId, AssetPatch, Assign, Decodability, Operation, ProjectId, RemoveAsset,
    Transaction, UpdateAsset,
};
use se_protocol::{DecodabilityReport, RelinkOutcome};

use super::{MediaService, find_asset, import::display, labels, media_paths::relative_hint};
use crate::{AppError, new_ids::NewIds, ports::MediaError, project_actor::ProjectActorHandle};

impl MediaService {
    /// Points an asset at the file at `path` (absolute). When the file's content differs from
    /// the asset's fingerprint (or the asset has none), nothing changes and
    /// [`RelinkOutcome::ContentDiffers`] asks for confirmation, unless `force`.
    ///
    /// # Errors
    /// Not open, unknown asset, missing or unreadable file, another kind of media, commit errors.
    pub async fn relink_asset(
        &self,
        project_id: ProjectId,
        asset_id: AssetId,
        path: &Path,
        force: bool,
    ) -> Result<RelinkOutcome, AppError> {
        let handle = self.handle(project_id)?;
        let project = handle.view().await?.project;
        let asset = find_asset(&project, asset_id)?;
        if !self.deps.files.is_file(path).await {
            return Err(MediaError::NotFound {
                path: path.to_path_buf(),
            }
            .into());
        }
        let fingerprint = self.deps.files.fingerprint(path).await?;
        let probed = self.deps.prober.probe(path).await?;
        if probed.kind != asset.kind {
            return Err(AppError::MediaKindMismatch {
                path: path.to_path_buf(),
                expected: asset.kind,
                found: probed.kind,
            });
        }
        let same_content = asset
            .fingerprint
            .as_ref()
            .is_some_and(|known| known.same_content(&fingerprint));
        if !same_content && !force {
            return Ok(RelinkOutcome::ContentDiffers);
        }
        let changes = AssetPatch {
            original_path: Some(display(path)),
            relative_hint: Some(Assign {
                value: relative_hint(handle.path(), path),
            }),
            // Another content must be checked again by the renderer.
            decodability: (!same_content).then(|| Decodability::unchecked(&probed.info)),
            media_info: Some(Assign {
                value: Some(probed.info),
            }),
            fingerprint: Some(Assign {
                value: Some(fingerprint),
            }),
            missing: Some(false),
            ..AssetPatch::default()
        };
        let operation = UpdateAsset {
            asset: asset_id,
            patch: changes,
        };
        self.commit(
            &handle,
            Actor::User,
            labels::RELINK_MEDIA,
            vec![operation.into()],
        )
        .await?;
        let project = handle.view().await?.project;
        let relinked = find_asset(&project, asset_id)?;
        self.schedule_jobs(handle.path(), project_id, relinked)
            .await;
        Ok(RelinkOutcome::Relinked)
    }

    /// Removes an asset (undoable). The document refuses it while an item uses the asset.
    /// Its jobs keep running: their outputs are cached by content and serve an undo.
    ///
    /// # Errors
    /// Not open, unknown asset, asset in use (`EDIT_REJECTED`), commit errors.
    pub async fn remove_asset(
        &self,
        project_id: ProjectId,
        asset_id: AssetId,
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id)?;
        find_asset(&handle.view().await?.project, asset_id)?;
        let operation = RemoveAsset { asset: asset_id };
        self.commit(
            &handle,
            Actor::User,
            labels::REMOVE_MEDIA,
            vec![operation.into()],
        )
        .await
    }

    /// Records what the renderer found about the decodability of an asset's streams (a system
    /// transaction, skipped when nothing changes).
    ///
    /// # Errors
    /// Not open, unknown asset, commit errors.
    pub async fn report_decodability(&self, report: &DecodabilityReport) -> Result<(), AppError> {
        let handle = self.handle(report.project_id)?;
        let project = handle.view().await?.project;
        let asset = find_asset(&project, report.asset_id)?;
        let decodability = Decodability {
            video: report.video,
            audio: report.audio,
        };
        if asset.decodability == decodability {
            return Ok(());
        }
        let operation = UpdateAsset {
            asset: report.asset_id,
            patch: AssetPatch {
                decodability: Some(decodability),
                ..AssetPatch::default()
            },
        };
        self.commit(
            &handle,
            Actor::System,
            labels::RECORD_DECODABILITY,
            vec![operation.into()],
        )
        .await
    }

    /// Commits `ops` as one transaction of `actor`.
    pub(super) async fn commit(
        &self,
        handle: &ProjectActorHandle,
        actor: Actor,
        label: &str,
        ops: Vec<Operation>,
    ) -> Result<(), AppError> {
        let transaction = Transaction {
            id: NewIds(self.deps.ids.as_ref()).transaction()?,
            actor,
            label: label.to_owned(),
            ops,
        };
        handle.apply(transaction).await.map(drop)
    }
}
