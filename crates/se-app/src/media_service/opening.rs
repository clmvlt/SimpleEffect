//! When a project opens: flag originals that are missing (relinking those found through their
//! relative hint), then queue the derived files that are not cached.

use se_domain::{Actor, Asset, AssetPatch, Operation, UpdateAsset};

use super::{MediaService, import::display, labels, media_paths::resolve_hint};
use crate::project_actor::ProjectActorHandle;

impl MediaService {
    /// Checks the originals of an opened project and queues missing derived files. Failures are
    /// logged: opening must not fail because of media.
    pub(super) async fn check_media_files(&self, handle: &ProjectActorHandle) {
        let project = match handle.view().await {
            Ok(view) => view.project,
            Err(error) => {
                tracing::warn!(%error, "reading a project to check its media failed");
                return;
            }
        };
        let mut ops: Vec<Operation> = Vec::new();
        for asset in &project.assets {
            if let Some(patch) = self.file_check(handle, asset).await {
                ops.push(
                    UpdateAsset {
                        asset: asset.id,
                        patch,
                    }
                    .into(),
                );
            }
        }
        if !ops.is_empty() {
            let changed = ops.len();
            match self
                .commit(handle, Actor::System, labels::DETECT_MISSING_MEDIA, ops)
                .await
            {
                Ok(()) => tracing::info!(changed, "media files checked"),
                Err(error) => tracing::warn!(%error, "recording missing media failed"),
            }
        }
        let project = match handle.view().await {
            Ok(view) => view.project,
            Err(error) => {
                tracing::warn!(%error, "reading a project to queue its media jobs failed");
                return;
            }
        };
        for asset in &project.assets {
            self.schedule_jobs(handle.path(), project.id, asset).await;
        }
    }

    /// What must change in `asset` after looking for its original: nothing, a `missing` flag
    /// (set or cleared), or a new path found through the relative hint (same content only).
    async fn file_check(&self, handle: &ProjectActorHandle, asset: &Asset) -> Option<AssetPatch> {
        if self.deps.files.is_file(asset.original_path.as_ref()).await {
            return asset.missing.then(|| missing_patch(false));
        }
        if let Some(found) = asset
            .relative_hint
            .as_deref()
            .and_then(|hint| resolve_hint(handle.path(), hint))
        {
            let same_content = match (&asset.fingerprint, self.deps.files.is_file(&found).await) {
                (Some(known), true) => self
                    .deps
                    .files
                    .fingerprint(&found)
                    .await
                    .is_ok_and(|fingerprint| known.same_content(&fingerprint)),
                _ => false,
            };
            if same_content {
                tracing::info!(asset = %asset.id, path = %found.display(), "media found next to the project");
                return Some(AssetPatch {
                    original_path: Some(display(&found)),
                    missing: Some(false),
                    ..AssetPatch::default()
                });
            }
        }
        (!asset.missing).then(|| missing_patch(true))
    }
}

fn missing_patch(missing: bool) -> AssetPatch {
    AssetPatch {
        missing: Some(missing),
        ..AssetPatch::default()
    }
}
