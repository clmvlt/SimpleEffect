//! The AI task service against the replay provider and in-memory ports, through a real
//! `ProjectService` (tasks follow their project's opening and closing).

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use se_app::{
    AppError,
    ports::{AgentError, AgentSession},
    test_support::{MediaTestbed, REPLAY_PROVIDER_ID, ReplayScript, ReplayStep},
};
use se_domain::{ProjectId, SequenceId, TaskId, TrackId};
use se_protocol::{
    AgentEvent, AgentFailureKind, AiSettings, AuthStatus, CoreEvent, ErrorCode, ProtocolError,
    ProviderId, RunKind, SubmitTaskRequest, Task, TaskScope, TaskStatus,
};

const PARENT: &str = "/films";

struct Harness {
    bed: MediaTestbed,
    project_id: ProjectId,
    sequence_id: SequenceId,
    track_id: TrackId,
}

impl std::ops::Deref for Harness {
    type Target = MediaTestbed;

    fn deref(&self) -> &MediaTestbed {
        &self.bed
    }
}

fn project_path() -> PathBuf {
    Path::new(PARENT).join("Film.seproj")
}

fn code(error: AppError) -> ErrorCode {
    ProtocolError::from(error).code
}

async fn harness() -> Harness {
    let bed = MediaTestbed::new(Path::new(PARENT));
    bed.tasks.start().await.unwrap();
    let project_id = bed.create_project(Path::new(PARENT), "Film").await.unwrap();
    let snapshot = bed
        .projects
        .project_session(project_id)
        .await
        .unwrap()
        .snapshot;
    Harness {
        project_id,
        sequence_id: snapshot.sequences[0].id,
        track_id: snapshot.tracks[0].id,
        bed,
    }
}

/// Lets runs progress: paused time jumps forward once every task waits.
async fn run_for(millis: u64) {
    tokio::time::sleep(Duration::from_millis(millis)).await;
}

impl Harness {
    fn request(&self, prompt: &str) -> SubmitTaskRequest {
        SubmitTaskRequest {
            project_id: self.project_id,
            prompt: prompt.to_owned(),
            scope: TaskScope {
                sequence_id: self.sequence_id,
                range: None,
                tracks: None,
            },
            provider: None,
            model: None,
            effort: None,
        }
    }

    async fn submit(&self, prompt: &str) -> TaskId {
        self.tasks.submit(self.request(prompt)).await.unwrap().id
    }

    async fn task(&self, task_id: TaskId) -> Task {
        self.tasks
            .list(self.project_id)
            .await
            .unwrap()
            .into_iter()
            .find(|task| task.id == task_id)
            .unwrap()
    }

    async fn statuses(&self) -> Vec<TaskStatus> {
        let tasks = self.tasks.list(self.project_id).await.unwrap();
        tasks.into_iter().map(|task| task.status).collect()
    }

    async fn event_kinds(&self, task_id: TaskId) -> Vec<String> {
        let events = self.tasks.events(self.project_id, task_id).await.unwrap();
        events
            .iter()
            .map(|record| serde_kind(&record.event))
            .collect()
    }
}

fn serde_kind(event: &AgentEvent) -> String {
    match event {
        AgentEvent::SessionStarted { .. } => "session",
        AgentEvent::AssistantText { .. } => "text",
        AgentEvent::ToolCallStarted { .. } => "toolStart",
        AgentEvent::ToolCallFinished { .. } => "toolEnd",
        AgentEvent::Usage { .. } => "usage",
        AgentEvent::Completed { .. } => "completed",
        AgentEvent::Failed { .. } => "failed",
    }
    .to_owned()
}

fn session_of(session: &AgentSession) -> (&str, &str) {
    match session {
        AgentSession::Start(id) => ("start", id),
        AgentSession::Resume(id) => ("resume", id),
    }
}

