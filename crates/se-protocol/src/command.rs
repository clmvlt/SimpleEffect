//! Commands a client asks `se-core` to execute.

use se_domain::{Format, ProjectId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::CreateProjectRequest;

/// A command sent inside a [`crate::RequestEnvelope`], tagged by `kind` (camelCase).
///
/// Every variant has a [`crate::CommandResult`] variant **with the same name**: that pairing is
/// how the TypeScript client derives the result type of each command without a hand-written table
/// (ADR-0012). Adding a command: see the recipe in `crates/se-protocol/CLAUDE.md`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Command {
    /// Liveness check. Answered by [`crate::CommandResult::Ping`].
    Ping,
    /// Describes the running core (version, platform, folders). Answered by
    /// [`crate::CommandResult::GetCoreInfo`].
    GetCoreInfo,
    /// Choices for the « new project » dialog (folder, formats, frame rates). Answered by
    /// [`crate::CommandResult::GetNewProjectDefaults`].
    GetNewProjectDefaults,
    /// Creates a project folder with a default sequence (tracks V1, V2, A1, A2) and opens it.
    /// Answered by [`crate::CommandResult::CreateProject`].
    CreateProject(CreateProjectRequest),
    /// Opens the project folder at `path` (replaying its journal after a crash). Opening a project
    /// this core already has open returns its current session. Answered by
    /// [`crate::CommandResult::OpenProject`].
    OpenProject {
        /// Absolute path of a `.seproj` folder.
        path: String,
    },
    /// Saves and closes an open project. Answered by [`crate::CommandResult::CloseProject`].
    CloseProject {
        /// The open project.
        project_id: ProjectId,
    },
    /// The recent projects, most recently opened first. Answered by
    /// [`crate::CommandResult::ListRecentProjects`].
    ListRecentProjects,
    /// Renames a project (open or not; its folder keeps its name). Answered by
    /// [`crate::CommandResult::RenameProject`].
    RenameProject {
        /// Absolute path of the project folder.
        path: String,
        /// The new name (not empty).
        name: String,
    },
    /// Copies a project into a new folder next to it, with a new id and `name`. Answered by
    /// [`crate::CommandResult::DuplicateProject`].
    DuplicateProject {
        /// Absolute path of the project folder to copy.
        path: String,
        /// Name of the copy (not empty).
        name: String,
    },
    /// Closes the project if it is open, moves its folder to the OS trash (never deletes it) and
    /// forgets it in the recent list. Answered by [`crate::CommandResult::TrashProject`].
    TrashProject {
        /// Absolute path of the project folder.
        path: String,
    },
    /// The current session of an open project, e.g. after a missed patch or a reconnection.
    /// Answered by [`crate::CommandResult::GetProjectSnapshot`].
    GetProjectSnapshot {
        /// The open project.
        project_id: ProjectId,
    },
    /// Adds a sequence with tracks V1, V2, A1, A2 to an open project (undoable). Answered by
    /// [`crate::CommandResult::CreateSequence`].
    CreateSequence {
        /// The open project.
        project_id: ProjectId,
        /// Name of the sequence (not empty).
        name: String,
        /// Its format; `null` uses the project's default format.
        format: Option<Format>,
    },
    /// Undoes the latest user transaction of an open project. Answered by
    /// [`crate::CommandResult::Undo`].
    Undo {
        /// The open project.
        project_id: ProjectId,
    },
    /// Redoes the latest undone transaction of an open project. Answered by
    /// [`crate::CommandResult::Redo`].
    Redo {
        /// The open project.
        project_id: ProjectId,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_are_tagged_by_camel_case_kind() {
        assert_eq!(
            serde_json::to_string(&Command::GetCoreInfo).unwrap(),
            r#"{"kind":"getCoreInfo"}"#
        );
        let parsed: Command = serde_json::from_str(r#"{"kind":"ping"}"#).unwrap();
        assert_eq!(parsed, Command::Ping);
    }

    #[test]
    fn struct_command_fields_are_camel_case() {
        let undo = Command::Undo {
            project_id: ProjectId::from_test_index(1),
        };
        assert_eq!(
            serde_json::to_string(&undo).unwrap(),
            r#"{"kind":"undo","projectId":"00000000-0001-7000-8000-000000000000"}"#
        );
    }

    #[test]
    fn create_project_carries_its_request_next_to_the_kind() {
        let json = r#"{"kind":"createProject","name":"Film","parentDir":"/p","format":{"width":1920,"height":1080,"frameRate":{"numerator":25,"denominator":1},"sampleRate":48000}}"#;
        let Command::CreateProject(request) = serde_json::from_str(json).unwrap() else {
            panic!("createProject must parse as Command::CreateProject");
        };
        assert_eq!(request.name, "Film");
        assert_eq!(request.parent_dir, "/p");
        assert_eq!(request.format.width, 1920);
    }
}
