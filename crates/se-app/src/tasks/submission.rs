//! Submitting, cancelling, retrying and following up tasks.

use std::sync::Arc;

use se_domain::{Project, ProjectId, TaskId};
use se_protocol::{CoreEvent, RunKind, SubmitTaskRequest, Task, TaskScope, TaskStatus};

use super::{
    Shared, State, TaskService,
    queue::RunKey,
    task_state::{self, NewTask},
};
use crate::{AppError, new_ids::NewIds};

/// Longest prompt accepted, in characters (a safety net, far above any typed request).
const MAX_PROMPT_CHARS: usize = 100_000;

impl TaskService {
    /// Creates a task and queues its first run (it starts at once when a slot is free).
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`]; [`AppError::InvalidTask`] for an empty prompt or a scope
    /// outside the project; [`AppError::AiUnavailable`] when the provider cannot run tasks;
    /// [`AppError::Storage`].
    pub async fn submit(&self, request: SubmitTaskRequest) -> Result<Task, AppError> {
        let prompt = checked_prompt(&request.prompt)?;
        let settings = self.ai_settings();
        let provider = request.provider.unwrap_or(settings.provider);
        if self.shared.provider(&provider).is_none() {
            return Err(AppError::InvalidTask {
                reason: format!("no AI provider is called {provider}"),
            });
        }
        let handle = self.shared.handle(request.project_id).await?;
        check_scope(&handle.view().await?.project, &request.scope)?;
        self.shared.ensure_usable(&provider).await?;
        let task = task_state::new_task(
            NewTask {
                id: NewIds(self.shared.deps.ids.as_ref()).task()?,
                project_id: request.project_id,
                prompt,
                scope: request.scope,
                provider,
                model: request.model.or(settings.default_model),
                effort: request.effort.or(settings.default_effort),
            },
            self.shared.now(),
        );
        let mut state = self.shared.state.lock().await;
        let project = state.project_mut(request.project_id)?;
        project.handle.tasks().save_task(&task).await?;
        project.tasks.push(task.clone());
        self.shared.queue_and_publish(&mut state, &task);
        Ok(task)
    }

    /// Cancels a queued or running task. A running agent is stopped (its whole process tree);
    /// the task is marked cancelled once it is gone.
    ///
    /// # Errors
    /// [`AppError::ProjectNotOpen`], [`AppError::TaskNotFound`], [`AppError::TaskNotActive`],
    /// [`AppError::Storage`].
    pub async fn cancel(&self, project_id: ProjectId, task_id: TaskId) -> Result<(), AppError> {
        let now = self.shared.now();
        let mut state = self.shared.state.lock().await;
        state.project_mut(project_id)?.task_mut(task_id)?;
        if state.queue.request_cancel(task_id) {
            return Ok(());
        }
        let was_waiting = state.queue.remove_pending(task_id);
        let project = state.project_mut(project_id)?;
        let task = project.task_mut(task_id)?;
        if !was_waiting {
            return Err(AppError::TaskNotActive { task_id });
        }
        task_state::finish_run(task, TaskStatus::Cancelled, now);
        let task = task.clone();
        project.handle.tasks().save_task(&task).await?;
        self.shared
            .deps
            .events
            .publish(CoreEvent::TaskUpdated { task });
        Ok(())
    }

    /// Runs the last prompt of a failed or cancelled task again (resuming its session when an
    /// earlier run completed).
    ///
    /// # Errors
    /// [`AppError::TaskStillActive`], [`AppError::AiUnavailable`] and those of
    /// [`TaskService::cancel`].
    pub async fn retry(&self, project_id: ProjectId, task_id: TaskId) -> Result<Task, AppError> {
        self.queue_new_run(project_id, task_id, RunKind::Retry, None)
            .await
    }

    /// Sends a new prompt to a finished task's agent, resuming its session.
    ///
    /// # Errors
    /// [`AppError::TaskNotResumable`] when the task has no session or its provider cannot resume;
    /// [`AppError::InvalidTask`] for an empty prompt; those of [`TaskService::retry`].
    pub async fn follow_up(
        &self,
        project_id: ProjectId,
        task_id: TaskId,
        prompt: &str,
    ) -> Result<Task, AppError> {
        let prompt = checked_prompt(prompt)?;
        self.queue_new_run(project_id, task_id, RunKind::FollowUp, Some(prompt))
            .await
    }

    async fn queue_new_run(
        &self,
        project_id: ProjectId,
        task_id: TaskId,
        run_kind: RunKind,
        prompt: Option<String>,
    ) -> Result<Task, AppError> {
        let provider = {
            let mut state = self.shared.state.lock().await;
            let task = state.project_mut(project_id)?.task_mut(task_id)?;
            task.provider.clone()
        };
        self.shared.ensure_usable(&provider).await?;
        let now = self.shared.now();
        let mut state = self.shared.state.lock().await;
        let project = state.project_mut(project_id)?;
        let task = project.task_mut(task_id)?;
        if task_state::is_active(&task.status) {
            return Err(AppError::TaskStillActive { task_id });
        }
        let prompt = match prompt {
            Some(prompt) => {
                self.shared.ensure_resumable(task)?;
                prompt
            }
            None => task
                .runs
                .last()
                .map_or_else(|| task.prompt.clone(), |run| run.prompt.clone()),
        };
        task_state::queue_run(task, run_kind, prompt, now);
        let task = task.clone();
        project.handle.tasks().save_task(&task).await?;
        self.shared.queue_and_publish(&mut state, &task);
        Ok(task)
    }
}

impl Shared {
    /// The actor of an open project.
    pub(super) async fn handle(
        &self,
        project_id: ProjectId,
    ) -> Result<crate::ProjectActorHandle, AppError> {
        let mut state = self.state.lock().await;
        Ok(state.project_mut(project_id)?.handle.clone())
    }

    /// Fails unless a follow-up can resume the task's session.
    fn ensure_resumable(&self, task: &Task) -> Result<(), AppError> {
        let not_resumable = |reason: &str| AppError::TaskNotResumable {
            task_id: task.id,
            reason: reason.to_owned(),
        };
        let resumes = self
            .provider(&task.provider)
            .is_some_and(|provider| provider.capabilities().resume_sessions);
        if !resumes {
            return Err(not_resumable(
                "its AI provider cannot resume sessions: submit a new task",
            ));
        }
        if task.agent_session_id.is_none() {
            return Err(not_resumable(
                "its agent never started a session: retry it or submit a new task",
            ));
        }
        Ok(())
    }

    /// Publishes the task and queues its latest run.
    fn queue_and_publish(self: &Arc<Self>, state: &mut State, task: &Task) {
        self.deps
            .events
            .publish(CoreEvent::TaskUpdated { task: task.clone() });
        state.queue.push(RunKey {
            project_id: task.project_id,
            task_id: task.id,
        });
        self.dispatch(state);
    }

    /// Starts waiting runs while slots are free.
    pub(super) fn dispatch(self: &Arc<Self>, state: &mut State) {
        while let Some(key) = state.queue.next_to_start() {
            let (cancelled, gone) = state.queue.mark_running(key);
            tokio::spawn(super::runner::execute(
                Arc::clone(self),
                key,
                cancelled,
                gone,
            ));
        }
    }
}

/// The prompt trimmed, refused when empty or absurdly long.
fn checked_prompt(prompt: &str) -> Result<String, AppError> {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Err(AppError::InvalidTask {
            reason: "the prompt is empty: say what the agent must do".to_owned(),
        });
    }
    if prompt.chars().count() > MAX_PROMPT_CHARS {
        return Err(AppError::InvalidTask {
            reason: format!("the prompt is longer than {MAX_PROMPT_CHARS} characters: shorten it"),
        });
    }
    Ok(prompt.to_owned())
}

/// Fails unless the scope names a sequence of the project, tracks of that sequence and a
/// non-empty range.
fn check_scope(project: &Project, scope: &TaskScope) -> Result<(), AppError> {
    let invalid = |reason: String| AppError::InvalidTask { reason };
    let sequence = project
        .sequences
        .iter()
        .find(|sequence| sequence.id == scope.sequence_id)
        .ok_or_else(|| {
            invalid(format!(
                "the project has no sequence {}: pick an existing sequence",
                scope.sequence_id
            ))
        })?;
    if scope.range.is_some_and(|range| range.duration().get() <= 0) {
        return Err(invalid(
            "the task's time range is empty: select a range, or none for the whole sequence"
                .to_owned(),
        ));
    }
    for track_id in scope.tracks.iter().flatten() {
        if !sequence.tracks.iter().any(|track| track.id == *track_id) {
            return Err(invalid(format!(
                "sequence {} has no track {track_id}: pick tracks of that sequence",
                sequence.name
            )));
        }
    }
    if scope.tracks.as_ref().is_some_and(Vec::is_empty) {
        return Err(invalid(
            "the task's track list is empty: pick tracks, or none for every track".to_owned(),
        ));
    }
    Ok(())
}
