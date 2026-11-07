//! AI tasks and their event logs in a real `project.db`, through `FsProjectStore::tasks()`.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use se_app::ports::ProjectRepository;
use se_domain::{ProjectId, SequenceId, TaskId, test_support::example_project};
use se_protocol::{
    AgentEvent, AgentFailureKind, ProviderId, RunKind, Task, TaskRun, TaskScope, TaskStatus,
};
use se_store::FsProjectRepository;

fn task() -> Task {
    Task {
        id: TaskId::from_test_index(1),
        project_id: ProjectId::from_test_index(1),
        prompt: "Remove the silences".to_owned(),
        scope: TaskScope {
            sequence_id: SequenceId::from_test_index(2),
            range: None,
            tracks: None,
        },
        provider: ProviderId("claude-code".to_owned()),
        model: None,
        effort: None,
        status: TaskStatus::Running,
        agent_session_id: Some("6f1c2b8e-0d47-4bb8-9c4e-2a3d5e6f7a8b".to_owned()),
        created_at: 1_700_000_000_000,
        updated_at: 1_700_000_000_500,
        runs: vec![TaskRun {
            run_kind: RunKind::Initial,
            prompt: "Remove the silences".to_owned(),
            status: TaskStatus::Running,
            queued_at: 1_700_000_000_000,
            started_at: Some(1_700_000_000_100),
            finished_at: None,
        }],
    }
}

#[tokio::test]
async fn tasks_and_their_logs_survive_reopening_the_project() {
    let parent = tempfile::tempdir().unwrap();
    let repository = FsProjectRepository;
    let path = repository
        .create(parent.path(), &example_project())
        .await
        .unwrap();
    let store = repository.open(&path).await.unwrap();
    let tasks = store.tasks();
    tasks.save_task(&task()).await.unwrap();
    let failed = AgentEvent::Failed {
        failure: AgentFailureKind::RateLimited,
        message: "usage limit reached".to_owned(),
    };
    let text = AgentEvent::AssistantText {
        text: "Looking at the timeline".to_owned(),
    };
    tasks
        .append_task_event(task().id, 0, 1_700_000_000_200, &text)
        .await
        .unwrap();
    tasks
        .append_task_event(task().id, 0, 1_700_000_000_300, &failed)
        .await
        .unwrap();
    // A crash: dropped without closing.
    drop(tasks);
    drop(store);

    let store = repository.open(&path).await.unwrap();
    let tasks = store.tasks();
    assert_eq!(tasks.load_tasks().await.unwrap(), [task()]);
    let events = tasks.load_task_events(task().id).await.unwrap();
    let kinds: Vec<(u32, &AgentEvent)> = events
        .iter()
        .map(|record| (record.sequence, &record.event))
        .collect();
    assert_eq!(kinds, [(0, &text), (1, &failed)]);
    store.close().await.unwrap();
}
