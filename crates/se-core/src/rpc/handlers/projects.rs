//! Project commands of the home screen: defaults, create, open, close, recent list, rename,
//! duplicate, trash.

use se_domain::ProjectId;
use se_protocol::{CommandResult, CreateProjectRequest, ProtocolError};

use super::{HandlerContext, absolute_path};

/// `getNewProjectDefaults`.
pub async fn get_new_project_defaults(
    context: &HandlerContext,
) -> Result<CommandResult, ProtocolError> {
    let defaults = context.projects.new_project_defaults().await?;
    Ok(CommandResult::GetNewProjectDefaults(defaults))
}

/// `setProjectsFolder` (first-run wizard, prompt 30).
pub async fn set_projects_folder(
    context: &HandlerContext,
    path: &str,
) -> Result<CommandResult, ProtocolError> {
    let defaults = context
        .projects
        .set_projects_folder(absolute_path(path)?)
        .await?;
    Ok(CommandResult::SetProjectsFolder { defaults })
}

/// `createProject`.
pub async fn create_project(
    context: &HandlerContext,
    request: &CreateProjectRequest,
) -> Result<CommandResult, ProtocolError> {
    let parent_dir = absolute_path(&request.parent_dir)?;
    let session = context
        .projects
        .create_project(&request.name, parent_dir, request.format)
        .await?;
    Ok(CommandResult::CreateProject(session))
}

/// `openProject`.
pub async fn open_project(
    context: &HandlerContext,
    path: &str,
) -> Result<CommandResult, ProtocolError> {
    let session = context.projects.open_project(absolute_path(path)?).await?;
    Ok(CommandResult::OpenProject(session))
}

/// `closeProject`.
pub async fn close_project(
    context: &HandlerContext,
    project_id: ProjectId,
) -> Result<CommandResult, ProtocolError> {
    context.projects.close_project(project_id).await?;
    Ok(CommandResult::CloseProject)
}

/// `listRecentProjects`.
pub async fn list_recent_projects(
    context: &HandlerContext,
) -> Result<CommandResult, ProtocolError> {
    let projects = context.projects.list_recent().await?;
    Ok(CommandResult::ListRecentProjects { projects })
}

/// `renameProject`.
pub async fn rename_project(
    context: &HandlerContext,
    path: &str,
    name: &str,
) -> Result<CommandResult, ProtocolError> {
    let project = context
        .projects
        .rename_project(absolute_path(path)?, name)
        .await?;
    Ok(CommandResult::RenameProject(project))
}

/// `duplicateProject`.
pub async fn duplicate_project(
    context: &HandlerContext,
    path: &str,
    name: &str,
) -> Result<CommandResult, ProtocolError> {
    let copy = context
        .projects
        .duplicate_project(absolute_path(path)?, name)
        .await?;
    Ok(CommandResult::DuplicateProject(copy))
}

/// `trashProject`.
pub async fn trash_project(
    context: &HandlerContext,
    path: &str,
) -> Result<CommandResult, ProtocolError> {
    context.projects.trash_project(absolute_path(path)?).await?;
    Ok(CommandResult::TrashProject)
}
