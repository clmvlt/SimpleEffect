//! Pure changes of a [`Task`]: new tasks, runs queued, started and finished, interrupted tasks,
//! and which agent session a new run uses. No IO and no clock: times are passed in.

use se_protocol::{
    AgentFailureKind, EffortLevel, ProviderId, RunKind, Task, TaskRun, TaskScope, TaskStatus,
};

use se_domain::{ProjectId, TaskId};

/// What a new task is made of.
pub(super) struct NewTask {
    pub id: TaskId,
    pub project_id: ProjectId,
    pub prompt: String,
    pub scope: TaskScope,
    pub provider: ProviderId,
    pub model: Option<String>,
    pub effort: Option<EffortLevel>,
}

/// How the next run of a task reaches its agent session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SessionPlan {
    /// A new session.
    Fresh,
    /// The task's session, resumed.
    Resume(String),
}

/// A task with its initial run queued at `now`.
pub(super) fn new_task(new: NewTask, now: i64) -> Task {
    let run = queued_run(RunKind::Initial, new.prompt.clone(), now);
    Task {
        id: new.id,
        project_id: new.project_id,
        prompt: new.prompt,
        scope: new.scope,
        provider: new.provider,
        model: new.model,
        effort: new.effort,
        status: TaskStatus::Queued,
        agent_session_id: None,
        created_at: now,
        updated_at: now,
        runs: vec![run],
    }
}

/// Whether a status is still waiting or running.
pub(super) const fn is_active(status: &TaskStatus) -> bool {
    matches!(status, TaskStatus::Queued | TaskStatus::Running)
}

/// Index of the task's latest run (the one events are attributed to).
pub(super) fn current_run(task: &Task) -> u32 {
    u32::try_from(task.runs.len().saturating_sub(1)).unwrap_or(u32::MAX)
}

/// Appends a queued run.
pub(super) fn queue_run(task: &mut Task, run_kind: RunKind, prompt: String, now: i64) {
    task.runs.push(queued_run(run_kind, prompt, now));
    set_status(task, TaskStatus::Queued, now);
}

/// Marks the latest run started.
pub(super) fn start_run(task: &mut Task, now: i64) {
    if let Some(run) = task.runs.last_mut() {
        run.started_at = Some(now);
    }
    set_status(task, TaskStatus::Running, now);
}

/// Ends the latest run with `status` (a terminal status).
pub(super) fn finish_run(task: &mut Task, status: TaskStatus, now: i64) {
    if let Some(run) = task.runs.last_mut() {
        run.finished_at = Some(now);
    }
    set_status(task, status, now);
}

/// Fails a task left queued or running by a core that stopped; returns whether it changed.
pub(super) fn interrupt(task: &mut Task, now: i64) -> bool {
    let message = match task.status {
        TaskStatus::Queued => {
            "SimpleEffect stopped before this task started: retry it to run it now"
        }
        TaskStatus::Running => {
            "SimpleEffect stopped while this task was running: retry it (edits already made are \
             kept)"
        }
        _ => return false,
    };
    finish_run(
        task,
        TaskStatus::Failed {
            failure: AgentFailureKind::Interrupted,
            message: message.to_owned(),
        },
        now,
    );
    true
}

/// The session the task's latest run uses: follow-ups resume the task's session, and so do
/// retries once a run of the task completed (its session holds a finished exchange); other runs
/// start a new session.
pub(super) fn session_plan(task: &Task) -> SessionPlan {
    let Some(latest) = task.runs.last() else {
        return SessionPlan::Fresh;
    };
    let earlier = &task.runs[..task.runs.len() - 1];
    let resumes = match latest.run_kind {
        RunKind::Initial => false,
        RunKind::FollowUp => true,
        RunKind::Retry => earlier
            .iter()
            .any(|run| run.status == TaskStatus::Completed),
    };
    match (&task.agent_session_id, resumes) {
        (Some(session), true) => SessionPlan::Resume(session.clone()),
        _ => SessionPlan::Fresh,
    }
}

fn queued_run(run_kind: RunKind, prompt: String, now: i64) -> TaskRun {
    TaskRun {
        run_kind,
        prompt,
        status: TaskStatus::Queued,
        queued_at: now,
        started_at: None,
        finished_at: None,
    }
}

/// Sets the status of the task and of its latest run (always equal).
fn set_status(task: &mut Task, status: TaskStatus, now: i64) {
    if let Some(run) = task.runs.last_mut() {
        run.status = status.clone();
    }
    task.status = status;
    task.updated_at = now;
}

#[cfg(test)]
mod tests {
    use se_domain::SequenceId;

    use super::*;

    fn task() -> Task {
        new_task(
            NewTask {
                id: TaskId::from_test_index(1),
                project_id: ProjectId::from_test_index(2),
                prompt: "Cut the silences".to_owned(),
                scope: TaskScope {
                    sequence_id: SequenceId::from_test_index(3),
                    range: None,
                    tracks: None,
                },
                provider: ProviderId("fake".to_owned()),
                model: None,
                effort: None,
            },
            10,
        )
    }

    #[test]
    fn a_run_goes_from_queued_to_finished() {
        let mut task = task();
        assert_eq!(task.status, TaskStatus::Queued);
        start_run(&mut task, 20);
        assert_eq!(task.runs[0].status, TaskStatus::Running);
        finish_run(&mut task, TaskStatus::Completed, 30);
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.runs[0].started_at, Some(20));
        assert_eq!(task.runs[0].finished_at, Some(30));
        assert_eq!(task.updated_at, 30);
        assert_eq!(current_run(&task), 0);
    }

    #[test]
    fn retries_start_fresh_until_a_run_completed() {
        let mut task = task();
        task.agent_session_id = Some("first".to_owned());
        finish_run(&mut task, TaskStatus::Cancelled, 11);
        let prompt = task.prompt.clone();
        queue_run(&mut task, RunKind::Retry, prompt, 12);
        assert_eq!(session_plan(&task), SessionPlan::Fresh);
        finish_run(&mut task, TaskStatus::Completed, 13);
        queue_run(&mut task, RunKind::FollowUp, "Shorter".to_owned(), 14);
        assert_eq!(session_plan(&task), SessionPlan::Resume("first".to_owned()));
        let failed = TaskStatus::Failed {
            failure: AgentFailureKind::RateLimited,
            message: "later".to_owned(),
        };
        finish_run(&mut task, failed, 15);
        queue_run(&mut task, RunKind::Retry, "Shorter".to_owned(), 16);
        assert_eq!(session_plan(&task), SessionPlan::Resume("first".to_owned()));
        assert_eq!(current_run(&task), 3);
    }

    #[test]
    fn only_active_tasks_are_interrupted() {
        let mut queued = task();
        assert!(interrupt(&mut queued, 50));
        assert!(matches!(
            queued.status,
            TaskStatus::Failed {
                failure: AgentFailureKind::Interrupted,
                ..
            }
        ));
        assert!(!interrupt(&mut queued, 60));
        assert_eq!(queued.updated_at, 50);
    }
}
