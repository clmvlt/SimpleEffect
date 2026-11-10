//! Projects as the home screen and the editor see them: open sessions, undo state, recent
//! projects and the choices offered when creating one.

use se_domain::{Format, FormatPreset, FrameRate, ProjectId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ProjectSnapshot;

/// An open project: where it lives, its entities and its undo state. Answers `createProject`,
/// `openProject` and `getProjectSnapshot`, and is pushed as [`crate::CoreEvent::ProjectOpened`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSession {
    /// Absolute path of the project folder (`…/<Name>.seproj`).
    pub path: String,
    /// Every entity at the current revision.
    pub snapshot: ProjectSnapshot,
    /// What undo and redo would do now.
    pub history: HistoryState,
    /// How opening recovered the project after an unexpected stop (prompt 29); `null` when it
    /// had been closed cleanly. Kept for as long as the project stays open.
    pub recovery: Option<ProjectRecovery>,
}

/// What opening a project had to recover: commits replayed from the journal (the app or the
/// core stopped before saving them) and, when `project.json` was unreadable, the backup it was
/// restored from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRecovery {
    /// Changes replayed from the journal on top of the snapshot.
    pub replayed_changes: u32,
    /// The backup file (`project.json.bak.1`…) the document was restored from, when
    /// `project.json` could not be read; that file was kept as `project.json.corrupted`.
    pub restored_from: Option<String>,
    /// Why `project.json` could not be read.
    pub problem: Option<String>,
    /// Journal changes that could not be replayed on the backup (lost).
    pub lost_changes: u32,
}

/// What undo and redo would do now, for menus and shortcuts. Pushed as
/// [`crate::CoreEvent::HistoryChanged`] after every commit.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct HistoryState {
    /// Whether `undo` has a user transaction to revert.
    pub can_undo: bool,
    /// Whether `redo` has an undone transaction to replay.
    pub can_redo: bool,
    /// Label of the transaction `undo` reverts (`Create sequence`).
    pub undo_label: Option<String>,
    /// Label of the transaction `redo` replays.
    pub redo_label: Option<String>,
}

/// [`HistoryState`] of one project, as pushed by [`crate::CoreEvent::HistoryChanged`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProjectHistory {
    /// The project whose history changed.
    pub project_id: ProjectId,
    /// Its new undo state.
    pub history: HistoryState,
}

/// Whether an open project's snapshot can be saved, as pushed by
/// [`crate::CoreEvent::ProjectSaveStateChanged`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSaveState {
    /// The project.
    pub project_id: ProjectId,
    /// Why the last save failed; `null` once a save succeeded again.
    pub failure: Option<String>,
    /// Changes not in `project.json` yet (kept by the journal).
    pub unsaved_changes: u32,
}

/// A project of the home screen's recent list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RecentProject {
    /// Stable id of the project.
    pub id: ProjectId,
    /// Its name.
    pub name: String,
    /// Absolute path of its folder.
    pub path: String,
    /// Its default sequence format (shown as a badge: 16:9, 9:16…).
    pub format: Format,
    /// When it was last opened (or created), in milliseconds since the Unix epoch.
    pub last_opened_at: i64,
    /// Whether the folder is still there (it may have been moved or deleted outside the app).
    pub available: bool,
}

/// Choices offered by the « new project » dialog, so the renderer never duplicates domain lists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct NewProjectDefaults {
    /// Folder proposed for the new project: the last one used, or `<Documents>/SimpleEffect`.
    pub parent_dir: String,
    /// Format proposed for the new project.
    pub format: Format,
    /// Named format presets and their dimensions.
    pub presets: Vec<FormatPresetDimensions>,
    /// Frame rates the domain supports exactly, slowest first.
    pub frame_rates: Vec<FrameRate>,
}

/// A named format preset and its frame size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FormatPresetDimensions {
    /// The preset.
    pub preset: FormatPreset,
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
}

/// Payload of [`crate::Command::CreateProject`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectRequest {
    /// Name of the project (not empty); its folder is `<parentDir>/<name>.seproj`.
    pub name: String,
    /// Absolute path of the folder receiving the project folder (created if missing).
    pub parent_dir: String,
    /// Format of the project's default sequence (and of new sequences).
    pub format: Format,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_projects_use_camel_case_and_millisecond_dates() {
        let recent = RecentProject {
            id: ProjectId::from_test_index(1),
            name: "Film".to_owned(),
            path: "/projects/Film.seproj".to_owned(),
            format: Format::from_preset(FormatPreset::Hd16x9, FrameRate::FPS_25).unwrap(),
            last_opened_at: 1_700_000_000_000,
            available: true,
        };
        let json = serde_json::to_value(&recent).unwrap();
        assert_eq!(json["lastOpenedAt"], 1_700_000_000_000_i64);
        assert_eq!(json["format"]["width"], 1920);
        assert_eq!(
            serde_json::from_value::<RecentProject>(json).unwrap(),
            recent
        );
    }

    #[test]
    fn history_state_labels_are_null_when_absent() {
        let json = serde_json::to_string(&HistoryState::default()).unwrap();
        assert_eq!(
            json,
            r#"{"canUndo":false,"canRedo":false,"undoLabel":null,"redoLabel":null}"#
        );
    }
}