#[tokio::test(start_paused = true)]
async fn five_tasks_with_a_limit_of_three_run_three_and_queue_two_in_order() {
    let harness = harness().await;
    harness.agent.set_default_script(ReplayScript::held());
    let mut ids = Vec::new();
    for number in 1..=5 {
        ids.push(harness.submit(&format!("Task {number}")).await);
    }
    run_for(100).await;
    let running = TaskStatus::Running;
    assert_eq!(
        harness.statuses().await,
        [
            running.clone(),
            running.clone(),
            running,
            TaskStatus::Queued,
            TaskStatus::Queued
        ]
    );
    assert_eq!(harness.agent.alive(), 3);
    harness.agent.release(1);
    run_for(100).await;
    assert_eq!(harness.task(ids[0]).await.status, TaskStatus::Completed);
    assert_eq!(harness.task(ids[3]).await.status, TaskStatus::Running);
    assert_eq!(harness.task(ids[4]).await.status, TaskStatus::Queued);
    let prompts: Vec<String> = harness
        .agent
        .specs()
        .into_iter()
        .map(|spec| spec.prompt)
        .collect();
    assert_eq!(prompts, ["Task 1", "Task 2", "Task 3", "Task 4"]);
    harness.agent.release(4);
    run_for(100).await;
    assert!(
        harness
            .statuses()
            .await
            .iter()
            .all(|status| *status == TaskStatus::Completed)
    );
    assert_eq!(harness.agent.alive(), 0);
    assert_eq!(harness.workspaces.released().len(), 5);
}

#[tokio::test(start_paused = true)]
async fn cancelling_a_running_task_stops_its_agent() {
    let harness = harness().await;
    harness.agent.set_default_script(ReplayScript::held());
    let task_id = harness.submit("Long work").await;
    run_for(100).await;
    assert_eq!(harness.agent.alive(), 1);
    harness
        .tasks
        .cancel(harness.project_id, task_id)
        .await
        .unwrap();
    run_for(100).await;
    assert_eq!(harness.task(task_id).await.status, TaskStatus::Cancelled);
    assert_eq!((harness.agent.alive(), harness.agent.killed()), (0, 1));
    assert_eq!(harness.event_kinds(task_id).await, ["session", "failed"]);
    assert_eq!(harness.workspaces.released(), [task_id]);
    let again = harness.tasks.cancel(harness.project_id, task_id).await;
    assert_eq!(code(again.unwrap_err()), ErrorCode::TaskNotActive);
}

#[tokio::test(start_paused = true)]
async fn cancelling_a_queued_task_never_starts_it() {
    let harness = harness().await;
    let mut settings = harness.tasks.ai_settings();
    settings.max_concurrent_tasks = 1;
    harness.tasks.update_ai_settings(settings).await.unwrap();
    harness.agent.set_default_script(ReplayScript::held());
    let first = harness.submit("First").await;
    let second = harness.submit("Second").await;
    harness
        .tasks
        .cancel(harness.project_id, second)
        .await
        .unwrap();
    harness.agent.release(1);
    run_for(100).await;
    assert_eq!(harness.task(first).await.status, TaskStatus::Completed);
    assert_eq!(harness.task(second).await.status, TaskStatus::Cancelled);
    assert_eq!(harness.agent.specs().len(), 1);
    let unknown = harness
        .tasks
        .cancel(harness.project_id, TaskId::from_test_index(9))
        .await;
    assert_eq!(code(unknown.unwrap_err()), ErrorCode::TaskNotFound);
}

