//! Successful results of commands.

use se_domain::{ItemId, LinkGroupId, MarkerId, SequenceId, TrackId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    ImportReport, JobSnapshot, MediaSettings, MediaToolsStatus, NewProjectDefaults,
    ProjectCacheInfo, ProjectSession, RecentProject, RelinkOutcome,
};

/// The result of a [`crate::Command`], tagged by the **same `kind` as the command** it answers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CommandResult {
    /// Answer to [`crate::Command::Ping`]: the core is alive and reading requests.
    Ping,
    /// Answer to [`crate::Command::GetCoreInfo`].
    GetCoreInfo(CoreInfo),
    /// Answer to [`crate::Command::GetNewProjectDefaults`].
    GetNewProjectDefaults(NewProjectDefaults),
    /// Answer to [`crate::Command::CreateProject`]: the new project, open.
    CreateProject(ProjectSession),
    /// Answer to [`crate::Command::OpenProject`].
    OpenProject(ProjectSession),
    /// Answer to [`crate::Command::CloseProject`]: saved and closed.
    CloseProject,
    /// Answer to [`crate::Command::ListRecentProjects`].
    ListRecentProjects {
        /// Most recently opened first.
        projects: Vec<RecentProject>,
    },
    /// Answer to [`crate::Command::RenameProject`]: the project as listed after renaming.
    RenameProject(RecentProject),
    /// Answer to [`crate::Command::DuplicateProject`]: the copy, now first in the recent list.
    DuplicateProject(RecentProject),
    /// Answer to [`crate::Command::TrashProject`]: the folder is in the OS trash.
    TrashProject,
    /// Answer to [`crate::Command::GetProjectSnapshot`].
    GetProjectSnapshot(ProjectSession),
    /// Answer to [`crate::Command::CreateSequence`].
    CreateSequence {
        /// Id of the new sequence.
        sequence_id: SequenceId,
    },
    /// Answer to [`crate::Command::Undo`]: committed (the patch arrives as an event).
    Undo,
    /// Answer to [`crate::Command::Redo`]: committed (the patch arrives as an event).
    Redo,
    /// Answer to [`crate::Command::AddTrack`].
    AddTrack {
        /// Id of the new track.
        track_id: TrackId,
    },
    /// Answer to [`crate::Command::RemoveTrack`]: committed.
    RemoveTrack,
    /// Answer to [`crate::Command::UpdateTrack`]: committed.
    UpdateTrack,
    /// Answer to [`crate::Command::ReorderTracks`]: committed.
    ReorderTracks,
    /// Answer to [`crate::Command::AddMarker`].
    AddMarker {
        /// Id of the new marker.
        marker_id: MarkerId,
    },
    /// Answer to [`crate::Command::UpdateMarker`]: committed.
    UpdateMarker,
    /// Answer to [`crate::Command::RemoveMarker`]: committed.
    RemoveMarker,
    /// Answer to [`crate::Command::PlaceMedia`].
    PlaceMedia {
        /// Ids of the new items, in placement order (picture before sound for each asset).
        item_ids: Vec<ItemId>,
    },
    /// Answer to [`crate::Command::MoveItems`]: committed.
    MoveItems,
    /// Answer to [`crate::Command::DuplicateItems`].
    DuplicateItems {
        /// Ids of the copies, in the order of the request.
        item_ids: Vec<ItemId>,
    },
    /// Answer to [`crate::Command::PasteItems`].
    PasteItems {
        /// Ids of the copies, in the order of the request.
        item_ids: Vec<ItemId>,
    },
    /// Answer to [`crate::Command::TrimItems`]: committed.
    TrimItems,
    /// Answer to [`crate::Command::SplitItems`].
    SplitItems {
        /// Ids of the new right parts, in the order of the request.
        item_ids: Vec<ItemId>,
    },
    /// Answer to [`crate::Command::DeleteItems`]: committed.
    DeleteItems,
    /// Answer to [`crate::Command::LinkItems`].
    LinkItems {
        /// The new link group.
        link_group: LinkGroupId,
    },
    /// Answer to [`crate::Command::UnlinkItems`]: committed.
    UnlinkItems,
    /// Answer to [`crate::Command::ResizeGap`]: committed (nothing when the length is unchanged).
    ResizeGap,
    /// Answer to [`crate::Command::ImportMedia`].
    ImportMedia(ImportReport),
    /// Answer to [`crate::Command::RelinkAsset`].
    RelinkAsset {
        /// Whether the asset was relinked or the user must confirm first.
        outcome: RelinkOutcome,
    },
    /// Answer to [`crate::Command::RemoveAsset`]: committed (the patch arrives as an event).
    RemoveAsset,
    /// Answer to [`crate::Command::ListJobs`].
    ListJobs {
        /// Oldest first.
        jobs: Vec<JobSnapshot>,
    },
    /// Answer to [`crate::Command::CancelJob`]: cancelled (`jobCancelled` follows).
    CancelJob,
    /// Answer to [`crate::Command::ReportDecodability`]: recorded.
    ReportDecodability,
    /// Answer to [`crate::Command::ReportDecoderCapabilities`]: recorded.
    ReportDecoderCapabilities,
    /// Answer to [`crate::Command::GenerateDerivedMedia`]: queued (the asset's derived state
    /// becomes pending; job events follow).
    GenerateDerivedMedia,
    /// Answer to [`crate::Command::GetMediaSettings`].
    GetMediaSettings {
        /// The current settings.
        settings: MediaSettings,
    },
    /// Answer to [`crate::Command::UpdateMediaSettings`].
    UpdateMediaSettings {
        /// The settings as saved.
        settings: MediaSettings,
    },
    /// Answer to [`crate::Command::GetMediaToolsStatus`].
    GetMediaToolsStatus {
        /// Tools found and hardware encoders.
        status: MediaToolsStatus,
    },
    /// Answer to [`crate::Command::GetProjectCache`].
    GetProjectCache {
        /// The cache folder and its size.
        cache: ProjectCacheInfo,
    },
    /// Answer to [`crate::Command::ClearProjectCache`].
    ClearProjectCache {
        /// The cache folder and what is left in it (files in use cannot be deleted).
        cache: ProjectCacheInfo,
    },
}

