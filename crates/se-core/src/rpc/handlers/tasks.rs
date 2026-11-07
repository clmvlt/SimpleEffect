//! AI task commands (prompt 17): submit, cancel, retry, follow up, list, event logs.

use se_domain::{ProjectId, TaskId};
use se_protocol::{CommandResult, ProtocolError, SubmitTaskRequest};

use super::HandlerContext;

/// `submitTask`.
pub async fn submit_task(
    context: &HandlerContext,
    request: &SubmitTaskRequest,
) -> Result<CommandResult, ProtocolError> {
    let task = context.tasks.submit(request.clone()).await?;
    Ok(CommandResult::SubmitTask { task })
}

/// `cancelTask`.
pub async fn cancel_task(
    context: &HandlerContext,
    project_id: ProjectId,
    task_id: TaskId,
) -> Result<CommandResult, ProtocolError> {
    context.tasks.cancel(project_id, task_id).await?;
    Ok(CommandResult::CancelTask)
}

/// `retryTask`.
pub async fn retry_task(
    context: &HandlerContext,
    project_id: ProjectId,
    task_id: TaskId,
) -> Result<CommandResult, ProtocolError> {
    let task = context.tasks.retry(project_id, task_id).await?;
    Ok(CommandResult::RetryTask { task })
}

/// `followUpTask`.
pub async fn follow_up_task(
    context: &HandlerContext,
    project_id: ProjectId,
    task_id: TaskId,
    prompt: &str,
) -> Result<CommandResult, ProtocolError> {
    let task = context.tasks.follow_up(project_id, task_id, prompt).await?;
    Ok(CommandResult::FollowUpTask { task })
}

/// `listTasks`.
pub async fn list_tasks(
    context: &HandlerContext,
    project_id: ProjectId,
) -> Result<CommandResult, ProtocolError> {
    let tasks = context.tasks.list(project_id).await?;
    Ok(CommandResult::ListTasks { tasks })
}

/// `getTaskEvents`.
pub async fn get_task_events(
    context: &HandlerContext,
    project_id: ProjectId,
    task_id: TaskId,
) -> Result<CommandResult, ProtocolError> {
    let events = context.tasks.events(project_id, task_id).await?;
    Ok(CommandResult::GetTaskEvents { events })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use se_domain::{Format, FormatPreset, FrameRate};
    use se_protocol::{ErrorCode, TaskScope, TaskStatus};

    use super::*;
    use crate::rpc::router::testing::context;

    #[tokio::test]
    async fn tasks_are_submitted_listed_and_cancelled_through_the_handlers() {
        let context = context();
        let format = Format::from_preset(FormatPreset::Hd16x9, FrameRate::FPS_25).unwrap();
        let session = context
            .projects
            .create_project("Film", Path::new("/projects"), format)
            .await
            .unwrap();
        let project_id = session.snapshot.project.id;
        let request = SubmitTaskRequest {
            project_id,
            prompt: "Describe the timeline".to_owned(),
            scope: TaskScope {
                sequence_id: session.snapshot.sequences[0].id,
                range: None,
                tracks: None,
            },
            provider: None,
            model: None,
            effort: None,
        };
        let CommandResult::SubmitTask { task } = submit_task(&context, &request).await.unwrap()
        else {
            panic!("submitTask must answer with the task");
        };
        assert_eq!(task.status, TaskStatus::Queued);
        let CommandResult::ListTasks { tasks } = list_tasks(&context, project_id).await.unwrap()
        else {
            panic!("listTasks must answer with the tasks");
        };
        assert_eq!(tasks.len(), 1);
        let unknown = cancel_task(&context, project_id, TaskId::from_test_index(9)).await;
        assert_eq!(unknown.unwrap_err().code, ErrorCode::TaskNotFound);
        let closed = list_tasks(&context, ProjectId::from_test_index(7)).await;
        assert_eq!(closed.unwrap_err().code, ErrorCode::ProjectNotOpen);
    }
}