#[tokio::test(start_paused = true)]
async fn a_follow_up_resumes_the_session_of_its_task() {
    let harness = harness().await;
    let task_id = harness.submit("Find the best take").await;
    run_for(100).await;
    let task = harness.task(task_id).await;
    assert_eq!(task.status, TaskStatus::Completed);
    let specs = harness.agent.specs();
    let ("start", session) = session_of(&specs[0].session) else {
        panic!("the first run must start a session");
    };
    assert_eq!(task.agent_session_id.as_deref(), Some(session));
    let followed = harness
        .tasks
        .follow_up(harness.project_id, task_id, "Make it shorter")
        .await
        .unwrap();
    assert_eq!(followed.runs.len(), 2);
    assert_eq!(followed.runs[1].run_kind, RunKind::FollowUp);
    run_for(100).await;
    let specs = harness.agent.specs();
    assert_eq!(session_of(&specs[1].session), ("resume", session));
    assert_eq!(specs[1].prompt, "Make it shorter");
    let task = harness.task(task_id).await;
    assert_eq!(task.status, TaskStatus::Completed);
    assert_eq!(task.agent_session_id.as_deref(), Some(session));
    let events = harness
        .tasks
        .events(harness.project_id, task_id)
        .await
        .unwrap();
    assert_eq!(events.last().unwrap().run, 1);
    let sequences: Vec<u32> = events.iter().map(|record| record.sequence).collect();
    assert_eq!(sequences, (0..8).collect::<Vec<u32>>());
}

#[tokio::test(start_paused = true)]
async fn a_retry_runs_the_failed_prompt_again_in_a_new_session() {
    let harness = harness().await;
    harness.agent.push_script(ReplayScript::failing(
        AgentFailureKind::RateLimited,
        "limit reached",
    ));
    let task_id = harness.submit("Add captions").await;
    run_for(100).await;
    let task = harness.task(task_id).await;
    assert_eq!(
        task.status,
        TaskStatus::Failed {
            failure: AgentFailureKind::RateLimited,
            message: "limit reached".to_owned()
        }
    );
    let follow_up = harness
        .tasks
        .follow_up(harness.project_id, task_id, "Anyway")
        .await;
    assert!(
        follow_up.is_ok(),
        "a session was announced, so it can be resumed"
    );
    run_for(100).await;
    harness.agent.push_script(ReplayScript::held());
    let retried = harness
        .tasks
        .retry(harness.project_id, task_id)
        .await
        .unwrap();
    assert_eq!(retried.runs.last().unwrap().prompt, "Anyway");
    run_for(100).await;
    let still_running = harness.tasks.retry(harness.project_id, task_id).await;
    assert_eq!(code(still_running.unwrap_err()), ErrorCode::TaskStillActive);
    harness.agent.release(1);
    run_for(100).await;
    let specs = harness.agent.specs();
    assert_eq!(specs.len(), 3);
    assert_eq!(
        session_of(&specs[2].session).0,
        "resume",
        "an earlier run completed"
    );
}

#[tokio::test(start_paused = true)]
async fn a_failed_first_run_is_retried_in_a_fresh_session() {
    let harness = harness().await;
    harness
        .agent
        .push_script(ReplayScript::failing(AgentFailureKind::Crashed, "boom"));
    let task_id = harness.submit("Color grade").await;
    run_for(100).await;
    harness
        .tasks
        .retry(harness.project_id, task_id)
        .await
        .unwrap();
    run_for(100).await;
    let specs = harness.agent.specs();
    let (first, second) = (session_of(&specs[0].session), session_of(&specs[1].session));
    assert_eq!((first.0, second.0), ("start", "start"));
    assert_ne!(first.1, second.1);
    assert_eq!(specs[1].prompt, "Color grade");
    let task = harness.task(task_id).await;
    assert_eq!(task.status, TaskStatus::Completed);
    assert_eq!(task.runs[1].run_kind, RunKind::Retry);
}