/// Facts about the running core, shown in the UI and useful in bug reports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CoreInfo {
    /// Version of the running `se-core` binary (semver).
    pub version: String,
    /// Operating system, as Rust's `std::env::consts::OS` (`windows`, `macos`, `linux`).
    pub os: String,
    /// CPU architecture, as Rust's `std::env::consts::ARCH` (`x86_64`, `aarch64`).
    pub arch: String,
    /// Absolute path of the app data folder (`SE_DATA_DIR`, set by the Electron app).
    pub app_data_dir: String,
    /// Absolute path of the folder holding the core's daily log files (`<app data>/logs`).
    pub logs_dir: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn results_carry_their_payload_next_to_the_kind() {
        let result = CommandResult::GetCoreInfo(CoreInfo {
            version: "0.1.0".to_owned(),
            os: "linux".to_owned(),
            arch: "x86_64".to_owned(),
            app_data_dir: "/data".to_owned(),
            logs_dir: "/data/logs".to_owned(),
        });
        let json = serde_json::to_string(&result).unwrap();
        assert_eq!(
            json,
            r#"{"kind":"getCoreInfo","version":"0.1.0","os":"linux","arch":"x86_64","appDataDir":"/data","logsDir":"/data/logs"}"#
        );
        assert_eq!(
            serde_json::from_str::<CommandResult>(&json).unwrap(),
            result
        );
    }

    #[test]
    fn struct_result_fields_are_camel_case() {
        let result = CommandResult::CreateSequence {
            sequence_id: SequenceId::from_test_index(2),
        };
        assert_eq!(
            serde_json::to_string(&result).unwrap(),
            r#"{"kind":"createSequence","sequenceId":"00000000-0002-7000-8000-000000000000"}"#
        );
    }

    #[test]
    fn unit_results_are_just_the_kind() {
        assert_eq!(
            serde_json::to_string(&CommandResult::Ping).unwrap(),
            r#"{"kind":"ping"}"#
        );
    }
}
