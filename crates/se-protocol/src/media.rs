//! Media DTOs: import reports, relinking outcomes and the metadata served by `/media` routes.

use se_domain::{AssetId, Flicks, ProjectId, StreamDecodability};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ErrorCode;

/// Arguments of [`crate::Command::ImportMedia`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ImportMediaRequest {
    /// The open project receiving the media.
    pub project_id: ProjectId,
    /// Absolute paths of the files, in the order they should appear in the media bin.
    pub paths: Vec<String>,
}

/// What [`crate::Command::ImportMedia`] did with each path.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    /// Assets created (one undoable transaction), in the order of the paths.
    pub imported: Vec<AssetId>,
    /// Paths whose content is already in the project (or earlier in the same request).
    pub duplicates: Vec<DuplicateMedia>,
    /// Paths that could not be imported, with the reason.
    pub failures: Vec<ImportFailure>,
}

/// A path that was not imported because the project already has its content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateMedia {
    /// The path given.
    pub path: String,
    /// The asset holding the same content.
    pub asset_id: AssetId,
}

/// A path that could not be imported.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ImportFailure {
    /// The path given.
    pub path: String,
    /// Why (`MEDIA_NOT_FOUND`, `MEDIA_UNSUPPORTED`, `MEDIA_TOOLS_MISSING`…).
    pub code: ErrorCode,
    /// Actionable explanation.
    pub message: String,
}

/// Arguments of [`crate::Command::RelinkAsset`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RelinkAssetRequest {
    /// The open project.
    pub project_id: ProjectId,
    /// The asset to point at another file.
    pub asset_id: AssetId,
    /// Absolute path of the replacement file.
    pub path: String,
    /// Relink even when the file's content differs from the asset's fingerprint (the user
    /// confirmed after a [`RelinkOutcome::ContentDiffers`] answer).
    pub force: bool,
}

/// Outcome of [`crate::Command::RelinkAsset`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum RelinkOutcome {
    /// The asset now uses the file (an undoable transaction).
    Relinked,
    /// Nothing changed: the file's content differs from the asset's. Ask the user, then send the
    /// command again with `force: true`.
    ContentDiffers,
}

/// Arguments of [`crate::Command::ReportDecodability`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct DecodabilityReport {
    /// The open project.
    pub project_id: ProjectId,
    /// The asset checked.
    pub asset_id: AssetId,
    /// Whether WebCodecs decodes its picture stream (`noStream` when it has none).
    pub video: StreamDecodability,
    /// Whether WebCodecs decodes its first sound stream (`noStream` when it has none).
    pub audio: StreamDecodability,
}

/// Layout of an asset's thumbnail sheet, served as JSON by `/media/assets/{id}/thumbnails`; the
/// sheet itself (JPEG) is `/media/assets/{id}/thumbnails/sheet` and the poster
/// `/media/assets/{id}/thumbnails/poster`.
///
/// Tiles are laid out left to right, then top to bottom; tile `i` shows the picture at `times[i]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailSheet {
    /// Width of a tile in pixels.
    pub tile_width: u32,
    /// Height of a tile in pixels (90).
    pub tile_height: u32,
    /// Tiles per row.
    pub columns: u32,
    /// Rows of tiles.
    pub rows: u32,
    /// Source time of each tile (`times.len()` tiles; the last row may be incomplete).
    pub times: Vec<Flicks>,
    /// Width of the poster in pixels.
    pub poster_width: u32,
    /// Height of the poster in pixels.
    pub poster_height: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relink_outcomes_are_camel_case_strings() {
        assert_eq!(
            serde_json::to_string(&RelinkOutcome::ContentDiffers).unwrap(),
            r#""contentDiffers""#
        );
    }

    #[test]
    fn import_failures_carry_a_code() {
        let failure = ImportFailure {
            path: "/a.txt".to_owned(),
            code: ErrorCode::MediaUnsupported,
            message: "not media".to_owned(),
        };
        assert_eq!(
            serde_json::to_string(&failure).unwrap(),
            r#"{"path":"/a.txt","code":"MEDIA_UNSUPPORTED","message":"not media"}"#
        );
    }
}