#[tokio::test(start_paused = true)]
async fn events_are_saved_and_published_as_they_arrive() {
    let harness = harness().await;
    harness.events.take();
    let task_id = harness.submit("Describe the timeline").await;
    run_for(100).await;
    assert_eq!(
        harness.event_kinds(task_id).await,
        ["session", "text", "usage", "completed"]
    );
    let published = harness.events.take();
    let appended = published
        .iter()
        .filter(|event| matches!(event, CoreEvent::TaskEventAppended { task_id: id, .. } if *id == task_id))
        .count();
    assert_eq!(appended, 4);
    let statuses: Vec<TaskStatus> = published
        .iter()
        .filter_map(|event| match event {
            CoreEvent::TaskUpdated { task } => Some(task.status.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(statuses.first(), Some(&TaskStatus::Queued));
    assert_eq!(statuses.last(), Some(&TaskStatus::Completed));
    let folder = harness.repository.folder(project_path()).unwrap();
    assert_eq!(folder.tasks[0].status, TaskStatus::Completed);
    assert_eq!(folder.task_events[&task_id].len(), 4);
    let spec = &harness.agent.specs()[0];
    assert!(
        spec.system_instructions
            .contains(&harness.sequence_id.to_string())
    );
    assert_eq!(
        spec.working_dir,
        Path::new("/agent-workdirs").join(task_id.to_string())
    );
}

#[tokio::test(start_paused = true)]
async fn tasks_left_running_by_a_stopped_core_fail_as_interrupted_when_the_project_reopens() {
    let harness = harness().await;
    harness.agent.set_default_script(ReplayScript::held());
    let running = harness.submit("Running").await;
    let mut settings = harness.tasks.ai_settings();
    settings.max_concurrent_tasks = 1;
    harness.tasks.update_ai_settings(settings).await.unwrap();
    let queued = harness.submit("Queued").await;
    run_for(100).await;
    // The core "crashes": a new core finds the folder as it was left.
    let folder = harness.repository.folder(project_path()).unwrap();
    let restarted = MediaTestbed::new(Path::new(PARENT));
    restarted.tasks.start().await.unwrap();
    restarted
        .repository
        .insert(project_path(), folder.snapshot.clone(), Vec::new());
    restarted
        .repository
        .set_tasks(project_path(), folder.tasks.clone());
    restarted
        .projects
        .open_project(&project_path())
        .await
        .unwrap();
    let tasks = restarted.tasks.list(harness.project_id).await.unwrap();
    for task in &tasks {
        assert!(
            matches!(
                task.status,
                TaskStatus::Failed {
                    failure: AgentFailureKind::Interrupted,
                    ..
                }
            ),
            "{:?}",
            task.status
        );
    }
    assert_eq!(
        tasks.iter().map(|task| task.id).collect::<Vec<_>>(),
        [running, queued]
    );
    let saved = restarted.repository.folder(project_path()).unwrap();
    assert!(matches!(saved.tasks[0].status, TaskStatus::Failed { .. }));
    restarted
        .tasks
        .retry(harness.project_id, running)
        .await
        .unwrap();
    run_for(100).await;
    assert_eq!(
        restarted.tasks.list(harness.project_id).await.unwrap()[0].status,
        TaskStatus::Completed
    );
}

#[tokio::test(start_paused = true)]
async fn closing_a_project_cancels_its_tasks() {
    let harness = harness().await;
    let mut settings = harness.tasks.ai_settings();
    settings.max_concurrent_tasks = 1;
    harness.tasks.update_ai_settings(settings).await.unwrap();
    harness.agent.set_default_script(ReplayScript::held());
    harness.submit("Running").await;
    harness.submit("Queued").await;
    run_for(100).await;
    harness
        .projects
        .close_project(harness.project_id)
        .await
        .unwrap();
    assert_eq!((harness.agent.alive(), harness.agent.killed()), (0, 1));
    let folder = harness.repository.folder(project_path()).unwrap();
    let statuses: Vec<&TaskStatus> = folder.tasks.iter().map(|task| &task.status).collect();
    assert_eq!(statuses, [&TaskStatus::Cancelled, &TaskStatus::Cancelled]);
    let closed = harness.tasks.list(harness.project_id).await;
    assert_eq!(code(closed.unwrap_err()), ErrorCode::ProjectNotOpen);
}

#[tokio::test(start_paused = true)]
async fn tasks_are_refused_while_the_cli_is_missing_or_logged_out() {
    let harness = harness().await;
    harness
        .agent
        .set_installation(Err(AgentError::NotInstalled {
            message: "replay was not found".to_owned(),
        }));
    harness.tasks.refresh_ai_status().await;
    let missing = harness
        .tasks
        .submit(harness.request("Anything"))
        .await
        .unwrap_err();
    assert_eq!(code(missing.clone()), ErrorCode::AiUnavailable);
    assert!(missing.to_string().contains("replay was not found"));
    harness
        .agent
        .set_installation(Ok(se_app::ports::ProviderInstallation {
            path: PathBuf::from("/bin/replay"),
            version: "1.0.0".to_owned(),
        }));
    harness.agent.set_auth(Ok(AuthStatus::LoggedOut));
    harness.events.take();
    let status = harness.tasks.refresh_ai_status().await;
    assert_eq!(status.auth, AuthStatus::LoggedOut);
    assert!(
        harness
            .events
            .take()
            .iter()
            .any(|event| matches!(event, CoreEvent::AiStatusChanged(_)))
    );
    let logged_out = harness
        .tasks
        .submit(harness.request("Anything"))
        .await
        .unwrap_err();
    assert_eq!(code(logged_out.clone()), ErrorCode::AiUnavailable);
    assert!(logged_out.to_string().contains("/login"), "{logged_out}");
    assert_eq!(harness.tasks.list(harness.project_id).await.unwrap(), []);
}

#[tokio::test(start_paused = true)]
async fn a_run_refused_for_lack_of_login_marks_the_provider_logged_out() {
    let harness = harness().await;
    harness.tasks.refresh_ai_status().await;
    harness.agent.push_script(ReplayScript::failing(
        AgentFailureKind::NotAuthenticated,
        "not logged in",
    ));
    let task_id = harness.submit("Anything").await;
    run_for(100).await;
    assert!(matches!(
        harness.task(task_id).await.status,
        TaskStatus::Failed {
            failure: AgentFailureKind::NotAuthenticated,
            ..
        }
    ));
    assert_eq!(harness.tasks.ai_status().await.auth, AuthStatus::LoggedOut);
}

#[tokio::test(start_paused = true)]
async fn start_failures_and_silent_agents_fail_their_task_with_a_reason() {
    let harness = harness().await;
    // One at a time: each failure must reach its own task.
    harness.agent.fail_next_start(AgentError::Failed {
        message: "could not start".to_owned(),
    });
    let refused = harness.submit("One").await;
    run_for(100).await;
    harness.workspaces.fail_next("disk full");
    let no_folder = harness.submit("Two").await;
    run_for(100).await;
    harness
        .agent
        .push_script(ReplayScript(vec![ReplayStep::StartSession]));
    let silent = harness.submit("Three").await;
    run_for(100).await;
    for (task_id, expected) in [
        (refused, "could not start"),
        (no_folder, "disk full"),
        (silent, "without reporting a result"),
    ] {
        let TaskStatus::Failed { failure, message } = harness.task(task_id).await.status else {
            panic!("task {task_id} must fail");
        };
        assert_eq!(failure, AgentFailureKind::Crashed);
        assert!(message.contains(expected), "{message}");
        assert_eq!(harness.event_kinds(task_id).await.last().unwrap(), "failed");
    }
}

#[tokio::test(start_paused = true)]
async fn settings_choose_defaults_the_limit_and_the_executable() {
    let harness = harness().await;
    let detections = harness.agent.detections();
    let settings = AiSettings {
        provider: ProviderId(REPLAY_PROVIDER_ID.to_owned()),
        executable_path: Some("/opt/replay/bin/replay".to_owned()),
        default_model: Some("small".to_owned()),
        default_effort: Some(se_protocol::EffortLevel::High),
        max_concurrent_tasks: 2,
    };
    let saved = harness
        .tasks
        .update_ai_settings(settings.clone())
        .await
        .unwrap();
    assert_eq!(saved, settings);
    assert_eq!(
        harness.agent.executable(),
        Some(PathBuf::from("/opt/replay/bin/replay"))
    );
    assert!(
        harness.agent.detections() > detections,
        "the CLI is checked again"
    );
    let task = harness
        .tasks
        .submit(harness.request("Defaults"))
        .await
        .unwrap();
    assert_eq!(task.model.as_deref(), Some("small"));
    assert_eq!(task.effort, Some(se_protocol::EffortLevel::High));
    let mut unknown = settings;
    unknown.provider = ProviderId("other".to_owned());
    let refused = harness.tasks.update_ai_settings(unknown).await;
    assert_eq!(code(refused.unwrap_err()), ErrorCode::InvalidRequest);
}

#[tokio::test(start_paused = true)]
async fn the_scope_and_prompt_are_checked() {
    let harness = harness().await;
    let mut request = harness.request("   ");
    assert_eq!(
        code(harness.tasks.submit(request.clone()).await.unwrap_err()),
        ErrorCode::InvalidRequest
    );
    request.prompt = "Trim".to_owned();
    request.scope.sequence_id = SequenceId::from_test_index(99);
    assert_eq!(
        code(harness.tasks.submit(request.clone()).await.unwrap_err()),
        ErrorCode::InvalidRequest
    );
    request.scope.sequence_id = harness.sequence_id;
    request.scope.tracks = Some(vec![TrackId::from_test_index(99)]);
    assert_eq!(
        code(harness.tasks.submit(request.clone()).await.unwrap_err()),
        ErrorCode::InvalidRequest
    );
    request.scope.tracks = Some(vec![harness.track_id]);
    let task = harness.tasks.submit(request).await.unwrap();
    assert_eq!(task.scope.tracks, Some(vec![harness.track_id]));
}

#[tokio::test(start_paused = true)]
async fn each_run_gets_an_mcp_token_bound_to_its_task_until_it_ends() {
    let harness = harness().await;
    harness.agent.set_default_script(ReplayScript::held());
    let task_id = harness.submit("Describe the first seconds").await;
    run_for(100).await;
    let spec = harness.agent.specs().remove(0);
    assert_eq!(spec.mcp.url, se_app::test_support::TESTBED_MCP_URL);
    let context = harness
        .tokens
        .resolve(&spec.mcp.bearer_token)
        .expect("the running task's token resolves");
    assert_eq!(context.task_id, task_id);
    assert_eq!(context.project_id, harness.project_id);
    assert_eq!(context.run, 0);
    assert_eq!(context.sequence_id(), harness.sequence_id);
    assert!(spec.system_instructions.contains("# Reading the project"));
    assert!(spec.system_instructions.contains("« Film »"));
    harness.agent.release(1);
    run_for(100).await;
    assert_eq!(harness.task(task_id).await.status, TaskStatus::Completed);
    assert_eq!(harness.tokens.resolve(&spec.mcp.bearer_token), None);
    assert_eq!(harness.tokens.active_count(), 0);
}

#[tokio::test(start_paused = true)]
async fn follow_ups_get_a_new_token_and_cancelled_runs_lose_theirs() {
    let harness = harness().await;
    harness
        .agent
        .set_default_script(ReplayScript::success("ok"));
    let task_id = harness.submit("First").await;
    run_for(100).await;
    harness.agent.set_default_script(ReplayScript::held());
    harness
        .tasks
        .follow_up(harness.project_id, task_id, "Second")
        .await
        .unwrap();
    run_for(100).await;
    let specs = harness.agent.specs();
    assert_ne!(specs[0].mcp.bearer_token, specs[1].mcp.bearer_token);
    let context = harness.tokens.resolve(&specs[1].mcp.bearer_token).unwrap();
    assert_eq!(context.run, 1);
    harness
        .tasks
        .cancel(harness.project_id, task_id)
        .await
        .unwrap();
    run_for(6_000).await;
    assert_eq!(harness.task(task_id).await.status, TaskStatus::Cancelled);
    assert_eq!(harness.tokens.active_count(), 0);
}
