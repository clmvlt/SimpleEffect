//! Asset operations: [`AddAsset`], [`UpdateAsset`], [`RemoveAsset`].

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    Edit, Effect,
    checks::{ensure_unused, insertion_index, swap_field},
    lookup::{all_items, find_asset},
};
use crate::{
    Asset, AssetId, Decodability, DerivedMedia, DomainError, ItemKind, MediaInfo, Project,
};

/// Adds an asset. Inverse: [`RemoveAsset`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AddAsset {
    /// Position in the project's assets (`null` appends).
    pub index: Option<usize>,
    /// The asset, with an unused id.
    pub asset: Asset,
}

impl Edit for AddAsset {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let id = self.asset.id;
        ensure_unused(find_asset(project, id).is_ok(), "asset", &id)?;
        let index = insertion_index("asset", self.index, project.assets.len())?;
        project.assets.insert(index, self.asset.clone());
        effect.touched.assets.insert(id);
        effect.undo_with(RemoveAsset { asset: id });
        Ok(())
    }
}

/// A new value for a patch field that may itself be `null`: `{"value": null}` clears the field,
/// while a `null` patch field leaves it unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Assign<T> {
    /// The new value.
    pub value: T,
}

/// Changes of an asset; `null` (or absent) fields are unchanged. The kind of an asset cannot
/// change (its items depend on it).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct AssetPatch {
    /// Another path of the original (relinking).
    pub original_path: Option<String>,
    /// Another relative hint, or none.
    pub relative_hint: Option<Assign<Option<String>>>,
    /// Probe results, or none.
    pub media_info: Option<Assign<Option<MediaInfo>>>,
    /// Other mezzanine and proxy states.
    pub derived: Option<DerivedMedia>,
    /// Another decodability.
    pub decodability: Option<Decodability>,
    /// Whether the original is missing.
    pub missing: Option<bool>,
}

impl AssetPatch {
    /// Applies the patch and returns the one undoing it.
    fn apply(&self, asset: &mut Asset) -> Self {
        let relative_hint = self.relative_hint.as_ref().map(|assign| Assign {
            value: std::mem::replace(&mut asset.relative_hint, assign.value.clone()),
        });
        let media_info = self.media_info.as_ref().map(|assign| Assign {
            value: std::mem::replace(&mut asset.media_info, assign.value.clone()),
        });
        Self {
            original_path: swap_field(&mut asset.original_path, self.original_path.as_ref()),
            relative_hint,
            media_info,
            derived: swap_field(&mut asset.derived, self.derived.as_ref()),
            decodability: swap_field(&mut asset.decodability, self.decodability.as_ref()),
            missing: swap_field(&mut asset.missing, self.missing.as_ref()),
        }
    }
}

/// Changes what is known about an asset (path, probe, derived files…).
/// Inverse: [`UpdateAsset`] with the previous values of the changed fields.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAsset {
    /// The asset changed.
    pub asset: AssetId,
    /// What changes.
    pub patch: AssetPatch,
}

impl Edit for UpdateAsset {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let index = find_asset(project, self.asset)?;
        let previous = self.patch.apply(&mut project.assets[index]);
        effect.touched.assets.insert(self.asset);
        effect.undo_with(Self {
            asset: self.asset,
            patch: previous,
        });
        Ok(())
    }
}

/// Removes an asset no item uses. Inverse: [`AddAsset`] at the same position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RemoveAsset {
    /// The asset removed.
    pub asset: AssetId,
}

impl Edit for RemoveAsset {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        let index = find_asset(project, self.asset)?;
        let user = all_items(project).find(|item| match &item.kind {
            ItemKind::MediaClip(clip) => clip.asset_id == self.asset,
            ItemKind::Image(image) => image.asset_id == self.asset,
            _ => false,
        });
        if let Some(user) = user {
            return Err(DomainError::AssetInUse {
                asset: self.asset,
                item: user.id,
            });
        }
        let asset = project.assets.remove(index);
        effect.touched.assets.insert(self.asset);
        effect.undo_with(AddAsset {
            index: Some(index),
            asset,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AssetKind, ItemId,
        operation::testing::{apply_err, apply_ok, example},
        test_support::example_ids,
    };

    #[test]
    fn adds_and_removes_unused_assets() {
        let mut asset = example().assets[2].clone();
        asset.id = AssetId::from_test_index(10);
        let applied = apply_ok(
            &example(),
            AddAsset {
                index: Some(1),
                asset,
            },
        );
        assert_eq!(applied.project.assets[1].id, AssetId::from_test_index(10));
        let applied = apply_ok(
            &applied.project,
            RemoveAsset {
                asset: AssetId::from_test_index(10),
            },
        );
        assert_eq!(applied.project.assets.len(), 3);
    }

    #[test]
    fn assets_in_use_stay() {
        let error = apply_err(
            &example(),
            RemoveAsset {
                asset: example_ids::IMAGE_ASSET,
            },
        );
        assert_eq!(
            error,
            DomainError::AssetInUse {
                asset: example_ids::IMAGE_ASSET,
                item: ItemId::from_test_index(4)
            }
        );
    }

    #[test]
    fn clears_nullable_fields_only_when_assigned() {
        let patch = AssetPatch {
            relative_hint: Some(Assign { value: None }),
            missing: Some(true),
            ..AssetPatch::default()
        };
        let applied = apply_ok(
            &example(),
            UpdateAsset {
                asset: example_ids::AUDIO_ASSET,
                patch,
            },
        );
        let asset = &applied.project.assets[1];
        assert_eq!(asset.relative_hint, None);
        assert!(asset.missing);
        assert!(asset.media_info.is_some());
        assert_eq!(asset.kind, AssetKind::Audio);
    }
}
